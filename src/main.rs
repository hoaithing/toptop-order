use axum::{
    extract::{State},
    routing::get,
    Json, Router,
};
use std::sync::{Arc};
use tokio::sync::RwLock;
use tracing::{info};

use tiktok_shop_oauth::config::Config;
use tiktok_shop_oauth::oauth::{TikTokShopOAuth};
use tiktok_shop_oauth::storage::{TokenStorage};

// #[derive(Clone)]
// struct AppState {
//     token_storage: Arc<RwLock<TokenStorage>>,
// }

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
    let oauth_client = TikTokShopOAuth::new(config.app_key, config.app_secret);

    // Initialize token storage (loads from file if exists)
    let token_storage = Arc::new(RwLock::new(TokenStorage::new()));
    {
        let storage = token_storage.read().await;
        if let Some(token_info) = storage.get() {
            info!("Loaded saved token from {}", storage.storage_path().display());
            info!("Token expires at: {}", token_info.expires_at);
            if token_info.expires_at < chrono::Utc::now() {
                let token_response = oauth_client.refresh_access_token(&token_info.refresh_token).await.expect("Failed to refresh token");
                // todo: try to store new token info
                println!("{:?}", token_response);
            }
            if token_info.refresh_token_expires_at < chrono::Utc::now() {
                info!("Refresh token expired. Please authorize again.");
            }

        } else {
            info!("No saved token found. Please authorize via /auth/tiktok");
        }
    }

    // let state = AppState {
    //     token_storage,
    // };

    // Build router
    let app = Router::new();

    let addr = "0.0.0.0:3000";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

