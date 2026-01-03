use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Calendar {
    pub id: String,
    pub summary: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
    #[serde(rename = "accessRole")]
    pub access_role: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventDateTime,
    pub end: EventDateTime,
    pub status: Option<String>,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
    pub attendees: Option<Vec<Attendee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attendee {
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "responseStatus")]
    pub response_status: Option<String>,
    pub optional: Option<bool>,
}

// Private response wrappers for API responses
#[derive(Debug, Deserialize)]
pub(crate) struct CalendarListResponse {
    pub items: Vec<Calendar>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EventsListResponse {
    pub items: Vec<Event>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_deserialize_minimal() {
        let json = r#"{
            "id": "primary",
            "summary": "My Calendar",
            "timeZone": "America/New_York",
            "accessRole": "owner"
        }"#;

        let calendar: Calendar = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(calendar.id, "primary");
        assert_eq!(calendar.summary, "My Calendar");
        assert_eq!(calendar.primary, false); // default
        assert_eq!(calendar.time_zone, "America/New_York");
        assert_eq!(calendar.access_role, "owner");
        assert_eq!(calendar.background_color, None);
        assert_eq!(calendar.description, None);
    }

    #[test]
    fn test_calendar_deserialize_full() {
        let json = r##"{
            "id": "calendar123",
            "summary": "Work Calendar",
            "primary": true,
            "timeZone": "Europe/London",
            "accessRole": "writer",
            "backgroundColor": "#0088aa",
            "description": "My work events"
        }"##;

