use axum::{
    extract::{State},
    routing::get,
    Json, Router,
};
use std::sync::{Arc};
use tokio::sync::RwLock;
use tracing::{info, error};
use std::time::Duration;
use tokio::time::sleep;


use tiktok_shop_oauth::config::Config;
use tiktok_shop_oauth::error::AppError;
use tiktok_shop_oauth::oauth::{TikTokShopOAuth};
use tiktok_shop_oauth::storage::{TokenStorage};
use tiktok_shop_oauth::database::Database;
use tiktok_shop_oauth::order::{GetOrderListRequest, OrderClient, Order};

#[derive(Clone)]
struct AppState {
    oauth_client: TikTokShopOAuth,
    token_storage: Arc<RwLock<TokenStorage>>,
    db: Arc<Database>,
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

    // Initialize database
    let db = Arc::new(Database::new("orders.db").await?);
    db.init().await?;

    // Initialize OAuth client
    let oauth_client = TikTokShopOAuth::new(config.app_key.clone(), config.app_secret.clone());

    // Initialize token storage (loads from file if exists)
    let token_storage = Arc::new(RwLock::new(TokenStorage::new()));
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

    // Spawn background task
    let db_clone = db.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        fetch_orders_task(db_clone, config_clone).await;
    });

    let state = AppState {
        oauth_client,
        token_storage,
        db,
    };

    // Build router
    let app = Router::new()
        .route("/auth/refresh", get(refresh_token_endpoint))
        .route("/auth/status", get(auth_status))
        .route("/orders", get(get_orders_endpoint))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn fetch_orders_task(db: Arc<Database>, config: Config) {
    let order_client = OrderClient::new(
        config.app_key.clone(),
        config.app_secret.clone(),
    );
    loop {
        info!("Fetching orders...");
        
        let token_data = match std::fs::read_to_string(&config.token_file) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to read token file: {}", e);
                sleep(Duration::from_secs(3600)).await;
                continue;
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&token_data) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to parse token file: {}", e);
                sleep(Duration::from_secs(3600)).await;
                continue;
            }
        };

        let access_token = match json["data"]["access_token"].as_str() {
            Some(token) => token,
            None => {
                error!("Access token not found in token file");
                sleep(Duration::from_secs(3600)).await;
                continue;
            }
        };

        let shop_cipher = config.shop_cipher.as_deref();
        let shop_id = config.shop_id.as_deref();

        let request = GetOrderListRequest::new().with_page_size(50);
        match order_client.get_order_list(access_token, shop_cipher, shop_id, request).await {
            Ok(response) => {
                info!("Fetched {} orders", response.orders.len());
                if let Err(e) = db.upsert_orders(&response.orders).await {
                    error!("Failed to save orders to database: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to fetch orders: {}", e);
            }
        }
        
        sleep(Duration::from_secs(3600)).await;
    }
}

async fn get_orders_endpoint(State(state): State<AppState>) -> Result<Json<Vec<Order>>, AppError> {
    let orders = state.db.get_orders().await.map_err(|e| {
        error!("Failed to get orders from database: {}", e);
        AppError::InternalServerError
    })?;
    Ok(Json(orders))
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
