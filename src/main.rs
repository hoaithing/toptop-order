use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use tiktok_shop_oauth::config::Config;
use tiktok_shop_oauth::error::AppError;
use tiktok_shop_oauth::oauth::{TikTokShopOAuth, CallbackParams};
use tiktok_shop_oauth::storage::{TokenStorage, TokenInfo};

#[derive(Clone)]
struct AppState {
    // config: Config,
    oauth_client: TikTokShopOAuth,
    token_storage: Arc<RwLock<TokenStorage>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Initialize OAuth client
    let oauth_client = TikTokShopOAuth::new(
        config.app_key.clone(),
        config.app_secret.clone(),
        config.redirect_uri.clone(),
    );

    // Initialize token storage (loads from file if exists)
    let token_storage = Arc::new(RwLock::new(TokenStorage::new()));

    // Check if we have a saved token
    {
        let storage = token_storage.read().await;
        if let Some(token_info) = storage.get() {
            info!("Loaded saved token from {}", storage.storage_path().display());
            info!("Token expires at: {}", token_info.expires_at);
            info!("Saved shops: {}", token_info.shops.len());
            for shop in &token_info.shops {
                info!("  - {} (cipher: {})", shop.shop_name, shop.cipher);
            }
        } else {
            info!("No saved token found. Please authorize via /auth/tiktok");
        }
    }

    let state = AppState {
        // config: config.clone(),
        oauth_client,
        token_storage,
    };

    // Build router
    let app = Router::new()
        .route("/", get(home))
        .route("/auth/tiktok", get(initiate_authorization))
        .route("/auth/callback", get(authorization_callback))
        .route("/auth/refresh", get(refresh_token_endpoint))
        .route("/auth/status", get(auth_status))
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Home page with links to start authorization
async fn home() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>TikTok Shop OAuth Demo</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                }
                button {
                    background-color: #000;
                    color: white;
                    padding: 12px 24px;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 16px;
                }
                button:hover {
                    background-color: #333;
                }
                .status {
                    margin-top: 20px;
                    padding: 10px;
                    background-color: #f5f5f5;
                    border-radius: 4px;
                }
            </style>
        </head>
        <body>
            <h1>TikTok Shop OAuth Demo</h1>
            <p>Click the button below to authorize this application with TikTok Shop.</p>
            <a href="/auth/tiktok">
                <button>Authorize with TikTok Shop</button>
            </a>
            <div class="status">
                <h3>Current Status</h3>
                <p><a href="/auth/status">Check Authorization Status</a></p>
            </div>
        </body>
        </html>
        "#,
    )
}

/// Initiate the authorization flow
async fn initiate_authorization(
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    // Generate authorization URL
    let auth_url = state.oauth_client.get_authorization_url()?;
    
    info!("Redirecting to TikTok authorization: {}", auth_url);
    
    Ok(Redirect::to(&auth_url))
}

