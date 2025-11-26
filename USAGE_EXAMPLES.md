# TikTok Shop API Usage Examples

This document shows how to use the TikTok Shop API client with signed requests.

## Table of Contents
- [Token Storage](#token-storage)
- [Making Signed API Requests](#making-signed-api-requests)
- [Order Management](#order-management)
- [Shop Cipher](#shop-cipher)
- [Complete Example](#complete-example)

## Token Storage

The `TokenStorage` now persists tokens to a file (`tiktok_tokens.json` by default), allowing you to preserve authentication across application restarts.

```rust
use tiktok_shop_oauth::storage::TokenStorage;

// Create storage with default file path (tiktok_tokens.json)
let mut storage = TokenStorage::new();

// Create storage with custom file path
let mut storage = TokenStorage::with_path("custom_tokens.json");

// Store a token (automatically saves to file)
storage.store(token_info)?;

// Get the token
if let Some(token) = storage.get() {
    println!("Access token: {}", token.access_token);
}

// Check if tokens are valid
if storage.is_access_token_valid() {
    println!("Access token is still valid");
}

// Reload from file (if modified externally)
storage.reload()?;

// Clear token and delete file
storage.clear()?;
```

## Making Signed API Requests

Use the `TikTokShopApiClient` to make authenticated API requests with automatic signature generation.

```rust
use tiktok_shop_oauth::requests::TikTokShopApiClient;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

// Initialize the API client
let api_client = TikTokShopApiClient::new(
    app_key.clone(),
    app_secret.clone(),
);

// Example: Get order list
#[derive(Deserialize)]
struct OrderListResponse {
    orders: Vec<Order>,
    total: i64,
}

let mut params = BTreeMap::new();
params.insert("page_size".to_string(), "20".to_string());

let response: OrderListResponse = api_client
    .get(
        "/api/orders/search",
        Some(&access_token),
        Some(&shop_cipher),
        params,
    )
    .await?;

println!("Found {} orders", response.orders.len());
```

## Order Management

The `OrderClient` provides methods for managing TikTok Shop orders.

### Get Order List

```rust
use tiktok_shop_oauth::order::{OrderClient, GetOrderListRequest, OrderStatus, SortOrder};

// Create order client
let order_client = OrderClient::new(
    app_key.clone(),
    app_secret.clone(),
);

// Basic request - get first 10 orders
let request = GetOrderListRequest::new();

let response = order_client
    .get_order_list(&access_token, &shop_cipher, request)
    .await?;

println!("Total orders: {}", response.total);
for order in response.orders {
    println!("Order ID: {}, Status: {}", order.id, order.status);
}
```

### Filter Orders by Status

```rust
// Get only orders awaiting shipment
let request = GetOrderListRequest::new()
    .with_status(OrderStatus::AwaitingShipment)
    .with_page_size(20);

let response = order_client
    .get_order_list(&access_token, &shop_cipher, request)
    .await?;
```

### Filter by Time Range

```rust
use chrono::{Utc, Duration};

// Get orders created in the last 7 days
let now = Utc::now().timestamp();
let seven_days_ago = (Utc::now() - Duration::days(7)).timestamp();

let request = GetOrderListRequest::new()
    .with_create_time_range(seven_days_ago, now)
    .with_page_size(50);

let response = order_client
    .get_order_list(&access_token, &shop_cipher, request)
    .await?;
```

### Pagination

```rust
// Get first page
let mut request = GetOrderListRequest::new()
    .with_page_size(20)
    .sort_by("create_time".to_string(), SortOrder::Descending);

let mut all_orders = Vec::new();

loop {
    let response = order_client
        .get_order_list(&access_token, &shop_cipher, request.clone())
        .await?;

    all_orders.extend(response.orders);

    // Check if there are more pages
    if !response.more {
        break;
    }

    // Get next page
    if let Some(next_token) = response.next_page_token {
        request = request.with_page_token(next_token);
    } else {
        break;
    }
}

println!("Retrieved {} total orders", all_orders.len());
```

### Available Order Statuses

```rust
use tiktok_shop_oauth::order::OrderStatus;

// All available order statuses
let statuses = vec![
    OrderStatus::Unpaid,              // 100
    OrderStatus::AwaitingShipment,    // 111
    OrderStatus::AwaitingCollection,  // 112
    OrderStatus::PartiallyShipped,    // 114
    OrderStatus::InTransit,           // 121
    OrderStatus::Delivered,           // 122
    OrderStatus::Completed,           // 130
    OrderStatus::Cancelled,           // 140
];
```

### Process Order Details

```rust
let response = order_client
    .get_order_list(&access_token, &shop_cipher, GetOrderListRequest::new())
    .await?;

for order in response.orders {
    println!("Order: {}", order.id);

    // Payment information
    if let Some(payment) = &order.payment {
        println!("  Total: {} {}", payment.total_amount, payment.currency);
        println!("  Shipping: {}", payment.shipping_fee);
    }

    // Order items
    for item in &order.item_list {
        println!("  Item: {} x{}", item.product_name, item.quantity);
        println!("    SKU: {}", item.sku_id);
        println!("    Price: {}", item.sale_price);
    }

    // Delivery information
    if let Some(delivery) = &order.delivery {
        if let Some(tracking) = &delivery.tracking_number {
            println!("  Tracking: {}", tracking);
        }
    }

    // Recipient
    if let Some(address) = &order.recipient_address {
        if let Some(name) = &address.name {
            println!("  Ship to: {}", name);
        }
    }
}
```

## Making POST Requests

```rust
use serde::Serialize;

#[derive(Serialize)]
struct UpdateProductRequest {
    product_id: String,
    title: String,
    description: String,
}

let request_body = UpdateProductRequest {
    product_id: "123456".to_string(),
    title: "New Product Title".to_string(),
    description: "Updated description".to_string(),
};

let response: ProductResponse = api_client
    .post(
        "/api/products/update",
        Some(&access_token),
        Some(&shop_cipher),
        &request_body,
    )
    .await?;
```

## Shop Cipher

The `shop_cipher` field from `AuthorizedShop` is required for most TikTok Shop API calls:

```rust
// After OAuth authorization, get authorized shops
let shops = oauth_client.get_authorized_shops(&access_token).await?;

// Use the shop cipher in API requests
for shop in shops {
    println!("Shop: {} ({})", shop.shop_name, shop.cipher);

    // Make API call with shop_cipher
    let orders: OrderListResponse = api_client
        .get(
            "/api/orders/search",
            Some(&access_token),
            Some(&shop.cipher),  // Use cipher here
            params.clone(),
        )
        .await?;
}
```

## Complete Example

```rust
use tiktok_shop_oauth::{
    config::Config,
    oauth::TikTokShopOAuth,
    requests::TikTokShopApiClient,
    storage::TokenStorage,
};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::from_env()?;

    // Initialize OAuth client
    let oauth_client = TikTokShopOAuth::new(
        config.app_key.clone(),
        config.app_secret.clone(),
        config.redirect_uri,
    );

    // Initialize API client
    let api_client = TikTokShopApiClient::new(
        config.app_key.clone(),
        config.app_secret.clone(),
    );

    // Initialize token storage (loads from file if exists)
    let mut storage = TokenStorage::new();

    // Check if we have a valid token
    if !storage.is_access_token_valid() {
        println!("No valid token found. Please authorize first.");
        // Perform OAuth flow...
        return Ok(());
    }

    // Get token info
    let token_info = storage.get().unwrap();

    // Use the first shop
    if let Some(shop) = token_info.shops.first() {
        println!("Using shop: {}", shop.shop_name);

        // Make API request
        let mut params = BTreeMap::new();
        params.insert("page_size".to_string(), "10".to_string());

        let response: serde_json::Value = api_client
            .get(
                "/api/orders/search",
                Some(&token_info.access_token),
                Some(&shop.cipher),
                params,
            )
            .await?;

        println!("Response: {:#?}", response);
    }

    Ok(())
}
```

## API Signature Details

The `TikTokShopApiClient` automatically generates HMAC-SHA256 signatures for all requests:

1. Constructs a signing string from: `app_key + timestamp + access_token + shop_cipher + path + sorted_params`
2. Generates HMAC-SHA256 using the app secret
3. Adds the signature to request parameters as `sign`

All common parameters (`app_key`, `timestamp`, `access_token`, `shop_cipher`, `sign`) are automatically added to requests.

## Error Handling

All API methods return `Result<T, AppError>`:

```rust
match api_client.get(...).await {
    Ok(response) => {
        // Handle success
    }
    Err(AppError::ApiError(code, message)) => {
        println!("API error {}: {}", code, message);
    }
    Err(AppError::SignatureError(msg)) => {
        println!("Signature generation failed: {}", msg);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```
