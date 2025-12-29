use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

use crate::calendar::models::{Calendar, Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewFocus {
    Calendar,
    Events,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventsViewMode {
    List,
    Details { event_index: usize },
}

#[derive(Debug)]
pub struct AppState {
    pub selected_date: NaiveDate,
    pub today: NaiveDate,
    pub calendars: Vec<Calendar>,
    pub events: HashMap<NaiveDate, Vec<Event>>,
    pub loading: bool,
    pub error: Option<String>,
    pub view_focus: ViewFocus,
    pub selected_event_index: Option<usize>,
    pub events_view_mode: EventsViewMode,
}

impl AppState {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            selected_date: today,
            today,
            calendars: Vec::new(),
            events: HashMap::new(),
            loading: true,
            error: None,
            view_focus: ViewFocus::Calendar,
            selected_event_index: None,
            events_view_mode: EventsViewMode::List,
        }
    }

    pub fn get_events_for_date(&self, date: NaiveDate) -> Vec<&Event> {
        self.events.get(&date).map(|v| v.iter().collect()).unwrap_or_default()
    }

    pub fn has_events(&self, date: NaiveDate) -> bool {
        self.events.get(&date).map(|v| !v.is_empty()).unwrap_or(false)
    }

    pub fn move_selected_date(&mut self, days: i64) {
        if let Some(new_date) = self.selected_date.checked_add_signed(chrono::Duration::days(days)) {
            self.selected_date = new_date;
        }
    }

    pub fn move_to_next_week(&mut self) {
        self.move_selected_date(7);
    }

    pub fn move_to_prev_week(&mut self) {
        self.move_selected_date(-7);
    }

    pub fn toggle_focus(&mut self) {
        self.view_focus = match self.view_focus {
            ViewFocus::Calendar => ViewFocus::Events,
            ViewFocus::Events => ViewFocus::Calendar,
        };
    }

    pub fn jump_to_today(&mut self) {
        self.selected_date = self.today;
    }

    pub fn move_event_selection_down(&mut self) {
        let events = self.get_events_for_date(self.selected_date);
        let event_count = events.len();

        if event_count == 0 {
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            None => 0,
            Some(idx) => (idx + 1) % event_count,
        });
    }

    pub fn move_event_selection_up(&mut self) {
        let events = self.get_events_for_date(self.selected_date);
        let event_count = events.len();

        if event_count == 0 {
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            None => event_count - 1,
            Some(0) => event_count - 1,
            Some(idx) => idx - 1,
        });
    }
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn five_month_span(center_date: NaiveDate) -> Self {
        // Calculate start: 2 months before
        let start = if center_date.month() <= 2 {
            // Handle year boundary
            let year = center_date.year() - 1;
            let month = center_date.month() + 10; // 12 - (2 - month)
            NaiveDate::from_ymd_opt(year, month, 1).unwrap()
        } else {
            let month = center_date.month() - 2;
            NaiveDate::from_ymd_opt(center_date.year(), month, 1).unwrap()
        };

        // Calculate end: 2 months after, last day of that month
        let end = if center_date.month() >= 11 {
            // Handle year boundary
            let year = center_date.year() + 1;
            let month = center_date.month() - 10;
            Self::last_day_of_month(year, month)
        } else {
            let month = center_date.month() + 2;
            Self::last_day_of_month(center_date.year(), month)
        };

        Self { start, end }
    }

    fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
        // First day of next month
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        // Subtract one day to get last day of current month
        first_of_next.pred_opt().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();

        assert_eq!(state.selected_date, Local::now().date_naive());
        assert_eq!(state.calendars.len(), 0);
        assert_eq!(state.events.len(), 0);
        assert!(state.loading);
        assert_eq!(state.error, None);
        assert_eq!(state.view_focus, ViewFocus::Calendar);
    }

    #[test]
    fn test_move_selected_date() {
        let mut state = AppState::new();
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        state.move_selected_date(1);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 16).unwrap());

        state.move_selected_date(-3);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 13).unwrap());
    }

    #[test]
    fn test_move_week() {
        let mut state = AppState::new();
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        state.move_to_next_week();
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 22).unwrap());

        state.move_to_prev_week();
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    #[test]
    fn test_toggle_focus() {
        let mut state = AppState::new();

        assert_eq!(state.view_focus, ViewFocus::Calendar);

        state.toggle_focus();
        assert_eq!(state.view_focus, ViewFocus::Events);

        state.toggle_focus();
        assert_eq!(state.view_focus, ViewFocus::Calendar);
    }

    #[test]
    fn test_date_range_five_month_span_normal_case() {
        let center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let range = DateRange::five_month_span(center);

        // 2 months before June = April
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2025, 4, 1).unwrap());

        // 2 months after June = August, last day (31st)
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2025, 8, 31).unwrap());
    }

    #[test]
    fn test_date_range_five_month_span_year_boundary_start() {
        let center = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let range = DateRange::five_month_span(center);

        // 2 months before January = November of previous year
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 11, 1).unwrap());

        // 2 months after January = March
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());
    }

    #[test]
    fn test_date_range_five_month_span_year_boundary_end() {
        let center = NaiveDate::from_ymd_opt(2025, 12, 15).unwrap();
        let range = DateRange::five_month_span(center);

        // 2 months before December = October
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());

        // 2 months after December = February of next year (28/29 days)
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_date_range_last_day_of_month() {
        // Test various months
        assert_eq!(
            DateRange::last_day_of_month(2025, 1),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()
        );
        assert_eq!(
            DateRange::last_day_of_month(2025, 2),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
        );
        assert_eq!(
            DateRange::last_day_of_month(2024, 2), // Leap year
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        assert_eq!(
            DateRange::last_day_of_month(2025, 4),
            NaiveDate::from_ymd_opt(2025, 4, 30).unwrap()
        );
        assert_eq!(
            DateRange::last_day_of_month(2025, 12),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn test_has_events() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        assert!(!state.has_events(date));

        // Add an event
        state.events.insert(date, vec![]);
        assert!(!state.has_events(date)); // Empty vec

        // Add real event (minimal event structure for testing)
        use crate::calendar::models::{Event, EventDateTime};
        let event = Event {
            id: "test".to_string(),
            summary: Some("Test Event".to_string()),
            description: None,
            location: None,
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
            status: None,
            html_link: None,
            attendees: None,
        };
        state.events.insert(date, vec![event]);
        assert!(state.has_events(date));
    }

    #[test]
    fn test_app_state_today_initialized() {
        let state = AppState::new();
        let expected_today = Local::now().date_naive();
        assert_eq!(state.today, expected_today);
        assert_eq!(state.selected_date, state.today);
    }

    #[test]
    fn test_jump_to_today() {
        let mut state = AppState::new();
        state.selected_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_ne!(state.selected_date, state.today);

        state.jump_to_today();
        assert_eq!(state.selected_date, state.today);
    }

    #[test]
    fn test_today_remains_constant_after_navigation() {
        let mut state = AppState::new();
        let original_today = state.today;

        state.move_selected_date(5);
        state.move_to_next_week();

        assert_eq!(state.today, original_today);
    }

    #[test]
    fn test_event_selection_initialization() {
        let state = AppState::new();
        assert_eq!(state.selected_event_index, None);
        assert!(matches!(state.events_view_mode, EventsViewMode::List));
    }

    #[test]
    fn test_move_event_selection_down() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;

        // Add some test events
        use crate::calendar::models::{Event, EventDateTime};
        let events = vec![
            Event {
                id: "1".to_string(),
                summary: Some("Event 1".to_string()),
                description: None,
                location: None,
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
                status: None,
                html_link: None,
                attendees: None,
            },
            Event {
                id: "2".to_string(),
                summary: Some("Event 2".to_string()),
                description: None,
                location: None,
                start: EventDateTime {
                    date_time: Some("2025-06-15T14:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                },
                end: EventDateTime {
                    date_time: Some("2025-06-15T15:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                },
                status: None,
                html_link: None,
                attendees: None,
            },
        ];
        state.events.insert(date, events);

        // Start with no selection, should select index 0
        assert_eq!(state.selected_event_index, None);
        state.move_event_selection_down();
        assert_eq!(state.selected_event_index, Some(0));

        // Move down to index 1
        state.move_event_selection_down();
        assert_eq!(state.selected_event_index, Some(1));

        // Wrap around to index 0
        state.move_event_selection_down();
        assert_eq!(state.selected_event_index, Some(0));
    }

    #[test]
    fn test_move_event_selection_up() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;

        // Add some test events
        use crate::calendar::models::{Event, EventDateTime};
        let events = vec![
            Event {
                id: "1".to_string(),
                summary: Some("Event 1".to_string()),
                description: None,
                location: None,
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
                status: None,
                html_link: None,
                attendees: None,
            },
            Event {
                id: "2".to_string(),
                summary: Some("Event 2".to_string()),
                description: None,
                location: None,
                start: EventDateTime {
                    date_time: Some("2025-06-15T14:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                },
                end: EventDateTime {
                    date_time: Some("2025-06-15T15:00:00Z".to_string()),
                    date: None,
                    time_zone: None,
                },
                status: None,
                html_link: None,
                attendees: None,
            },
        ];
        state.events.insert(date, events);

        // Start with no selection, should select last index (1)
        assert_eq!(state.selected_event_index, None);
        state.move_event_selection_up();
        assert_eq!(state.selected_event_index, Some(1));

        // Move up to index 0
        state.move_event_selection_up();
        assert_eq!(state.selected_event_index, Some(0));

        // Wrap around to last index (1)
        state.move_event_selection_up();
        assert_eq!(state.selected_event_index, Some(1));
    }

    #[test]
    fn test_move_event_selection_no_events() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;

        // No events for this date
        assert_eq!(state.selected_event_index, None);

        state.move_event_selection_down();
        assert_eq!(state.selected_event_index, None);

        state.move_event_selection_up();
        assert_eq!(state.selected_event_index, None);
    }
}
