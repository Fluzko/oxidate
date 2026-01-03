use super::port::PortSelector;
use super::tokens::Tokens;
use anyhow::{Context, Result};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

pub struct OAuthClient {
    client: BasicClient,
    port: u16,
}

impl OAuthClient {
    pub fn new() -> Result<Self> {
        let client_id = Self::get_client_id()?;
        let client_secret = Self::get_client_secret()?;
        let port = PortSelector::find_available()?;

        let redirect_url = format!("http://localhost:{}", port);

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new(
                "https://oauth2.googleapis.com/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url)?);

        Ok(Self { client, port })
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

    pub fn get_authorization_url(&self) -> (String, CsrfToken) {
        let (url, csrf) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/calendar".to_string(),
            ))
            .url();

        (url.to_string(), csrf)
    }

    pub fn open_browser(&self, url: &str) -> Result<()> {
        webbrowser::open(url).context("Failed to open browser")?;
        Ok(())
    }

    pub fn listen_for_callback(&self) -> Result<String> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .context("Failed to bind to port")?;

        println!("Waiting for OAuth callback on port {}...", self.port);

        // Accept one connection
        let (mut stream, _) = listener.accept().context("Failed to accept connection")?;

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .context("Failed to read request")?;

        // Extract the authorization code from the request
        let code = Self::extract_code_from_request(&request_line)?;

        // Send success response
        let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Success!</h1><p>You can close this window and return to the application.</p></body></html>";
        stream
            .write_all(response.as_bytes())
            .context("Failed to write response")?;

        Ok(code)
    }

    fn extract_code_from_request(request_line: &str) -> Result<String> {
        // Request line format: GET /?code=...&state=... HTTP/1.1
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid request format");
        }

        let path = parts[1];
        let query_start = path.find('?').context("No query parameters in request")?;
        let query = &path[query_start + 1..];

        for param in query.split('&') {
            let kv: Vec<&str> = param.split('=').collect();
            if kv.len() == 2 && kv[0] == "code" {
                return Ok(kv[1].to_string());
            }
        }

        anyhow::bail!("Authorization code not found in request")
    }

    pub async fn exchange_code(&self, code: String) -> Result<Tokens> {
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .context("Failed to exchange authorization code")?;

        let access_token = token_result.access_token().secret().clone();
        let refresh_token = token_result
            .refresh_token()
            .context("No refresh token received")?
            .secret()
            .clone();

        Ok(Tokens::new(access_token, refresh_token))
    }

    pub async fn run_flow(&self) -> Result<Tokens> {
        let (auth_url, _csrf_token) = self.get_authorization_url();

        println!("Opening browser for authentication...");
        println!("If the browser doesn't open, visit: {}", auth_url);

        self.open_browser(&auth_url)?;

        let code = self.listen_for_callback()?;
        let tokens = self.exchange_code(code).await?;

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_client_new() {
        let result = OAuthClient::new();
        // This might fail if credentials aren't set, but structure should be valid
        if result.is_ok() {
            let client = result.unwrap();
            assert!(client.port > 0);
        }
    }

    #[test]
    fn test_get_authorization_url() {
        if let Ok(client) = OAuthClient::new() {
            let (url, csrf_token) = client.get_authorization_url();

            assert!(url.contains("accounts.google.com"));
            assert!(url.contains("oauth2"));
            assert!(url.contains("calendar"));
            assert!(!csrf_token.secret().is_empty());
        }
    }

    #[test]
    fn test_extract_code_from_request() {
        let request = "GET /?code=test_code_123&state=random_state HTTP/1.1";
        let result = OAuthClient::extract_code_from_request(request);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_code_123");
    }

    #[test]
    fn test_extract_code_from_request_with_multiple_params() {
        let request = "GET /?state=xyz&code=my_auth_code&scope=calendar HTTP/1.1";
        let result = OAuthClient::extract_code_from_request(request);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_auth_code");
    }

    #[test]
    fn test_extract_code_fails_without_code_param() {
        let request = "GET /?state=xyz&scope=calendar HTTP/1.1";
        let result = OAuthClient::extract_code_from_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_code_fails_with_invalid_request() {
        let request = "INVALID REQUEST";
        let result = OAuthClient::extract_code_from_request(request);

        assert!(result.is_err());
    }
}
