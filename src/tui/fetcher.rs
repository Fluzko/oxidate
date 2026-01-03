use anyhow::{Context, Result};
#[allow(unused_imports)]
use chrono::Timelike;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use std::collections::HashMap;

use super::state::DateRange;
use crate::calendar::client::CalendarClient;
use crate::calendar::models::{Calendar, Event};

pub async fn fetch_calendar_data(
    client: &mut CalendarClient,
    date_range: DateRange,
) -> Result<(Vec<Calendar>, HashMap<NaiveDate, Vec<Event>>)> {
    // Fetch all calendars
    let calendars = client
        .list_calendars()
        .await
        .context("Failed to fetch calendars")?;

    // Convert date range to DateTime<Utc>
    let time_min = date_to_utc(date_range.start);
    let time_max = date_to_utc(date_range.end);

    // Fetch events from all calendars
    let mut all_events_by_date: HashMap<NaiveDate, Vec<Event>> = HashMap::new();

    for calendar in &calendars {
        match client.list_events(&calendar.id, time_min, time_max).await {
            Ok(events) => {
                // Group events by date
                for event in events {
                    if let Some(date) = extract_date_from_event(&event) {
                        all_events_by_date.entry(date).or_default().push(event);
                    }
                }
            }
            Err(_) => {
                // TODO: Log error
            }
        }
    }

    Ok((calendars, all_events_by_date))
}

fn date_to_utc(date: NaiveDate) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .expect("Invalid date")
}

fn extract_date_from_event(event: &Event) -> Option<NaiveDate> {
    // Try to extract date from event start time
    if let Some(ref date_time_str) = event.start.date_time {
        // Parse RFC3339 format
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_time_str) {
            return Some(dt.date_naive());
        }
    }

    // Try all-day event (date field)
    if let Some(ref date_str) = event.start.date {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(date);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::models::EventDateTime;

    #[test]
    fn test_date_to_utc() {
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let utc = date_to_utc(date);

        assert_eq!(utc.year(), 2025);
        assert_eq!(utc.month(), 6);
        assert_eq!(utc.day(), 15);
        assert_eq!(utc.hour(), 0);
        assert_eq!(utc.minute(), 0);
        assert_eq!(utc.second(), 0);
    }

    #[test]
    fn test_extract_date_from_event_with_datetime() {
        let event = Event {
            id: "test".to_string(),
            summary: Some("Test Event".to_string()),
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

        let date = extract_date_from_event(&event);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()));
    }

    #[test]
    fn test_extract_date_from_event_with_date_only() {
        let event = Event {
            id: "test".to_string(),
            summary: Some("All-day Event".to_string()),
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

        let date = extract_date_from_event(&event);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()));
    }

    #[test]
    fn test_extract_date_from_event_with_invalid_format() {
        let event = Event {
            id: "test".to_string(),
            summary: Some("Invalid Event".to_string()),
            description: None,
            location: None,
            start: EventDateTime {
                date_time: Some("invalid_date".to_string()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some("invalid_date".to_string()),
                date: None,
                time_zone: None,
            },
            status: None,
            html_link: None,
            attendees: None,
        };

        let date = extract_date_from_event(&event);
        assert_eq!(date, None);
    }
}
