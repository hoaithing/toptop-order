use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// TikTok Shop OAuth client
#[derive(Clone)]
pub struct TikTokShopOAuth {
    app_key: String,
    app_secret: String,
    http_client: Client,
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
    const TOKEN_URL: &'static str = "https://auth.tiktok-shops.com/api/v2/token/get";
    const REFRESH_TOKEN_URL: &'static str = "https://auth.tiktok-shops.com/api/v2/token/refresh";

    pub fn new(app_key: String, app_secret: String) -> Self {
        Self {
            app_key,
            app_secret,
            http_client: Client::new(),
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

}
