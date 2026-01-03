pub mod oauth;
pub mod port;
pub mod tokens;

pub use oauth::OAuthClient;
pub use tokens::Tokens;

use anyhow::Result;

/// Main authentication workflow
/// Checks if tokens exist, if not runs OAuth flow
pub async fn authenticate() -> Result<Tokens> {
    if Tokens::exists() {
        println!("Loading existing credentials...");
        Tokens::load()
    } else {
        println!("No credentials found. Starting OAuth flow...");
        let oauth_client = OAuthClient::new()?;
        let tokens = oauth_client.run_flow().await?;
        tokens.save()?;
        println!("Credentials saved successfully!");
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_module_accessible() {
        let tokens = Tokens::new("test".to_string(), "test".to_string());
        assert_eq!(tokens.access_token, "test");
    }
}
