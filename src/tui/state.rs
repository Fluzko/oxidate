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
    pub current_date_range: DateRange,
    pub current_month: (i32, u32),
}

impl AppState {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        let current_date_range = DateRange::five_month_span(today);
        let current_month = (today.year(), today.month());
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
            current_date_range,
            current_month,
        }
    }

    pub fn get_events_for_date(&self, date: NaiveDate) -> Vec<&Event> {
        self.events
            .get(&date)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_events(&self, date: NaiveDate) -> bool {
        self.events
            .get(&date)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    pub fn move_selected_date(&mut self, days: i64) {
        if let Some(new_date) = self
            .selected_date
            .checked_add_signed(chrono::Duration::days(days))
        {
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

    pub fn select_event(&mut self) {
        if let Some(index) = self.selected_event_index {
            self.events_view_mode = EventsViewMode::Details { event_index: index };
        }
    }

    pub fn exit_event_details(&mut self) {
        self.events_view_mode = EventsViewMode::List;
    }

    pub fn reset_event_selection(&mut self) {
        self.selected_event_index = None;
        self.events_view_mode = EventsViewMode::List;
    }

    pub fn needs_date_range_refresh(&self) -> bool {
        let selected_month = (self.selected_date.year(), self.selected_date.month());
        let start_month = (
            self.current_date_range.start.year(),
            self.current_date_range.start.month(),
        );
        let end_month = (
            self.current_date_range.end.year(),
            self.current_date_range.end.month(),
        );

        // Refresh if we're at the first or last month of the range
        selected_month == start_month || selected_month == end_month
    }

    pub fn update_date_range(&mut self, new_range: DateRange) {
        self.current_date_range = new_range;
    }

    pub fn trim_events_to_25_month_span(&mut self) {
        let cache_range = DateRange::twenty_five_month_span(self.selected_date);

        // Collect dates to remove (those outside the 25-month span)
        let dates_to_remove: Vec<NaiveDate> = self
            .events
            .keys()
            .filter(|&&date| {
                // Always preserve current month
                let date_month = (date.year(), date.month());
                if date_month == self.current_month {
                    return false;
                }

                // Remove if outside the cache range
                date < cache_range.start || date > cache_range.end
            })
            .copied()
            .collect();

        // Remove the dates
        for date in dates_to_remove {
            self.events.remove(&date);
        }
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

    pub fn twenty_five_month_span(center_date: NaiveDate) -> Self {
        // Calculate start: 12 months before center date
        let start_year;
        let start_month;

        if center_date.month() <= 12 {
            let months_back = 12;
            if center_date.month() as i32 - months_back <= 0 {
                // Need to go to previous year
                start_year = center_date.year() - 1;
                start_month = (12 + center_date.month() as i32 - months_back) as u32;
            } else {
                start_year = center_date.year();
                start_month = center_date.month() - months_back as u32;
            }
        } else {
            start_year = center_date.year();
            start_month = center_date.month();
        }

        let start = NaiveDate::from_ymd_opt(start_year, start_month, 1).unwrap();

        // Calculate end: 12 months after center date, last day of that month
        let end_year;
        let end_month;

        let months_ahead = 12;
        if center_date.month() + months_ahead > 12 {
            // Need to go to next year
            end_year = center_date.year() + 1;
            end_month = center_date.month() + months_ahead - 12;
        } else {
            end_year = center_date.year();
            end_month = center_date.month() + months_ahead;
        }

        let end = Self::last_day_of_month(end_year, end_month);

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
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2025, 6, 16).unwrap()
        );

        state.move_selected_date(-3);
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2025, 6, 13).unwrap()
        );
    }

    #[test]
    fn test_move_week() {
        let mut state = AppState::new();
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        state.move_to_next_week();
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2025, 6, 22).unwrap()
        );

        state.move_to_prev_week();
        assert_eq!(
            state.selected_date,
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()
        );
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

    #[test]
    fn test_select_event() {
        let mut state = AppState::new();
        state.selected_event_index = Some(2);

        assert!(matches!(state.events_view_mode, EventsViewMode::List));

        state.select_event();

        assert!(matches!(
            state.events_view_mode,
            EventsViewMode::Details { event_index: 2 }
        ));
    }

    #[test]
    fn test_select_event_with_no_selection() {
        let mut state = AppState::new();
        assert_eq!(state.selected_event_index, None);

        state.select_event();

        // Should still be in List mode since no event is selected
        assert!(matches!(state.events_view_mode, EventsViewMode::List));
    }

    #[test]
    fn test_exit_event_details() {
        let mut state = AppState::new();
        state.events_view_mode = EventsViewMode::Details { event_index: 1 };

        state.exit_event_details();

        assert!(matches!(state.events_view_mode, EventsViewMode::List));
    }

    #[test]
    fn test_reset_event_selection() {
        let mut state = AppState::new();
        state.selected_event_index = Some(3);
        state.events_view_mode = EventsViewMode::Details { event_index: 3 };

        state.reset_event_selection();

        assert_eq!(state.selected_event_index, None);
        assert!(matches!(state.events_view_mode, EventsViewMode::List));
    }

    #[test]
    fn test_twenty_five_month_span_calculation() {
        let center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let range = DateRange::twenty_five_month_span(center);

        // 12 months before June 2025 = June 2024
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());

        // 12 months after June 2025 = June 2026, last day (30th)
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }

    #[test]
    fn test_twenty_five_month_span_year_boundary() {
        let center = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let range = DateRange::twenty_five_month_span(center);

        // 12 months before January 2025 = January 2024
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        // 12 months after January 2025 = January 2026
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
    }

    #[test]
    fn test_needs_refresh_at_start_boundary() {
        let mut state = AppState::new();
        let center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.current_date_range = DateRange::five_month_span(center);

        // Navigate to first month (April) of 5-month span (Apr-Aug)
        state.selected_date = NaiveDate::from_ymd_opt(2025, 4, 15).unwrap();

        assert!(state.needs_date_range_refresh());
    }

    #[test]
    fn test_needs_refresh_at_end_boundary() {
        let mut state = AppState::new();
        let center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.current_date_range = DateRange::five_month_span(center);

        // Navigate to last month (August) of 5-month span (Apr-Aug)
        state.selected_date = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();

        assert!(state.needs_date_range_refresh());
    }

    #[test]
    fn test_no_refresh_in_middle_months() {
        let mut state = AppState::new();
        let center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.current_date_range = DateRange::five_month_span(center);

        // Stay in middle month (May, June, July)
        state.selected_date = NaiveDate::from_ymd_opt(2025, 5, 15).unwrap();
        assert!(!state.needs_date_range_refresh());

        state.selected_date = NaiveDate::from_ymd_opt(2025, 6, 20).unwrap();
        assert!(!state.needs_date_range_refresh());

        state.selected_date = NaiveDate::from_ymd_opt(2025, 7, 10).unwrap();
        assert!(!state.needs_date_range_refresh());
    }

    #[test]
    fn test_update_date_range_changes_current_range() {
        let mut state = AppState::new();
        let old_center = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.current_date_range = DateRange::five_month_span(old_center);

        let new_center = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap();
        let new_range = DateRange::five_month_span(new_center);

        state.update_date_range(new_range.clone());

        assert_eq!(state.current_date_range.start, new_range.start);
        assert_eq!(state.current_date_range.end, new_range.end);
    }

    #[test]
    fn test_trim_events_to_25_month_span() {
        let mut state = AppState::new();
        let selected = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = selected;
        state.current_month = (2025, 6);

        // Add events spanning 30 months (too many)
        use crate::calendar::models::{Event, EventDateTime};
        for month_offset in -15i32..=14 {
            let date = selected
                .checked_add_signed(chrono::Duration::days(month_offset as i64 * 30))
                .unwrap();
            let event = Event {
                id: format!("event_{}", month_offset),
                summary: Some("Test".to_string()),
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
        }

        let initial_count = state.events.len();
        assert!(initial_count > 25); // We added 30 months worth

        state.trim_events_to_25_month_span();

        // Should be trimmed to approximately 25 months (may vary slightly due to month lengths)
        assert!(state.events.len() <= 26); // Allow small variance
        assert!(state.events.len() >= 24);
    }

    #[test]
    fn test_trim_preserves_current_month() {
        let mut state = AppState::new();
        // Set selected date far from current month
        state.selected_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        state.current_month = (2025, 6); // Current month is June 2025

        // Add events for current month (June 2025) - should be preserved
        use crate::calendar::models::{Event, EventDateTime};
        let current_month_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let event = Event {
            id: "current_month_event".to_string(),
            summary: Some("Current Month".to_string()),
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
        state.events.insert(current_month_date, vec![event]);

        // Add events centered on selected date (Jan 2024) - 25 months worth
        for month_offset in -12i32..=12 {
            let date = state
                .selected_date
                .checked_add_signed(chrono::Duration::days(month_offset as i64 * 30))
                .unwrap();
            if date.month() != current_month_date.month()
                || date.year() != current_month_date.year()
            {
                let event = Event {
                    id: format!("event_{}", month_offset),
                    summary: Some("Test".to_string()),
                    description: None,
                    location: None,
                    start: EventDateTime {
                        date_time: Some("2024-01-15T10:00:00Z".to_string()),
                        date: None,
                        time_zone: None,
                    },
                    end: EventDateTime {
                        date_time: Some("2024-01-15T11:00:00Z".to_string()),
                        date: None,
                        time_zone: None,
                    },
                    status: None,
                    html_link: None,
                    attendees: None,
                };
                state.events.insert(date, vec![event]);
            }
        }

        state.trim_events_to_25_month_span();

        // Current month event should still be there
        assert!(state.events.contains_key(&current_month_date));

        // Verify we have the event
        let events = state.events.get(&current_month_date).unwrap();
        assert_eq!(events[0].summary, Some("Current Month".to_string()));
    }
}
