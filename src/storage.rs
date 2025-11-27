use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
}

impl TokenInfo {
    pub fn new(access_token: String,
               refresh_token: String,
               expires_at: DateTime<Utc>,
               refresh_token_expires_at: DateTime<Utc>) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_at,
            refresh_token_expires_at,
        }
    }
}

pub struct TokenStorage {
    token: Option<TokenInfo>,
    storage_path: PathBuf,
}

impl TokenStorage {
    const DEFAULT_STORAGE_FILE: &'static str = "tiktok_tokens.json";
    pub fn new() -> Self {
        Self::with_path(Self::DEFAULT_STORAGE_FILE)
    }
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        let storage_path = PathBuf::from(path.as_ref());
        let token = Self::load_from_file(&storage_path).ok();

        Self {
            token,
            storage_path,
        }
    }

    fn load_from_file(path: &Path) -> Result<TokenInfo, AppError> {
        if !path.exists() {
            return Err(AppError::ConfigError("Token file not found".to_string()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read token file: {}", e)))?;

        let token_info: TokenInfo = serde_json::from_str(&content)
            .map_err(|e| AppError::ParseError(format!("Failed to parse token file: {}", e)))?;

        info!("Loaded token from file: {}", path.display());
        Ok(token_info)
    }

    /// Save token to file
    fn save_to_file(&self, token_info: &TokenInfo) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(token_info)
            .map_err(|e| AppError::ParseError(format!("Failed to serialize token: {}", e)))?;

        fs::write(&self.storage_path, json).map_err(|e| {
            AppError::ConfigError(format!(
                "Failed to write token file {}: {}",
                self.storage_path.display(),
                e
            ))
        })?;

        info!("Saved token to file: {}", self.storage_path.display());
        Ok(())
    }

    /// Store token information and persist to disk
    pub fn store(&mut self, token_info: TokenInfo) -> Result<(), AppError> {
        self.save_to_file(&token_info)?;
        self.token = Some(token_info);
        Ok(())
    }

    /// Get the stored token
    pub fn get(&self) -> Option<&TokenInfo> {
        self.token.as_ref()
    }

    /// Clear the stored token and delete the file
    pub fn clear(&mut self) -> Result<(), AppError> {
        self.token = None;

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path).map_err(|e| {
                AppError::ConfigError(format!(
                    "Failed to delete token file {}: {}",
                    self.storage_path.display(),
                    e
                ))
            })?;
            info!("Deleted token file: {}", self.storage_path.display());
        }

        Ok(())
    }

    /// Check if access token is valid (not expired)
    pub fn is_access_token_valid(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| t.expires_at > Utc::now())
            .unwrap_or(false)
    }

    /// Check if refresh token is valid (not expired)
    pub fn is_refresh_token_valid(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| t.refresh_token_expires_at > Utc::now())
            .unwrap_or(false)
    }

    /// Reload token from file (useful if file was updated externally)
    pub fn reload(&mut self) -> Result<(), AppError> {
        let token_info = Self::load_from_file(&self.storage_path)?;
        self.token = Some(token_info);
        Ok(())
    }

    /// Get the storage file path
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }
}

impl Default for TokenStorage {
    fn default() -> Self {
        Self::new()
    }
}
