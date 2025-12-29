use chrono::{Datelike, NaiveDate, Weekday};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Widget},
};

use crate::tui::state::{AppState, ViewFocus};

pub struct CalendarWidget<'a> {
    state: &'a AppState,
}

impl<'a> CalendarWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn get_days_in_month(year: i32, month: u32) -> u32 {
        // Last day of current month
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        first_of_next.pred_opt().unwrap().day()
    }

    fn get_first_weekday(year: i32, month: u32) -> Weekday {
        NaiveDate::from_ymd_opt(year, month, 1).unwrap().weekday()
    }

    fn weekday_to_offset(weekday: Weekday) -> u32 {
        match weekday {
            Weekday::Sun => 0,
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
        }
    }
}

impl<'a> Widget for CalendarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_date = self.state.selected_date;
        let year = selected_date.year();
        let month = selected_date.month();

        // Create border with focus indicator
        let border_style = if self.state.view_focus == ViewFocus::Calendar {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} {} ", month_name(month), year));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 12 || inner.width < 28 {
            // Not enough space to render calendar
            return;
        }

        // Render day names header with larger spacing
        let day_names = vec!["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut x = inner.x;
        let y = inner.y;

        for day_name in day_names {
            let span = Span::styled(day_name, Style::default().add_modifier(Modifier::BOLD));
            buf.set_span(x, y, &span, 4);
            x += 4;
        }

        // Calculate calendar grid
        let days_in_month = Self::get_days_in_month(year, month);
        let first_weekday = Self::get_first_weekday(year, month);
        let offset = Self::weekday_to_offset(first_weekday);

        // Render dates
        let mut day = 1;

        for week_row in 0..6 {
            if day > days_in_month {
                break;
            }

            for col in 0..7 {
                if week_row == 0 && col < offset {
                    // Empty cell before first day
                    continue;
                }

                if day > days_in_month {
                    break;
                }

                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                let x_pos = inner.x + (col * 4) as u16;
                let y_pos = inner.y + 3 + (week_row * 2) as u16;

                // Determine style
                let mut style = Style::default();
                let is_today = date == self.state.today;
                let is_selected = date == selected_date;

                // Priority 1: Both today AND selected
                if is_today && is_selected {
                    style = style.bg(Color::Cyan).fg(Color::White).add_modifier(Modifier::BOLD);
                }
                // Priority 2: Selected but not today
                else if is_selected {
                    style = style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
                }
                // Priority 3: Today but not selected
                else if is_today {
                    style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
                }
                // Priority 4: Has events
                else if self.state.has_events(date) {
                    style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                }

                let day_str = format!("{:3}", day);
                let span = Span::styled(day_str, style);
                buf.set_span(x_pos, y_pos, &span, 3);

                day += 1;
            }
        }
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_get_days_in_month() {
        assert_eq!(CalendarWidget::get_days_in_month(2025, 1), 31);
        assert_eq!(CalendarWidget::get_days_in_month(2025, 2), 28);
        assert_eq!(CalendarWidget::get_days_in_month(2024, 2), 29); // Leap year
        assert_eq!(CalendarWidget::get_days_in_month(2025, 4), 30);
        assert_eq!(CalendarWidget::get_days_in_month(2025, 12), 31);
    }

    #[test]
    fn test_get_first_weekday() {
        // June 1, 2025 is a Sunday
        let weekday = CalendarWidget::get_first_weekday(2025, 6);
        assert_eq!(weekday, Weekday::Sun);

        // January 1, 2025 is a Wednesday
        let weekday = CalendarWidget::get_first_weekday(2025, 1);
        assert_eq!(weekday, Weekday::Wed);
    }

    #[test]
    fn test_weekday_to_offset() {
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Sun), 0);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Mon), 1);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Tue), 2);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Wed), 3);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Thu), 4);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Fri), 5);
        assert_eq!(CalendarWidget::weekday_to_offset(Weekday::Sat), 6);
    }

    #[test]
    fn test_month_name() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(6), "June");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(13), "Unknown");
    }

    #[test]
    fn test_calendar_widget_new() {
        let state = AppState::new();
        let widget = CalendarWidget::new(&state);
        assert_eq!(widget.state.selected_date, Local::now().date_naive());
    }

    #[test]
    fn test_calendar_widget_uses_today_from_state() {
        let mut state = AppState::new();
        state.today = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        state.selected_date = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();

        let widget = CalendarWidget::new(&state);

        assert_eq!(widget.state.today, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }
}
