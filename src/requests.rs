use crate::error::AppError;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use tracing::debug;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct TikTokShopApiClient {
    app_key: String,
    app_secret: String,
    http_client: Client,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub request_id: Option<String>,
}

impl TikTokShopApiClient {
    const API_BASE_URL: &'static str = "https://open-api.tiktokglobalshop.com";

    pub fn new(app_key: String, app_secret: String) -> Self {
        Self {
            app_key,
            app_secret,
            http_client: Client::new(),
        }
    }

    fn generate_signature(
        &self,
        path: &str,
        params: &BTreeMap<String, String>,
        timestamp: i64,
        access_token: Option<&str>,
        shop_cipher: Option<&str>,
    ) -> Result<String, AppError> {
        let mut sign_string = String::new();
        sign_string.push_str(&self.app_key);
        sign_string.push_str(&timestamp.to_string());

        if let Some(token) = access_token {
            sign_string.push_str(token);
        }

        if let Some(cipher) = shop_cipher {
            sign_string.push_str(cipher);
        }

        sign_string.push_str(path);

        for (key, value) in params.iter() {
            sign_string.push_str(key);
            sign_string.push_str(value);
        }

        debug!("Sign string: {}", sign_string);

        let mut mac = HmacSha256::new_from_slice(self.app_secret.as_bytes())
            .map_err(|e| AppError::SignatureError(e.to_string()))?;
        mac.update(sign_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());
        Ok(signature)
    }

    fn generate_signature_with_body(
        &self,
        path: &str,
        params: &BTreeMap<String, String>,
        body_json: &str,
    ) -> Result<String, AppError> {
        let mut params_string = String::new();
        for (key, value) in params.iter() {
            // Skip access_token and sign as per docs
            if key == "access_token" || key == "sign" {
                continue;
            }
            params_string.push_str(key);
            params_string.push_str(value);
        }

        let sign_string = format!("{}{}{}", path, params_string, body_json);
        let wrapped_string = format!("{}{}{}", self.app_secret, sign_string, self.app_secret);

        debug!("Sign string: {}", sign_string);
        debug!("Wrapped string: {}", wrapped_string);

        let mut mac = HmacSha256::new_from_slice(self.app_secret.as_bytes())
            .map_err(|e| AppError::SignatureError(e.to_string()))?;
        mac.update(wrapped_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        debug!("Generated signature: {}", signature);

        Ok(signature)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        access_token: Option<&str>,
        shop_cipher: Option<&str>,
        mut params: BTreeMap<String, String>,
    ) -> Result<T, AppError> {
        let timestamp = chrono::Utc::now().timestamp();

        // Add required common parameters
        params.insert("app_key".to_string(), self.app_key.clone());
        params.insert("timestamp".to_string(), timestamp.to_string());

        if let Some(token) = access_token {
            params.insert("access_token".to_string(), token.to_string());
        }

        if let Some(cipher) = shop_cipher {
            params.insert("shop_cipher".to_string(), cipher.to_string());
        }

        let signature = self.generate_signature(path, &params, timestamp, access_token, shop_cipher)?;
        params.insert("sign".to_string(), signature);
        let url = format!("{}{}", Self::API_BASE_URL, path);
        debug!("Making GET request to: {}", url);
        debug!("Parameters: {:?}", params);

        let mut request_builder = self
            .http_client
            .get(&url)
            .query(&params)
            .header("Content-Type", "application/json");

        if let Some(token) = access_token {
            request_builder = request_builder.header("x-tts-access-token", token);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        debug!("Response status: {}, body: {}", status, body);

        if !status.is_success() {
            return Err(AppError::HttpError(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let api_response: ApiResponse<T> = serde_json::from_str(&body)
            .map_err(|e| AppError::ParseError(format!("Failed to parse response: {}", e)))?;

        if api_response.code != 0 {
            return Err(AppError::ApiError(
                api_response.code,
                api_response.message,
            ));
        }

        api_response
            .data
            .ok_or_else(|| AppError::ApiError(api_response.code, "No data in response".to_string()))
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        access_token: Option<&str>,
        shop_cipher: Option<&str>,
        body: &B,
        extra_params: Option<BTreeMap<String, String>>,
    ) -> Result<T, AppError> {
        let timestamp = chrono::Utc::now().timestamp();

        // Serialize body to JSON string
        let body_json = serde_json::to_string(body)
            .map_err(|e| AppError::ParseError(format!("Failed to serialize body: {}", e)))?;

        let mut params = BTreeMap::new();
        params.insert("app_key".to_string(), self.app_key.clone());
        params.insert("timestamp".to_string(), timestamp.to_string());

        // access_token may be passed both in query and header
        if let Some(token) = access_token {
            params.insert("access_token".to_string(), token.to_string());
        }

        if let Some(cipher) = shop_cipher {
            params.insert("shop_cipher".to_string(), cipher.to_string());
        }

        // Add any extra query parameters (e.g., page_size, shop_id, version)
        if let Some(extra) = extra_params {
            for (key, value) in extra {
                params.insert(key, value);
            }
        }

        // For POST requests, generate signature including ALL query params and the request body
        let signature = self.generate_signature_with_body(path, &params, &body_json)?;
        params.insert("sign".to_string(), signature);

        let url = format!("{}{}", Self::API_BASE_URL, path);

        debug!("Making POST request to: {}", url);
        debug!("Query parameters: {:?}", params);
        debug!("Request body: {}", body_json);

        // Make request with required headers
        let mut request_builder = self
            .http_client
            .post(&url)
            .query(&params)
            .header("Content-Type", "application/json");

        if let Some(token) = access_token {
            request_builder = request_builder.header("x-tts-access-token", token);
        }

        let response = request_builder
            .body(body_json)
            .send()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        debug!("Response status: {}, body: {}", status, response_body);

        if !status.is_success() {
            return Err(AppError::HttpError(format!(
                "API request failed with status {}: {}",
                status, response_body
            )));
        }

        // Parse response;
        let api_response: ApiResponse<T> = serde_json::from_str(&response_body)
            .map_err(|e| AppError::ParseError(format!("Failed to parse response: {}", e)))?;

        if api_response.code != 0 {
            return Err(AppError::ApiError(
                api_response.code,
                api_response.message,
            ));
        }

        api_response
            .data
            .ok_or_else(|| AppError::ApiError(api_response.code, "No data in response".to_string()))
    }
}
