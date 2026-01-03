use chrono::NaiveDate;
use std::collections::HashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use super::fetcher::fetch_calendar_data;
use super::state::DateRange;
use crate::calendar::client::CalendarClient;
use crate::calendar::models::{Calendar, Event};

#[derive(Debug)]
pub enum DataMessage {
    Loading,
    Success {
        calendars: Vec<Calendar>,
        events: HashMap<NaiveDate, Vec<Event>>,
        client: CalendarClient,
    },
    Error {
        error: String,
        client: CalendarClient,
    },
}

pub struct DataLoader {
    receiver: UnboundedReceiver<DataMessage>,
}

impl DataLoader {
    pub fn new(mut client: CalendarClient, date_range: DateRange) -> Self {
        let (sender, receiver) = unbounded_channel();

        // Send initial loading message
        sender
            .send(DataMessage::Loading)
            .expect("Failed to send loading message");

        // Spawn async task using existing tokio runtime
        tokio::spawn(async move {
            // Run the async fetch operation
            let result = fetch_calendar_data(&mut client, date_range).await;

            // Send result through channel
            match result {
                Ok((calendars, events)) => {
                    let _ = sender.send(DataMessage::Success {
                        calendars,
                        events,
                        client,
                    });
                }
                Err(e) => {
                    let _ = sender.send(DataMessage::Error {
                        error: e.to_string(),
                        client,
                    });
                }
            }
        });

        Self { receiver }
    }

    pub fn try_recv(&mut self) -> Option<DataMessage> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_message_variants() {
        // Test that DataMessage variants can be created
        let loading = DataMessage::Loading;
        assert!(matches!(loading, DataMessage::Loading));

        // Note: We can't easily create CalendarClient instances in tests without OAuth setup,
        // so Success and Error variants are tested via integration tests
    }

    #[test]
    fn test_channel_communication() {
        let (sender, mut receiver) = unbounded_channel();

        // Send a message
        sender.send(DataMessage::Loading).unwrap();

        // Receive the message
        let msg = receiver.try_recv().ok();
        assert!(msg.is_some());
        assert!(matches!(msg.unwrap(), DataMessage::Loading));
    }
}
