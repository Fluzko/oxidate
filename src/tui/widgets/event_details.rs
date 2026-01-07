use chrono::DateTime;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget, Wrap,
    },
};

use crate::calendar::models::Event;
use crate::tui::color_utils::{default_event_color, parse_hex_color};
use crate::tui::state::{AppState, ViewFocus};

pub struct EventDetailsWidget<'a> {
    state: &'a mut AppState,
    event_index: usize,
    scroll_offset: usize,
}

impl<'a> EventDetailsWidget<'a> {
    pub fn new(state: &'a mut AppState, event_index: usize, scroll_offset: usize) -> Self {
        Self {
            state,
            event_index,
            scroll_offset,
        }
    }

    fn format_time(event: &Event) -> String {
        if let Some(ref date_time_str) = event.start.date_time {
            if let Ok(start_dt) = DateTime::parse_from_rfc3339(date_time_str) {
                let start_time = start_dt.format("%H:%M").to_string();

                if let Some(ref end_date_time_str) = event.end.date_time {
                    if let Ok(end_dt) = DateTime::parse_from_rfc3339(end_date_time_str) {
                        let end_time = end_dt.format("%H:%M").to_string();
                        return format!("{} - {}", start_time, end_time);
                    }
                }

                return start_time;
            }
        }

        "All day".to_string()
    }

    /// Calculate maximum scroll offset for given content and visible area
    /// Returns 0 if content fits, otherwise returns lines that can be scrolled past
    fn calculate_max_scroll(content_lines: usize, visible_height: usize) -> usize {
        if content_lines <= visible_height {
            0
        } else {
            content_lines.saturating_sub(visible_height)
        }
    }
}

