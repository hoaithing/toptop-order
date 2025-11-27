use axum::{extract::State, routing::get, Json, Router};
use chrono::DateTime;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use tiktok_shop_order::config::Config;
use tiktok_shop_order::database::Database;
use tiktok_shop_order::oauth::TikTokShopOAuth;
use tiktok_shop_order::order::{GetOrderListRequest, OrderClient};
use tiktok_shop_order::storage::{TokenInfo, TokenStorage};

#[derive(Clone)]
struct AppState {
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

    // Initialize OAuth client
    let oauth_client = TikTokShopOAuth::new(config.app_key.clone(), config.app_secret.clone());

    // Initialize token storage (loads from file if exists)
    let token_storage = Arc::new(RwLock::new(TokenStorage::new()));

    // Check and refresh token if needed
    {
        let storage = token_storage.read().await;
        if let Some(token_info) = storage.get() {
            info!(
                "Loaded saved token from {}",
                storage.storage_path().display()
            );
            info!("Token expires at: {}", token_info.expires_at);

            // Check if access token expired
            if token_info.expires_at < chrono::Utc::now() {
                info!("Access token expired. Refreshing...");

                // Check if refresh token is still valid
                if token_info.refresh_token_expires_at < chrono::Utc::now() {
                    info!("Refresh token expired. Please authorize again.");
                } else {
                    // Drop read lock before refreshing
                    let refresh_token = token_info.refresh_token.clone();
                    drop(storage);

                    // Refresh the token
                    let token_response = oauth_client
                        .refresh_access_token(&refresh_token)
                        .await
                        .expect("Failed to refresh token");

                    info!("Token refreshed successfully");

                    // Create new token info with refreshed data
                    let new_token_info = TokenInfo {
                        access_token: token_response.access_token,
                        refresh_token: token_response.refresh_token,
                        expires_at: DateTime::from_timestamp(token_response.access_token_expire_in, 0)
                            .expect("Failed to parse access token expire time"),
                        refresh_token_expires_at: DateTime::from_timestamp(token_response.refresh_token_expire_in, 0)
                            .expect("Failed to parse refresh token expire time"),
                    };

                    // Store the new token info
                    let mut storage = token_storage.write().await;
                    storage.store(new_token_info)
                        .expect("Failed to store refreshed token");
                    info!("Refreshed token saved to file");
                }
            } else if token_info.refresh_token_expires_at < chrono::Utc::now() {
                info!("Refresh token expired. Please authorize again.");
            }
        } else {
            info!("No saved token found. Please authorize via /auth/tiktok");
        }
    }

    // Initialize database
    info!("Initializing database at {}", config.database_path);
    let db = Database::new(&config.database_path).await?;
    db.init().await?;
    info!("Database initialized");

    let db = Arc::new(db);

    // Start background sync task
    let db_clone = db.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        sync_orders_background_task(db_clone, config_clone).await;
    });

    // Create app state
    let state = AppState {
        db: db.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/orders", get(get_orders_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_orders_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match state.db.get_orders().await {
        Ok(orders) => {
            Json(serde_json::json!({
                "success": true,
                "count": orders.len(),
                "orders": orders
            }))
        }
        Err(e) => {
            error!("Failed to get orders from database: {}", e);
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

async fn sync_orders_background_task(db: Arc<Database>, config: Config) {
    info!("Starting background order sync task (runs every hour)");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour

    loop {
        interval.tick().await;

        info!("Running order sync...");

        // Read token from file
        let token_storage = TokenStorage::new();
        let token_info = match token_storage.get() {
            Some(token) => token,
            None => {
                error!("No token found, skipping sync");
                continue;
            }
        };

        // Check if token is valid
        if token_info.expires_at < chrono::Utc::now() {
            error!("Access token expired, skipping sync. Please refresh token.");
            continue;
        }

        // Create order client
        let order_client = OrderClient::new(
            config.app_key.clone(),
            config.app_secret.clone(),
        );

        // Fetch orders
        let request = GetOrderListRequest::new().with_page_size(50);

        match order_client
            .get_order_list(
                &token_info.access_token,
                config.shop_cipher.as_deref(),
                config.shop_id.as_deref(),
                request,
            )
            .await
        {
            Ok(response) => {
                info!("Fetched {} orders from API", response.orders.len());

                // Save to database
                match db.upsert_orders(&response.orders).await {
                    Ok(_) => {
                        info!("Successfully synced {} orders to database", response.orders.len());
                    }
                    Err(e) => {
                        error!("Failed to save orders to database: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch orders from API: {}", e);
            }
        }
    }
}
