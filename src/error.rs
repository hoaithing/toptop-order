use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid state parameter - possible CSRF attack")]
    InvalidState,

    #[error("No token stored")]
    NoTokenStored,

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("API error (code {0}): {1}")]
    ApiError(i32, String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Signature generation error: {0}")]
    SignatureError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidState => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoTokenStored => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidUrl => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::HttpError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::TokenExchangeFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::TokenRefreshFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ApiError(_, _) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ParseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::SignatureError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
