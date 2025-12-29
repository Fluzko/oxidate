use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::Duration;

use crate::calendar::client::CalendarClient;
use super::{
    input::{handle_key_event, InputAction},
    loader::{DataLoader, DataMessage},
    state::{AppState, DateRange},
    widgets::{CalendarWidget, EventListWidget},
};

pub fn run_tui(client: CalendarClient) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app_state = AppState::new();

    // Start data loader
    let date_range = DateRange::five_month_span(Local::now().date_naive());
    let mut data_loader = Some(DataLoader::new(client, date_range));

    // Main event loop
    let result = run_app(&mut terminal, &mut app_state, &mut data_loader);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app_state: &mut AppState,
    data_loader: &mut Option<DataLoader>,
) -> Result<()> {
    loop {
        // Check for data updates from loader
        if let Some(loader) = data_loader {
            if let Some(message) = loader.try_recv() {
                match message {
                    DataMessage::Loading => {
                        app_state.loading = true;
                        app_state.error = None;
                    }
                    DataMessage::Success { calendars, events } => {
                        app_state.calendars = calendars;
                        app_state.events = events;
                        app_state.loading = false;
                        app_state.error = None;
                        *data_loader = None; // Drop loader after success
                    }
                    DataMessage::Error(err) => {
                        app_state.loading = false;
                        app_state.error = Some(err);
                        *data_loader = None; // Drop loader after error
                    }
                }
            }
        }

        // Render UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
                .split(f.area());

            // Render calendar widget
            let calendar_widget = CalendarWidget::new(app_state);
            f.render_widget(calendar_widget, chunks[0]);

            // Render events widget
            let events_widget = EventListWidget::new(app_state);
            f.render_widget(events_widget, chunks[1]);

            // Render status bar at the bottom
            render_status_bar(f, app_state);
        })?;

        // Handle input (non-blocking with timeout)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_key_event(key, app_state) {
                    InputAction::Quit => break,
                    InputAction::Refresh => {
                        // TODO: Implement refresh logic
                    }
                    InputAction::None => {}
                }
            }
        }
    }

    Ok(())
}

fn render_status_bar(f: &mut ratatui::Frame, app_state: &AppState) {
    let status_area = Rect {
        x: 0,
        y: f.area().height.saturating_sub(3),
        width: f.area().width,
        height: 3,
    };

    let status_text = if app_state.loading {
        vec![Line::from(Span::styled(
            "Loading calendars and events...",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))]
    } else if let Some(ref error) = app_state.error {
        vec![Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))]
    } else {
        vec![Line::from(vec![
            Span::raw("Keys: "),
            Span::styled("←→↑↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate | "),
            Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Today | "),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Switch View | "),
            Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Refresh | "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ])]
    };

    let status_block = Block::default()
        .borders(Borders::TOP)
        .title(" Status ");

    let status_paragraph = Paragraph::new(status_text)
        .block(status_block);

    f.render_widget(status_paragraph, status_area);
}
