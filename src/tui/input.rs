use crossterm::event::{KeyCode, KeyEvent};

use super::state::{AppState, EventsViewMode, ViewFocus};

pub enum InputAction {
    Quit,
    Refresh,
    None,
}

pub fn handle_key_event(key: KeyEvent, state: &mut AppState) -> InputAction {
    // Global keys that work regardless of focus
    match key.code {
        KeyCode::Char('q') => return InputAction::Quit,
        KeyCode::Char('r') => return InputAction::Refresh,
        KeyCode::Char('t') => {
            state.jump_to_today();
            return InputAction::None;
        }
        KeyCode::Tab => {
            state.toggle_focus();
            return InputAction::None;
        }
        _ => {}
    }

    // Focus-aware routing - NO SHARED LOGIC
    match state.view_focus {
        ViewFocus::Calendar => handle_calendar_input(key, state),
        ViewFocus::Events => handle_events_input(key, state),
    }
}

fn handle_calendar_input(key: KeyEvent, state: &mut AppState) -> InputAction {
    match key.code {
        KeyCode::Esc => InputAction::Quit,
        KeyCode::Left | KeyCode::Char('h') => {
            state.move_selected_date(-1);
            state.reset_event_selection();
            InputAction::None
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.move_selected_date(1);
            state.reset_event_selection();
            InputAction::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_to_prev_week();
            state.reset_event_selection();
            InputAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_to_next_week();
            state.reset_event_selection();
            InputAction::None
        }
        _ => InputAction::None,
    }
}

fn handle_events_input(key: KeyEvent, state: &mut AppState) -> InputAction {
    match state.events_view_mode {
        EventsViewMode::List => handle_events_list_input(key, state),
        EventsViewMode::Details { .. } => handle_events_details_input(key, state),
    }
}

fn handle_events_list_input(key: KeyEvent, state: &mut AppState) -> InputAction {
    match key.code {
        KeyCode::Esc => InputAction::Quit,
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_event_selection_up();
            InputAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_event_selection_down();
            InputAction::None
        }
        KeyCode::Enter => {
            state.select_event();
            InputAction::None
        }
        _ => InputAction::None,
    }
}

