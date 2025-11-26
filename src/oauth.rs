use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, info};
use url::Url;

/// TikTok Shop OAuth client
#[derive(Clone)]
pub struct TikTokShopOAuth {
    app_key: String,
    app_secret: String,
    redirect_uri: String,
    http_client: Client,
    /// Store CSRF state tokens
    state_storage: std::sync::Arc<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

/// Authorization request parameters
#[derive(Debug, Serialize)]
pub struct AuthorizationRequest {
    pub app_key: String,
    pub state: String,
    pub redirect_uri: String,
}

/// OAuth callback parameters
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// Token exchange response
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub access_token_expire_in: i64,
    pub refresh_token: String,
    pub refresh_token_expire_in: i64,
}

/// Authorized shop information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthorizedShop {
    pub cipher: String,
    pub shop_id: String,
    pub shop_name: String,
    pub region: String,
}

/// API response wrapper
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl TikTokShopOAuth {
    // TikTok Shop API endpoints
    const AUTHORIZATION_URL: &'static str = "https://services.tiktokshop.com/open/authorize";
    const TOKEN_URL: &'static str = "https://auth.tiktok-shops.com/api/v2/token/get";
    const REFRESH_TOKEN_URL: &'static str = "https://auth.tiktok-shops.com/api/v2/token/refresh";
    const AUTHORIZED_SHOPS_URL: &'static str = "https://auth.tiktok-shops.com/api/v2/shops/get_authorized";

    pub fn new(app_key: String, app_secret: String, redirect_uri: String) -> Self {
        Self {
            app_key,
            app_secret,
            redirect_uri,
            http_client: Client::new(),
            state_storage: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate a random state for CSRF protection
    fn generate_state(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Build authorization URL for redirecting users
    pub fn get_authorization_url(&self) -> Result<String, AppError> {
        let state = self.generate_state();
        
        // Store state with expiration (10 minutes)
        {
            let mut storage = self.state_storage.lock().unwrap();
            let expiry = chrono::Utc::now() + chrono::Duration::minutes(10);
            storage.insert(state.clone(), expiry);
            
            // Clean up expired states
            let now = chrono::Utc::now();
            storage.retain(|_, expiry| *expiry > now);
        }

        let mut url = Url::parse(Self::AUTHORIZATION_URL)
            .map_err(|_| AppError::InvalidUrl)?;

        url.query_pairs_mut()
            .append_pair("app_key", &self.app_key)
            .append_pair("state", &state)
            .append_pair("redirect_uri", &self.redirect_uri);

        debug!("Generated authorization URL: {}", url);
        Ok(url.to_string())
    }

    /// Verify the state parameter from callback
    pub fn verify_state(&self, state: &str) -> bool {
        let mut storage = self.state_storage.lock().unwrap();
        
        if let Some(expiry) = storage.get(state) {
            let valid = *expiry > chrono::Utc::now();
            if valid {
                storage.remove(state); // Single use
            }
            valid
        } else {
            false
        }
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(&self, code: &str) -> Result<TokenResponse, AppError> {
        info!("Exchanging authorization code for access token");
        info!("Authorization code: {}", code);
        let mut params = HashMap::new();
        params.insert("app_key", self.app_key.as_str());
        params.insert("app_secret", self.app_secret.as_str());
        params.insert("auth_code", code);
        params.insert("grant_type", "authorized_code");

        // let url = format!("{} {}", (Self::TOKEN_URL.to_owned() + "?{}"), urlencoding::encode(&params));
        let response = self
            .http_client
            .get(Self::TOKEN_URL)
            .query(&params)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        debug!("Token response status: {}, body: {}", status, body);

        if !status.is_success() {
            return Err(AppError::TokenExchangeFailed(body));
        }

        let api_response: ApiResponse<TokenResponse> = serde_json::from_str(&body)
            .map_err(|e| AppError::ParseError(format!("Failed to parse token response: {}", e)))?;

        if api_response.code != 0 {
            return Err(AppError::ApiError(
                api_response.code,
                api_response.message,
            ));
        }

        api_response
            .data
            .ok_or_else(|| AppError::ApiError(api_response.code, "No token data in response".to_string()))
    }

    /// Refresh access token using refresh token
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse, AppError> {
        info!("Refreshing access token");

        let mut params = HashMap::new();
        params.insert("app_key", self.app_key.as_str());
        params.insert("app_secret", self.app_secret.as_str());
        params.insert("refresh_token", refresh_token);
        params.insert("grant_type", "refresh_token");

        let response = self
            .http_client
            .post(Self::REFRESH_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        debug!("Refresh token response status: {}, body: {}", status, body);

        if !status.is_success() {
            return Err(AppError::TokenRefreshFailed(body));
        }

        let api_response: ApiResponse<TokenResponse> = serde_json::from_str(&body)
            .map_err(|e| AppError::ParseError(format!("Failed to parse refresh response: {}", e)))?;

        if api_response.code != 0 {
            return Err(AppError::ApiError(
                api_response.code,
                api_response.message,
            ));
        }

        api_response
            .data
            .ok_or_else(|| AppError::ApiError(api_response.code, "No token data in response".to_string()))
    }

    /// Get list of authorized shops
    pub async fn get_authorized_shops(&self, access_token: &str) -> Result<Vec<AuthorizedShop>, AppError> {
        info!("Fetching authorized shops");

        let mut params = HashMap::new();
        params.insert("app_key", self.app_key.as_str());
        params.insert("app_secret", self.app_secret.as_str());
        params.insert("access_token", access_token);

        let response = self
            .http_client
            .get(Self::AUTHORIZED_SHOPS_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&params)
            .send()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        debug!("Authorized shops response status: {}, body: {}", status, body);

        if !status.is_success() {
            return Err(AppError::HttpError(format!("Failed to get shops: {}", body)));
        }

        #[derive(Deserialize)]
        struct ShopsData {
            shop_list: Vec<AuthorizedShop>,
        }

        let api_response: ApiResponse<ShopsData> = serde_json::from_str(&body)
            .map_err(|e| AppError::ParseError(format!("Failed to parse shops response: {}", e)))?;

        if api_response.code != 0 {
            return Err(AppError::ApiError(
                api_response.code,
                api_response.message,
            ));
        }

        let shops = api_response
            .data
            .map(|d| d.shop_list)
            .unwrap_or_default();

        Ok(shops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_state() {
        let oauth = TikTokShopOAuth::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );

        let state1 = oauth.generate_state();
        let state2 = oauth.generate_state();

        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_authorization_url_format() {
        let oauth = TikTokShopOAuth::new(
            "test_app_key".to_string(),
            "test_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );

        let url = oauth.get_authorization_url().unwrap();
        
        assert!(url.starts_with(TikTokShopOAuth::AUTHORIZATION_URL));
        assert!(url.contains("app_key=test_app_key"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fcallback"));
        assert!(url.contains("state="));
    }
}
