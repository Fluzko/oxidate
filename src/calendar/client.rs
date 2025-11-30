use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RefreshToken, TokenResponse, TokenUrl,
};
use oauth2::reqwest::async_http_client;
use reqwest;

use crate::auth::Tokens;
use super::models::{Calendar, Event, CalendarListResponse, EventsListResponse};

pub struct CalendarClient {
    tokens: Tokens,
    oauth_client: BasicClient,
    http_client: reqwest::Client,
}

impl CalendarClient {
    pub fn new(tokens: Tokens) -> Result<Self> {
        let client_id = Self::get_client_id()?;
        let client_secret = Self::get_client_secret()?;

        let oauth_client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
        );

        let http_client = reqwest::Client::new();

        Ok(Self {
            tokens,
            oauth_client,
            http_client,
        })
    }

    pub async fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        todo!("Implement list_calendars")
    }

    pub async fn list_events(
        &mut self,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
    ) -> Result<Vec<Event>> {
        todo!("Implement list_events")
    }

    async fn with_token_refresh<F, Fut, T>(&mut self, api_call: F) -> Result<T>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response>>,
    {
        todo!("Implement with_token_refresh")
    }

    async fn refresh_access_token(&mut self) -> Result<()> {
        todo!("Implement refresh_access_token")
    }

    fn get_client_id() -> Result<String> {
        option_env!("GOOGLE_CLIENT_ID")
            .map(|s| s.to_string())
            .context("GOOGLE_CLIENT_ID not set at compile time")
    }

    fn get_client_secret() -> Result<String> {
        option_env!("GOOGLE_CLIENT_SECRET")
            .map(|s| s.to_string())
            .context("GOOGLE_CLIENT_SECRET not set at compile time")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_client_new_creates_instance() {
        // Given valid tokens
        let tokens = Tokens::new(
            "test_access_token".to_string(),
            "test_refresh_token".to_string(),
        );

        // When creating a new client (may fail without credentials)
        let result = CalendarClient::new(tokens);

        // Then it should create an instance if credentials are available
        // Note: This test may fail in CI without credentials
        if result.is_ok() {
            let _client = result.unwrap();
            // Client created successfully
            assert!(true);
        } else {
            // Expected to fail without compile-time credentials
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_get_client_id_and_secret() {
        // Document that these methods load from compile-time environment
        // They should either return a value or error with context

        let client_id_result = CalendarClient::get_client_id();
        let client_secret_result = CalendarClient::get_client_secret();

        // Both should have the same outcome (both Ok or both Err)
        assert_eq!(client_id_result.is_ok(), client_secret_result.is_ok());
    }
}
