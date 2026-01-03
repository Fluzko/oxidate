use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event},
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

use super::{
    input::{handle_key_event, InputAction},
    loader::{DataLoader, DataMessage},
    state::{AppState, DateRange, EventsViewMode, ViewFocus},
    widgets::{CalendarWidget, EventDetailsWidget, EventListWidget},
};
use crate::calendar::client::CalendarClient;

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
                    DataMessage::Success {
                        calendars,
                        events,
                        client,
                    } => {
                        app_state.calendars = calendars;
                        app_state.events = events;
                        app_state.loading = false;
                        app_state.error = None;
                        *data_loader = None; // Drop loader after success
                        // TODO: Store client for reuse (Step 3)
                        drop(client); // Temporary: drop client until Step 3
                    }
                    DataMessage::Error { error, client } => {
                        app_state.loading = false;
                        app_state.error = Some(error);
                        *data_loader = None; // Drop loader after error
                        // TODO: Store client for reuse (Step 3)
                        drop(client); // Temporary: drop client until Step 3
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

            // Render events widget based on mode
            match app_state.events_view_mode {
                EventsViewMode::List => {
                    let events_widget = EventListWidget::new(app_state);
                    f.render_widget(events_widget, chunks[1]);
                }
                EventsViewMode::Details { event_index } => {
                    let details_widget = EventDetailsWidget::new(app_state, event_index);
                    f.render_widget(details_widget, chunks[1]);
                }
            }

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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))]
    } else if let Some(ref error) = app_state.error {
        vec![Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))]
    } else {
        // Show different hints based on focus and mode
        match (app_state.view_focus, app_state.events_view_mode) {
            (ViewFocus::Calendar, _) => {
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
            }
            (ViewFocus::Events, EventsViewMode::List) => {
                vec![Line::from(vec![
                    Span::raw("Keys: "),
                    Span::styled(
                        "\u{2191}\u{2193}",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Select | "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Details | "),
                    Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Switch View | "),
                    Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Today | "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Quit"),
                ])]
            }
            (ViewFocus::Events, EventsViewMode::Details { .. }) => {
                vec![Line::from(vec![
                    Span::raw("Keys: "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Back to List | "),
                    Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Switch View | "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Quit"),
                ])]
            }
        }
    };

    let status_block = Block::default().borders(Borders::TOP).title(" Status ");

    let status_paragraph = Paragraph::new(status_text).block(status_block);

    f.render_widget(status_paragraph, status_area);
}
