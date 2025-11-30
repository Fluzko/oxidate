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

    // Test-only methods that accept custom paths
    #[cfg(test)]
    fn save_to(&self, path: &std::path::Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize tokens")?;

        fs::write(path, json)
            .context("Failed to write tokens file")?;

        Ok(())
    }

    #[cfg(test)]
    fn load_from(path: &std::path::Path) -> Result<Self> {
        let json = fs::read_to_string(path)
            .context("Failed to read tokens file")?;

        let tokens: Self = serde_json::from_str(&json)
            .context("Failed to deserialize tokens")?;

        Ok(tokens)
    }

    #[cfg(test)]
    fn delete_at(path: &std::path::Path) -> Result<()> {
        if path.exists() {
            fs::remove_file(path)
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
    fn test_save_and_load_tokens() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let token_path = temp_dir.path().join("tokens.json");

        let original_tokens = Tokens::new(
            "test_access_token".to_string(),
            "test_refresh_token".to_string()
        );

        // Save tokens
        original_tokens.save_to(&token_path).expect("Failed to save tokens");

        // Verify file exists
        assert!(token_path.exists());

        // Load tokens
        let loaded_tokens = Tokens::load_from(&token_path).expect("Failed to load tokens");

        // Verify they match
        assert_eq!(original_tokens, loaded_tokens);

        // temp_dir is automatically cleaned up when dropped
    }

    #[test]
    fn test_delete_tokens() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let token_path = temp_dir.path().join("tokens.json");

        // Create and save tokens
        let tokens = Tokens::new(
            "test_access".to_string(),
            "test_refresh".to_string()
        );

        tokens.save_to(&token_path).expect("Failed to save tokens");
        assert!(token_path.exists());

        // Delete tokens
        Tokens::delete_at(&token_path).expect("Failed to delete tokens");

        // Verify they're gone
        assert!(!token_path.exists());
    }

    #[test]
    fn test_delete_tokens_when_file_does_not_exist() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let token_path = temp_dir.path().join("tokens.json");

        // Should not error even if file doesn't exist
        let result = Tokens::delete_at(&token_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_tokens_fails_when_file_does_not_exist() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let token_path = temp_dir.path().join("tokens.json");

        let result = Tokens::load_from(&token_path);
        assert!(result.is_err());
    }

    // Integration test using real storage path
    #[test]
    fn test_tokens_exist_returns_false_when_file_does_not_exist() {
        // Clean up any existing tokens first
        Tokens::delete().ok();

        // Should return false when tokens don't exist
        assert!(!Tokens::exists());
    }

    #[test]
    fn test_real_path_integration() {
        // Clean up first
        Tokens::delete().ok();

        let original_tokens = Tokens::new(
            "integration_access".to_string(),
            "integration_refresh".to_string()
        );

        // Save using real path
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
}
