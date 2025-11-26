use crate::error::AppError;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_key: String,
    pub app_secret: String,
    pub shop_cipher: Option<String>,
    pub shop_id: Option<String>,
    pub token_file: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            app_key: env::var("TIKTOK_APP_KEY")
                .map_err(|_| AppError::ConfigError("TIKTOK_APP_KEY not set".to_string()))?,
            app_secret: env::var("TIKTOK_APP_SECRET")
                .map_err(|_| AppError::ConfigError("TIKTOK_APP_SECRET not set".to_string()))?,
            shop_cipher: env::var("TIKTOK_SHOP_CIPHER").ok(),
            shop_id: env::var("TIKTOK_SHOP_ID").ok(),
            token_file: env::var("TIKTOK_TOKEN_FILE")
                .unwrap_or_else(|_| "token.json".to_string()),
        })
    }
}