        let calendar: Calendar = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(calendar.id, "calendar123");
        assert_eq!(calendar.summary, "Work Calendar");
        assert_eq!(calendar.primary, true);
        assert_eq!(calendar.time_zone, "Europe/London");
        assert_eq!(calendar.access_role, "writer");
        assert_eq!(calendar.background_color, Some("#0088aa".to_string()));
        assert_eq!(calendar.description, Some("My work events".to_string()));
    }

    #[test]
    fn test_event_datetime_with_datetime() {
        let json = r#"{
            "dateTime": "2025-11-28T10:00:00-05:00",
            "timeZone": "America/New_York"
        }"#;

        let event_dt: EventDateTime = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(
            event_dt.date_time,
            Some("2025-11-28T10:00:00-05:00".to_string())
        );
        assert_eq!(event_dt.date, None);
        assert_eq!(event_dt.time_zone, Some("America/New_York".to_string()));
    }

    #[test]
    fn test_event_datetime_with_date_only() {
        let json = r#"{
            "date": "2025-11-28"
        }"#;

        let event_dt: EventDateTime = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(event_dt.date_time, None);
        assert_eq!(event_dt.date, Some("2025-11-28".to_string()));
        assert_eq!(event_dt.time_zone, None);
    }

    #[test]
    fn test_attendee_deserialize_minimal() {
        let json = r#"{
            "email": "user@example.com"
        }"#;

        let attendee: Attendee = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(attendee.email, "user@example.com");
        assert_eq!(attendee.display_name, None);
        assert_eq!(attendee.response_status, None);
        assert_eq!(attendee.optional, None);
    }

    #[test]
    fn test_attendee_deserialize_full() {
        let json = r#"{
            "email": "attendee@example.com",
            "displayName": "John Doe",
            "responseStatus": "accepted",
            "optional": true
        }"#;

        let attendee: Attendee = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(attendee.email, "attendee@example.com");
        assert_eq!(attendee.display_name, Some("John Doe".to_string()));
        assert_eq!(attendee.response_status, Some("accepted".to_string()));
        assert_eq!(attendee.optional, Some(true));
    }

    #[test]
    fn test_event_deserialize_minimal() {
        let json = r#"{
            "id": "event123",
            "start": {
                "dateTime": "2025-11-28T10:00:00-05:00"
            },
            "end": {
                "dateTime": "2025-11-28T11:00:00-05:00"
            }
        }"#;

        let event: Event = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(event.id, "event123");
        assert_eq!(event.summary, None);
        assert_eq!(event.description, None);
        assert_eq!(event.location, None);
        assert_eq!(
            event.start.date_time,
            Some("2025-11-28T10:00:00-05:00".to_string())
        );
        assert_eq!(
            event.end.date_time,
            Some("2025-11-28T11:00:00-05:00".to_string())
        );
        assert_eq!(event.status, None);
        assert_eq!(event.html_link, None);
        assert_eq!(event.attendees, None);
    }

    #[test]
    fn test_event_deserialize_full() {
        let json = r#"{
            "id": "event456",
            "summary": "Team Meeting",
            "description": "Quarterly review",
            "location": "Conference Room A",
            "start": {
                "dateTime": "2025-11-28T14:00:00-05:00",
                "timeZone": "America/New_York"
            },
            "end": {
                "dateTime": "2025-11-28T15:00:00-05:00",
                "timeZone": "America/New_York"
            },
            "status": "confirmed",
            "htmlLink": "https://calendar.google.com/event?eid=abc123",
            "attendees": [
                {
                    "email": "alice@example.com",
                    "displayName": "Alice Smith",
                    "responseStatus": "accepted"
                },
                {
                    "email": "bob@example.com",
                    "displayName": "Bob Jones",
                    "responseStatus": "tentative",
                    "optional": true
                }
            ]
        }"#;

        let event: Event = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(event.id, "event456");
        assert_eq!(event.summary, Some("Team Meeting".to_string()));
        assert_eq!(event.description, Some("Quarterly review".to_string()));
        assert_eq!(event.location, Some("Conference Room A".to_string()));
        assert_eq!(event.status, Some("confirmed".to_string()));
        assert_eq!(
            event.html_link,
            Some("https://calendar.google.com/event?eid=abc123".to_string())
        );

        let attendees = event.attendees.unwrap();
        assert_eq!(attendees.len(), 2);
        assert_eq!(attendees[0].email, "alice@example.com");
        assert_eq!(attendees[0].display_name, Some("Alice Smith".to_string()));
        assert_eq!(attendees[0].response_status, Some("accepted".to_string()));
        assert_eq!(attendees[1].email, "bob@example.com");
        assert_eq!(attendees[1].optional, Some(true));
    }

    #[test]
    fn test_calendar_list_response_deserialize_without_pagination() {
        let json = r#"{
            "items": [
                {
                    "id": "cal1",
                    "summary": "Calendar 1",
                    "timeZone": "UTC",
                    "accessRole": "owner"
                },
                {
                    "id": "cal2",
                    "summary": "Calendar 2",
                    "timeZone": "UTC",
                    "accessRole": "reader"
                }
            ]
        }"#;

        let response: CalendarListResponse =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].id, "cal1");
        assert_eq!(response.items[1].id, "cal2");
        assert_eq!(response.next_page_token, None);
    }

    #[test]
    fn test_calendar_list_response_deserialize_with_pagination() {
        let json = r#"{
            "items": [
                {
                    "id": "cal1",
                    "summary": "Calendar 1",
                    "timeZone": "UTC",
                    "accessRole": "owner"
                }
            ],
            "nextPageToken": "token123"
        }"#;

        let response: CalendarListResponse =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.next_page_token, Some("token123".to_string()));
    }

    #[test]
    fn test_events_list_response_deserialize_without_pagination() {
        let json = r#"{
            "items": [
                {
                    "id": "event1",
                    "summary": "Event 1",
                    "start": {
                        "dateTime": "2025-11-28T10:00:00Z"
                    },
                    "end": {
                        "dateTime": "2025-11-28T11:00:00Z"
                    }
                }
            ]
        }"#;

        let response: EventsListResponse =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].id, "event1");
        assert_eq!(response.next_page_token, None);
    }

    #[test]
    fn test_events_list_response_deserialize_with_pagination() {
        let json = r#"{
            "items": [
                {
                    "id": "event1",
                    "start": {
                        "date": "2025-11-28"
                    },
                    "end": {
                        "date": "2025-11-28"
                    }
                }
            ],
            "nextPageToken": "next_page_token_xyz"
        }"#;

        let response: EventsListResponse =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(response.items.len(), 1);
        assert_eq!(
            response.next_page_token,
            Some("next_page_token_xyz".to_string())
        );
    }
}
