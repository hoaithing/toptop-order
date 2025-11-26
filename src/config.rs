use crate::error::AppError;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_key: String,
    pub app_secret: String,
    pub redirect_uri: String,
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            app_key: env::var("TIKTOK_APP_KEY")
                .map_err(|_| AppError::ConfigError("TIKTOK_APP_KEY not set".to_string()))?,
            app_secret: env::var("TIKTOK_APP_SECRET")
                .map_err(|_| AppError::ConfigError("TIKTOK_APP_SECRET not set".to_string()))?,
            redirect_uri: env::var("TIKTOK_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
        })
    }
}
