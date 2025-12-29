use chrono::DateTime;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::calendar::models::Event;
use crate::tui::state::{AppState, ViewFocus};

pub struct EventListWidget<'a> {
    state: &'a AppState,
}

impl<'a> EventListWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn format_event_time(event: &Event) -> String {
        // Try to extract time from dateTime field
        if let Some(ref date_time_str) = event.start.date_time {
            if let Ok(start_dt) = DateTime::parse_from_rfc3339(date_time_str) {
                let start_time = start_dt.format("%H:%M").to_string();

                // Try to get end time
                if let Some(ref end_date_time_str) = event.end.date_time {
                    if let Ok(end_dt) = DateTime::parse_from_rfc3339(end_date_time_str) {
                        let end_time = end_dt.format("%H:%M").to_string();
                        return format!("{} - {}", start_time, end_time);
                    }
                }

                return start_time;
            }
        }

        // All-day event
        "All day".to_string()
    }
}

impl<'a> Widget for EventListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_date = self.state.selected_date;
        let events = self.state.get_events_for_date(selected_date);

        // Create border with focus indicator
        let border_style = if self.state.view_focus == ViewFocus::Events {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let title = format!(
            " Events for {} ",
            selected_date.format("%B %d, %Y")
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let inner = block.inner(area);
        block.render(area, buf);

        if events.is_empty() {
            // No events for this date
            let no_events_text = vec![Line::from(Span::styled(
                "No events for this date",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ))];

            let paragraph = Paragraph::new(no_events_text)
                .wrap(Wrap { trim: true });
            paragraph.render(inner, buf);
            return;
        }

        // Render events
        let mut lines = Vec::new();

        for (i, event) in events.iter().enumerate() {
            let is_selected = self.state.selected_event_index == Some(i)
                             && self.state.view_focus == ViewFocus::Events;

            // Selection indicator and time
            let time_str = Self::format_event_time(event);
            let indicator = if is_selected { "> " } else { "  " };

            let time_span = Span::styled(
                format!("{}{}", indicator, time_str),
                if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                },
            );

            // Summary
            let summary = event.summary.as_deref().unwrap_or("(No title)");
            let summary_span = Span::styled(
                format!(" {}", summary),
                if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                },
            );

            lines.push(Line::from(vec![time_span, summary_span]));

            // Location (if available)
            if let Some(ref location) = event.location {
                let location_style = if is_selected {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                lines.push(Line::from(Span::styled(
                    format!("    \u{1f4cd} {}", location),
                    location_style,
                )));
            }

            // Add spacing between events (except last one)
            if i < events.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        // Add help hint when focused
        if self.state.view_focus == ViewFocus::Events {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "(\u{2191}\u{2193} to select, Enter for details)",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true });
        paragraph.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::models::EventDateTime;
    use chrono::Local;

    #[test]
    fn test_format_event_time_with_datetime() {
        let event = Event {
            id: "test".to_string(),
            summary: Some("Meeting".to_string()),
            description: None,
            location: None,
            start: EventDateTime {
                date_time: Some("2025-06-15T10:30:00-05:00".to_string()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some("2025-06-15T11:30:00-05:00".to_string()),
                date: None,
                time_zone: None,
            },
            status: None,
            html_link: None,
            attendees: None,
        };

        let time_str = EventListWidget::format_event_time(&event);
        assert!(time_str.contains("10:30"));
        assert!(time_str.contains("11:30"));
        assert!(time_str.contains(" - "));
    }

    #[test]
    fn test_format_event_time_with_date_only() {
        let event = Event {
            id: "test".to_string(),
            summary: Some("All-day event".to_string()),
            description: None,
            location: None,
            start: EventDateTime {
                date_time: None,
                date: Some("2025-06-15".to_string()),
                time_zone: None,
            },
            end: EventDateTime {
                date_time: None,
                date: Some("2025-06-15".to_string()),
                time_zone: None,
            },
            status: None,
            html_link: None,
            attendees: None,
        };

        let time_str = EventListWidget::format_event_time(&event);
        assert_eq!(time_str, "All day");
    }

    #[test]
    fn test_event_list_widget_new() {
        let state = AppState::new();
        let widget = EventListWidget::new(&state);
        assert_eq!(widget.state.selected_date, Local::now().date_naive());
    }

    #[test]
    fn test_widget_reads_selected_index_from_state() {
        let mut state = AppState::new();
        state.selected_event_index = Some(2);

        let widget = EventListWidget::new(&state);

        assert_eq!(widget.state.selected_event_index, Some(2));
    }

    #[test]
    fn test_no_selection_when_no_events() {
        let state = AppState::new();

        let widget = EventListWidget::new(&state);

        assert_eq!(widget.state.selected_event_index, None);
    }
}
