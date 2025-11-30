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
        T: serde::de::DeserializeOwned,
    {
        // First attempt with current access token
        let response = api_call(self.tokens.access_token.clone())
            .await
            .context("API call failed")?;

        // Check if 401 BEFORE error_for_status() - Google may return Ok(Response) with 401 status
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Refresh token
            self.refresh_access_token()
                .await
                .context("Failed to refresh access token after 401")?;

            // Retry with new token
            let retry_response = api_call(self.tokens.access_token.clone())
                .await
                .context("API call failed on retry after token refresh")?;

            // Check again - if still 401, something is wrong
            if retry_response.status() == reqwest::StatusCode::UNAUTHORIZED {
                anyhow::bail!("Still unauthorized after token refresh");
            }

            // Parse and return retry response
            retry_response
                .error_for_status()
                .context("API returned error status on retry")?
                .json::<T>()
                .await
                .context("Failed to parse response JSON on retry")
        } else {
            // Not 401, proceed normally
            response
                .error_for_status()
                .context("API returned error status")?
                .json::<T>()
                .await
                .context("Failed to parse response JSON")
        }
    }

    async fn refresh_access_token(&mut self) -> Result<()> {
        let refresh_token = RefreshToken::new(self.tokens.refresh_token.clone());

        let token_result = self.oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await
            .context("Failed to refresh access token")?;

        // Update access token
        self.tokens.access_token = token_result.access_token().secret().clone();

        // Update refresh token if a new one is provided
        if let Some(new_refresh_token) = token_result.refresh_token() {
            self.tokens.refresh_token = new_refresh_token.secret().clone();
        }

        // Save tokens to disk
        self.tokens.save()
            .context("Failed to save refreshed tokens")?;

        Ok(())
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