/// Handle the OAuth callback
async fn authorization_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    info!("Received callback with code");

    // Verify state to prevent CSRF attacks
    if !state.oauth_client.verify_state(&params.state) {
        error!("Invalid state parameter");
        return Err(AppError::InvalidState);
    }

    // Exchange authorization code for access token
    let token_response = state
        .oauth_client
        .exchange_code_for_token(&params.code)
        .await?;

    info!("Successfully obtained access token");

    // Get authorized shops
    let shops = state
        .oauth_client
        .get_authorized_shops(&token_response.access_token)
        .await?;

    info!("Retrieved {} authorized shops", shops.len());

    // Store token information
    let token_info = TokenInfo {
        access_token: token_response.access_token.clone(),
        refresh_token: token_response.refresh_token.clone(),
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(token_response.access_token_expire_in),
        refresh_token_expires_at: chrono::Utc::now() + chrono::Duration::seconds(token_response.refresh_token_expire_in),
        shops: shops.clone(),
    };

    if let Err(e) = state.token_storage.write().await.store(token_info) {
        error!("Failed to store token: {}", e);
        return Err(e);
    }

    // Create success page
    let shops_html = shops
        .iter()
        .map(|shop| {
            format!(
                r#"<div class="shop">
                    <h3>{}</h3>
                    <p>Shop ID: {}</p>
                    <p>Region: {}</p>
                    <p>Cipher: {}</p>
                </div>"#,
                shop.shop_name, shop.shop_id, shop.region, shop.cipher
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Authorization Successful</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                }}
                .success {{
                    background-color: #d4edda;
                    color: #155724;
                    padding: 15px;
                    border-radius: 4px;
                    margin-bottom: 20px;
                }}
                .shop {{
                    background-color: #f8f9fa;
                    padding: 15px;
                    margin: 10px 0;
                    border-radius: 4px;
                    border-left: 4px solid #000;
                }}
                .token-info {{
                    background-color: #fff3cd;
                    padding: 15px;
                    border-radius: 4px;
                    margin: 20px 0;
                }}
                code {{
                    background-color: #f5f5f5;
                    padding: 2px 6px;
                    border-radius: 3px;
                    font-family: monospace;
                }}
            </style>
        </head>
        <body>
            <div class="success">
                <h2>✓ Authorization Successful!</h2>
                <p>Your TikTok Shop account has been successfully connected.</p>
            </div>
            
            <div class="token-info">
                <h3>Token Information</h3>
                <p>Access Token: <code>{}...</code></p>
                <p>Expires in: {} seconds</p>
                <p>Refresh Token: <code>{}...</code></p>
            </div>

            <h2>Authorized Shops</h2>
            {}

            <p><a href="/">← Back to Home</a></p>
        </body>
        </html>
        "#,
        &token_response.access_token[..20],
        token_response.access_token_expire_in,
        &token_response.refresh_token[..20],
        shops_html
    );

    Ok(Html(html))
}

/// Refresh access token endpoint
async fn refresh_token_endpoint(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let storage = state.token_storage.read().await;
    
    let token_info = &storage
        .get()
        .ok_or(AppError::NoTokenStored)?;

    // Check if token is expired
    if chrono::Utc::now() < token_info.expires_at {
        return Ok(Json(serde_json::json!({
            "message": "Token is still valid",
            "expires_at": token_info.expires_at
        })));
    }

    // drop(storage);

    info!("Refreshing access token");

    // Refresh the token
    let new_token = state
        .oauth_client
        .refresh_access_token(&token_info.refresh_token)
        .await?;

    // Update storage
    let mut storage = state.token_storage.write().await;
    if let Some(mut token_info) = storage.get().cloned() {
        token_info.access_token = new_token.access_token.clone();
        token_info.refresh_token = new_token.refresh_token.clone();
        token_info.expires_at = chrono::Utc::now() + chrono::Duration::seconds(new_token.access_token_expire_in);
        token_info.refresh_token_expires_at = chrono::Utc::now() + chrono::Duration::seconds(new_token.refresh_token_expire_in);
        storage.store(token_info)?;
    }

    Ok(Json(serde_json::json!({
        "message": "Token refreshed successfully",
        "expires_in": new_token.access_token_expire_in
    })))
}

/// Check current authorization status
async fn auth_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let storage = state.token_storage.read().await;
    
    match storage.get() {
        Some(token_info) => {
            let is_expired = chrono::Utc::now() >= token_info.expires_at;
            let refresh_token_expired = chrono::Utc::now() >= token_info.refresh_token_expires_at;

            Ok(Json(serde_json::json!({
                "authorized": true,
                "access_token_expired": is_expired,
                "refresh_token_expired": refresh_token_expired,
                "expires_at": token_info.expires_at,
                "shops": token_info.shops,
            })))
        }
        None => Ok(Json(serde_json::json!({
            "authorized": false,
            "message": "No authorization found. Please authorize first."
        }))),
    }
}
