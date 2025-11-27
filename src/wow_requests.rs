use hmac::{Hmac, Mac};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct WowEsimApiClient {
    wow_secret: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct WowApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignatureBody<T> {
    pub signature: String,
    pub timestamp: i64,
    pub data: T,
}

#[derive(Debug)]
pub enum WowApiError {
    SignatureError(String),
    HttpError(String),
    ParseError(String),
    ApiError(String),
}

impl std::fmt::Display for WowApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WowApiError::SignatureError(e) => write!(f, "Signature error: {}", e),
            WowApiError::HttpError(e) => write!(f, "HTTP error: {}", e),
            WowApiError::ParseError(e) => write!(f, "Parse error: {}", e),
            WowApiError::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for WowApiError {}


impl Default for WowEsimApiClient {
    fn default() -> Self {
        Self::new(dotenvy::var("WOW_SECRET").expect("WOW_SECRET env var not set"))
    }
}

impl WowEsimApiClient {
    // const API_BASE_URL: &'static str = "https://api.wowesim.com/";

    /// Create a new WowEsimApiClient with the given secret
    pub fn new(wow_secret: String) -> Self {
        Self {
            wow_secret,
            http_client: reqwest::Client::new(),
        }
    }

    /// Generate HMAC-SHA256 signature for WowEsim API
    ///
    /// Format: ?key1=value1&key2=value2&timestamp=xxx
    fn generate_signature(
        &self,
        body: &BTreeMap<String, String>,
        timestamp: i64,
    ) -> Result<String, WowApiError> {
        let mut sign_string = String::new();

        // Build query string format
        body.iter().enumerate().for_each(|(i, (k, v))| {
            if i != 0 {
                sign_string.push('&');
            }
            sign_string.push_str(&format!("{}={}", k, v));
        });
        sign_string.push_str(&format!("&timestamp={}", timestamp));

        println!("Sign string: {}", sign_string);

        // Generate HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(self.wow_secret.as_bytes())
            .map_err(|e| WowApiError::SignatureError(e.to_string()))?;
        mac.update(sign_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        println!("Generated signature: {}", signature);

        Ok(signature)
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &BTreeMap<String, String>,
    ) -> Result<WowApiResponse<T>, WowApiError> {
        let timestamp = chrono::Utc::now().timestamp();

        // Generate signature
        let signature = self.generate_signature(body, timestamp)?;

        let signature_body = SignatureBody {
            signature,
            timestamp,
            data: body,
        };

        let api_base_url = env::var("WOW_API_BASE_URL").expect("WOW_API_BASE_URL env var not set");
        let url = format!("{}{}", api_base_url, path);
        println!("Making POST request to: {}", url);
        println!("Request body: {:?}", &signature_body);

        // Serialize body
        let body_json = serde_json::to_string(&signature_body)
            .map_err(|e| WowApiError::ParseError(format!("Failed to serialize body: {}", e)))?;

        // Make request
        let response = self.http_client
            .post(&url)
            .body(body_json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| WowApiError::HttpError(e.to_string()))?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .map_err(|e| WowApiError::HttpError(e.to_string()))?;

        println!("Response status: {}, body: {}", status, response_body);

        // Check HTTP status
        if !status.is_success() {
            return Err(WowApiError::ApiError(format!(
                "HTTP {} - {}",
                status, response_body
            )));
        }

        // Parse response
        let api_response: WowApiResponse<T> = serde_json::from_str(&response_body)
            .map_err(|e| WowApiError::ParseError(format!("Failed to parse response: {}", e)))?;

        // Check API success flag
        if !api_response.success {
            return Err(WowApiError::ApiError(
                api_response.message.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(api_response)
    }

    /// Make a simple POST request without parsing response data
    pub async fn post_simple(
        &self,
        path: &str,
        body: &BTreeMap<String, String>,
    ) -> Result<bool, WowApiError> {
        let response: WowApiResponse<serde_json::Value> = self.post(path, body).await?;
        Ok(response.success)
    }
}