fn handle_events_details_input(key: KeyEvent, state: &mut AppState) -> InputAction {
    match key.code {
        KeyCode::Esc => {
            state.exit_event_details();
            InputAction::None
        }
        _ => InputAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use chrono::NaiveDate;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_quit_actions() {
        let mut state = AppState::new();

        let action = handle_key_event(create_key_event(KeyCode::Char('q')), &mut state);
        assert!(matches!(action, InputAction::Quit));

        let action = handle_key_event(create_key_event(KeyCode::Esc), &mut state);
        assert!(matches!(action, InputAction::Quit));
    }

    #[test]
    fn test_refresh_action() {
        let mut state = AppState::new();

        let action = handle_key_event(create_key_event(KeyCode::Char('r')), &mut state);
        assert!(matches!(action, InputAction::Refresh));
    }

    #[test]
    fn test_navigation_left_right() {
        let mut state = AppState::new();
        state.view_focus = ViewFocus::Calendar; // Explicitly set Calendar focus
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        // Move right
        handle_key_event(create_key_event(KeyCode::Right), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 16).unwrap());

        // Move left
        handle_key_event(create_key_event(KeyCode::Left), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());

        // Test vim-style keys
        handle_key_event(create_key_event(KeyCode::Char('l')), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 16).unwrap());

        handle_key_event(create_key_event(KeyCode::Char('h')), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    #[test]
    fn test_navigation_up_down() {
        let mut state = AppState::new();
        state.view_focus = ViewFocus::Calendar; // Explicitly set Calendar focus
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        // Move down (next week)
        handle_key_event(create_key_event(KeyCode::Down), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 22).unwrap());

        // Move up (prev week)
        handle_key_event(create_key_event(KeyCode::Up), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());

        // Test vim-style keys
        handle_key_event(create_key_event(KeyCode::Char('j')), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 22).unwrap());

        handle_key_event(create_key_event(KeyCode::Char('k')), &mut state);
        assert_eq!(state.selected_date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    #[test]
    fn test_toggle_focus() {
        use crate::tui::state::ViewFocus;

        let mut state = AppState::new();
        assert_eq!(state.view_focus, ViewFocus::Calendar);

        handle_key_event(create_key_event(KeyCode::Tab), &mut state);
        assert_eq!(state.view_focus, ViewFocus::Events);

        handle_key_event(create_key_event(KeyCode::Tab), &mut state);
        assert_eq!(state.view_focus, ViewFocus::Calendar);
    }

    #[test]
    fn test_t_key_jumps_to_today() {
        let mut state = AppState::new();
        let away_date = NaiveDate::from_ymd_opt(2025, 12, 25).unwrap();
        state.selected_date = away_date;
        assert_ne!(state.selected_date, state.today);

        let action = handle_key_event(create_key_event(KeyCode::Char('t')), &mut state);

        assert!(matches!(action, InputAction::None));
        assert_eq!(state.selected_date, state.today);
    }

    #[test]
    fn test_t_key_when_already_on_today() {
        let mut state = AppState::new();
        assert_eq!(state.selected_date, state.today);

        handle_key_event(create_key_event(KeyCode::Char('t')), &mut state);

        assert_eq!(state.selected_date, state.today);
    }

    #[test]
    fn test_calendar_keys_only_work_when_calendar_focused() {
        let mut state = AppState::new();
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;
        state.view_focus = ViewFocus::Events; // Focus on Events

        // Arrow keys should NOT affect calendar when Events focused
        handle_key_event(create_key_event(KeyCode::Right), &mut state);
        assert_eq!(state.selected_date, initial_date); // Date unchanged

        handle_key_event(create_key_event(KeyCode::Down), &mut state);
        assert_eq!(state.selected_date, initial_date); // Date unchanged
    }

    #[test]
    fn test_events_keys_only_work_when_events_focused() {
        use crate::calendar::models::{Event, EventDateTime};

        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;
        state.view_focus = ViewFocus::Calendar; // Focus on Calendar

        // Add events to test selection
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
        ];
        state.events.insert(date, events);

        // Up/Down should NOT affect event selection when Calendar focused
        handle_key_event(create_key_event(KeyCode::Down), &mut state);
        assert_eq!(state.selected_event_index, None); // Selection unchanged

        handle_key_event(create_key_event(KeyCode::Up), &mut state);
        assert_eq!(state.selected_event_index, None); // Selection unchanged
    }

    #[test]
    fn test_enter_opens_details() {
        use crate::calendar::models::{Event, EventDateTime};

        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;
        state.view_focus = ViewFocus::Events;

        // Add an event
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
        ];
        state.events.insert(date, events);

        // Select an event first
        state.move_event_selection_down();
        assert_eq!(state.selected_event_index, Some(0));
        assert!(matches!(state.events_view_mode, EventsViewMode::List));

        // Press Enter to open details
        handle_key_event(create_key_event(KeyCode::Enter), &mut state);
        assert!(matches!(
            state.events_view_mode,
            EventsViewMode::Details { event_index: 0 }
        ));
    }

    #[test]
    fn test_esc_closes_details() {
        let mut state = AppState::new();
        state.view_focus = ViewFocus::Events;
        state.events_view_mode = EventsViewMode::Details { event_index: 0 };

        let action = handle_key_event(create_key_event(KeyCode::Esc), &mut state);

        assert!(matches!(action, InputAction::None)); // Doesn't quit
        assert!(matches!(state.events_view_mode, EventsViewMode::List)); // Back to list
    }

    #[test]
    fn test_tab_toggles_focus_from_any_mode() {
        let mut state = AppState::new();

        // From Calendar
        state.view_focus = ViewFocus::Calendar;
        handle_key_event(create_key_event(KeyCode::Tab), &mut state);
        assert_eq!(state.view_focus, ViewFocus::Events);

        // From Events List
        handle_key_event(create_key_event(KeyCode::Tab), &mut state);
        assert_eq!(state.view_focus, ViewFocus::Calendar);

        // From Events Details
        state.view_focus = ViewFocus::Events;
        state.events_view_mode = EventsViewMode::Details { event_index: 0 };
        handle_key_event(create_key_event(KeyCode::Tab), &mut state);
        assert_eq!(state.view_focus, ViewFocus::Calendar);
    }

    #[test]
    fn test_global_keys_work_regardless_of_focus() {
        let mut state = AppState::new();
        let initial_date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = initial_date;

        // Test 't' key from Calendar focus
        state.view_focus = ViewFocus::Calendar;
        state.selected_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        handle_key_event(create_key_event(KeyCode::Char('t')), &mut state);
        assert_eq!(state.selected_date, state.today);

        // Test 't' key from Events focus
        state.view_focus = ViewFocus::Events;
        state.selected_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        handle_key_event(create_key_event(KeyCode::Char('t')), &mut state);
        assert_eq!(state.selected_date, state.today);

        // Test 'q' key from Calendar
        state.view_focus = ViewFocus::Calendar;
        let action = handle_key_event(create_key_event(KeyCode::Char('q')), &mut state);
        assert!(matches!(action, InputAction::Quit));

        // Test 'q' key from Events
        state.view_focus = ViewFocus::Events;
        let action = handle_key_event(create_key_event(KeyCode::Char('q')), &mut state);
        assert!(matches!(action, InputAction::Quit));
    }

    #[test]
    fn test_date_change_resets_selection() {
        use crate::calendar::models::{Event, EventDateTime};

        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = date;
        state.view_focus = ViewFocus::Calendar;

        // Add an event and select it
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
        ];
        state.events.insert(date, events);
        state.selected_event_index = Some(0);
        state.events_view_mode = EventsViewMode::Details { event_index: 0 };

        // Change date with arrow key
        handle_key_event(create_key_event(KeyCode::Right), &mut state);

        // Selection should be reset
        assert_eq!(state.selected_event_index, None);
        assert!(matches!(state.events_view_mode, EventsViewMode::List));
    }

    #[test]
    fn test_esc_quits_from_list_mode() {
        let mut state = AppState::new();
        state.view_focus = ViewFocus::Events;
        state.events_view_mode = EventsViewMode::List;

        let action = handle_key_event(create_key_event(KeyCode::Esc), &mut state);

        assert!(matches!(action, InputAction::Quit));
    }
}