impl<'a> Widget for EventDetailsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_date = self.state.selected_date;
        let events = self.state.get_events_for_date(selected_date);

        // Create border with focus indicator
        let border_style = if self.state.view_focus == ViewFocus::Events {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Event Details ");

        let inner = block.inner(area);
        block.render(area, buf);

        // Check if event_index is valid
        if self.event_index >= events.len() {
            let error_text = vec![Line::from(Span::styled(
                "Error: Event not found",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))];
            let paragraph = Paragraph::new(error_text);
            paragraph.render(inner, buf);
            return;
        }

        let event = &events[self.event_index];
        let mut lines = Vec::new();

        if let Some(ref calendar_id) = event.calendar_id {
            if let Some(cal) = self.state.get_calendar_by_id(calendar_id) {
                let cal_color = cal
                    .background_color
                    .as_ref()
                    .and_then(|hex| parse_hex_color(hex))
                    .unwrap_or_else(default_event_color);

                lines.push(Line::from(vec![
                    Span::styled("▊▊ ", Style::default().fg(cal_color)),
                    Span::styled(&cal.summary, Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(""));
            }
        }

        let summary = event.summary.as_deref().unwrap_or("(No title)");
        lines.push(Line::from(Span::styled(
            summary,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Time
        let time_str = Self::format_time(event);
        lines.push(Line::from(vec![
            Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(time_str),
        ]));
        lines.push(Line::from(""));

        // Location
        if let Some(ref location) = event.location {
            lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(location, Style::default().fg(Color::Yellow)),
            ]));
            lines.push(Line::from(""));
        }

        // Description
        if let Some(ref description) = event.description {
            lines.push(Line::from(Span::styled(
                "Description:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::raw(description)));
            lines.push(Line::from(""));
        }

        // Status
        if let Some(ref status) = event.status {
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(status),
            ]));
            lines.push(Line::from(""));
        }

        // Attendees
        if let Some(ref attendees) = event.attendees {
            if !attendees.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Attendees:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));

                for attendee in attendees {
                    let name = attendee.display_name.as_deref().unwrap_or(&attendee.email);
                    let status_icon = match attendee.response_status.as_deref() {
                        Some("accepted") => "\u{2713}", // ✓
                        Some("declined") => "\u{2717}", // ✗
                        Some("tentative") => "?",
                        _ => "-",
                    };

                    let optional_marker = if attendee.optional == Some(true) {
                        " (optional)"
                    } else {
                        ""
                    };

                    lines.push(Line::from(Span::styled(
                        format!("  {} {}{}", status_icon, name, optional_marker),
                        Style::default(),
                    )));
                }

                lines.push(Line::from(""));
            }
        }

        // Google Calendar Link
        if let Some(ref link) = event.html_link {
            lines.push(Line::from(vec![
                Span::styled("Link: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(link, Style::default().fg(Color::Blue)),
            ]));
            lines.push(Line::from(""));
        }

        // Help hint
        lines.push(Line::from(Span::styled(
            "Press Esc to return, j/k to scroll",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));

        let content_height = lines.len();
        let visible_height = inner.height as usize;
        let max_scroll = Self::calculate_max_scroll(content_height, visible_height);

        let scroll_offset = self.scroll_offset.min(max_scroll);

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset as u16, 0));
        paragraph.render(inner, buf);

        if content_height > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(max_scroll)
                .position(scroll_offset);

            scrollbar.render(inner, buf, &mut scrollbar_state);
        }

        self.state.update_event_details_max_scroll(max_scroll);

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::models::{Attendee, EventDateTime};
    use chrono::NaiveDate;

    #[test]
    fn test_event_details_widget_new() {
        let mut state = AppState::new();
        let widget = EventDetailsWidget::new(&mut state, 0, 0);
        assert_eq!(widget.event_index, 0);
    }

    #[test]
    fn test_renders_with_all_fields() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;

        let event = Event {
            id: "1".to_string(),
            summary: Some("Team Meeting".to_string()),
            description: Some("Discuss Q2 roadmap and priorities".to_string()),
            location: Some("Conference Room A".to_string()),
            start: EventDateTime {
                date_time: Some("2025-06-15T10:00:00Z".to_string()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some("2025-06-15T11:00:00Z".to_string()),
                date: None,
                time_zone: None,
            },
            status: Some("confirmed".to_string()),
            html_link: Some("https://calendar.google.com/event?eid=test123".to_string()),
            attendees: Some(vec![
                Attendee {
                    email: "alice@example.com".to_string(),
                    display_name: Some("Alice Smith".to_string()),
                    response_status: Some("accepted".to_string()),
                    optional: Some(false),
                },
                Attendee {
                    email: "bob@example.com".to_string(),
                    display_name: Some("Bob Jones".to_string()),
                    response_status: Some("tentative".to_string()),
                    optional: Some(true),
                },
            ]),
            calendar_id: None,
        };

        state.events.insert(date, vec![event]);

        let widget = EventDetailsWidget::new(&mut state, 0, 0);

        // Widget should have access to all fields
        assert_eq!(widget.state.selected_date, date);
        assert_eq!(widget.event_index, 0);
    }

    #[test]
    fn test_renders_with_minimal_fields() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;

        let event = Event {
            id: "1".to_string(),
            summary: None,
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
            calendar_id: None,
        };

        state.events.insert(date, vec![event]);

        let widget = EventDetailsWidget::new(&mut state, 0, 0);

        // Should handle minimal fields without crashing
        assert_eq!(widget.event_index, 0);
    }

    #[test]
    fn test_handles_invalid_index() {
        let mut state = AppState::new();

        // Create widget with out-of-bounds index
        let widget = EventDetailsWidget::new(&mut state, 99, 0);

        // Should not panic, just have invalid index
        assert_eq!(widget.event_index, 99);
    }

    #[test]
    fn test_calculate_max_scroll_with_overflow() {
        let max_scroll = EventDetailsWidget::calculate_max_scroll(100, 20);
        assert_eq!(max_scroll, 80);
    }

    #[test]
    fn test_calculate_max_scroll_no_overflow() {
        let max_scroll = EventDetailsWidget::calculate_max_scroll(10, 20);
        assert_eq!(max_scroll, 0);
    }

    #[test]
    fn test_calculate_max_scroll_exact_fit() {
        let max_scroll = EventDetailsWidget::calculate_max_scroll(20, 20);
        assert_eq!(max_scroll, 0);
    }

    #[test]
    fn test_widget_accepts_scroll_offset() {
        let mut state = AppState::new();
        let widget = EventDetailsWidget::new(&mut state, 0, 5);
        assert_eq!(widget.scroll_offset, 5);
    }

    #[test]
    fn test_widget_new_with_zero_scroll() {
        let mut state = AppState::new();
        let widget = EventDetailsWidget::new(&mut state, 0, 0);
        assert_eq!(widget.scroll_offset, 0);
    }
}
