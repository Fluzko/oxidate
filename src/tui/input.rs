use crossterm::event::{KeyCode, KeyEvent};

use super::state::AppState;

pub enum InputAction {
    Quit,
    Refresh,
    None,
}

pub fn handle_key_event(key: KeyEvent, state: &mut AppState) -> InputAction {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => InputAction::Quit,
        KeyCode::Char('r') => InputAction::Refresh,
        KeyCode::Char('t') => {
            state.jump_to_today();
            InputAction::None
        }
        KeyCode::Tab => {
            state.toggle_focus();
            InputAction::None
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.move_selected_date(-1);
            InputAction::None
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.move_selected_date(1);
            InputAction::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_to_prev_week();
            InputAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_to_next_week();
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
}
