use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

impl Tokens {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }

    fn get_storage_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;

        let app_config_dir = config_dir.join("ai-rust-calendar");
        Ok(app_config_dir.join("tokens.json"))
    }

    pub fn exists() -> bool {
        match Self::get_storage_path() {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    pub fn save(&self) -> Result<()> {
        let tokens_path = Self::get_storage_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = tokens_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize tokens")?;

        fs::write(&tokens_path, json)
            .context("Failed to write tokens file")?;

        Ok(())
    }

    pub fn load() -> Result<Self> {
        let tokens_path = Self::get_storage_path()?;

        let json = fs::read_to_string(&tokens_path)
            .context("Failed to read tokens file")?;

        let tokens: Self = serde_json::from_str(&json)
            .context("Failed to deserialize tokens")?;

        Ok(tokens)
    }

    pub fn delete() -> Result<()> {
        let tokens_path = Self::get_storage_path()?;

        if tokens_path.exists() {
            fs::remove_file(&tokens_path)
                .context("Failed to delete tokens file")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_new() {
        let tokens = Tokens::new(
            "access123".to_string(),
            "refresh456".to_string()
        );

        assert_eq!(tokens.access_token, "access123");
        assert_eq!(tokens.refresh_token, "refresh456");
    }

    #[test]
    fn test_tokens_exist_returns_false_when_file_does_not_exist() {
        // Clean up any existing tokens first
        Tokens::delete().ok();

        // Should return false when tokens don't exist
        assert!(!Tokens::exists());
    }

    #[test]
    fn test_save_and_load_tokens() {
        // Clean up first
        Tokens::delete().ok();

        let original_tokens = Tokens::new(
            "test_access_token".to_string(),
            "test_refresh_token".to_string()
        );

        // Save tokens
        original_tokens.save().expect("Failed to save tokens");

        // Verify tokens exist
        assert!(Tokens::exists());

        // Load tokens
        let loaded_tokens = Tokens::load().expect("Failed to load tokens");

        // Verify they match
        assert_eq!(original_tokens, loaded_tokens);

        // Cleanup
        Tokens::delete().ok();
    }

    #[test]
    fn test_delete_tokens() {
        // Clean up first
        Tokens::delete().ok();

        // Create tokens
        let tokens = Tokens::new(
            "test_access".to_string(),
            "test_refresh".to_string()
        );

        // Save tokens
        tokens.save().expect("Failed to save tokens");
        assert!(Tokens::exists());

        // Delete tokens
        Tokens::delete().expect("Failed to delete tokens");

        // Verify they're gone
        assert!(!Tokens::exists());
    }

    #[test]
    fn test_delete_tokens_when_file_does_not_exist() {
        // Clean up first
        Tokens::delete().ok();

        // Should not error even if file doesn't exist
        let result = Tokens::delete();
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_tokens_fails_when_file_does_not_exist() {
        // Clean up first
        Tokens::delete().ok();

        let result = Tokens::load();
        assert!(result.is_err());
    }
}
