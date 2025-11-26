# TikTok Shop OAuth Implementation in Rust

A complete implementation of the TikTok Shop authorization workflow using Rust, Axum, and async/await patterns.

## Features

- ✅ **OAuth 2.0 Authorization Code Flow** - Full implementation with CSRF protection
- ✅ **Access Token Management** - Automatic token storage and validation
- ✅ **Token Refresh** - Refresh expired access tokens using refresh tokens
- ✅ **Authorized Shops** - Retrieve list of authorized TikTok Shop stores
- ✅ **Web Interface** - Simple HTML interface for testing the OAuth flow
- ✅ **Type-Safe** - Full Rust type safety with comprehensive error handling
- ✅ **Production-Ready** - Structured code with proper separation of concerns

## Architecture

```
src/
├── main.rs       # Application entry point and route handlers
├── oauth.rs      # TikTok Shop OAuth client implementation
├── config.rs     # Configuration management
├── error.rs      # Error types and handling
└── storage.rs    # Token storage (in-memory, extend for persistence)
```

## Prerequisites

- Rust 1.70 or later
- TikTok Shop Partner account
- Registered app in TikTok Shop Partner Center

## Getting Started

### 1. Register Your Application

1. Go to [TikTok Shop Partner Center](https://partner.tiktokshop.com/)
2. Sign up and complete the developer onboarding
3. Create a new app (Public App recommended)
4. Enable API access
5. Set your redirect URI (e.g., `http://localhost:3000/auth/callback`)
6. Note down your **App Key** and **App Secret**

### 2. Setup Environment

Clone or create the project:

```bash
mkdir tiktok-shop-oauth
cd tiktok-shop-oauth
```

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Edit `.env` and add your credentials:

```env
TIKTOK_APP_KEY=your_app_key_here
TIKTOK_APP_SECRET=your_app_secret_here
TIKTOK_REDIRECT_URI=http://localhost:3000/auth/callback
HOST=127.0.0.1
PORT=3000
```

### 3. Build and Run

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run with release optimizations
cargo run --release
```

The server will start at `http://localhost:3000`

## Usage

### Web Interface

1. Open your browser and go to `http://localhost:3000`
2. Click "Authorize with TikTok Shop"
3. Log in with your TikTok Shop seller account
4. Authorize the application
5. You'll be redirected back with your access tokens and shop information

### API Endpoints

#### `GET /`
Home page with authorization button

#### `GET /auth/tiktok`
Initiates the OAuth flow by redirecting to TikTok Shop authorization page

#### `GET /auth/callback`
OAuth callback endpoint that receives the authorization code and exchanges it for tokens

**Query Parameters:**
- `code` - Authorization code from TikTok
- `state` - CSRF token for validation

#### `GET /auth/status`
Check current authorization status

**Response:**
```json
{
  "authorized": true,
  "access_token_expired": false,
  "refresh_token_expired": false,
  "expires_at": "2025-11-27T10:30:00Z",
  "shops": [
    {
      "shop_id": "123456",
      "shop_name": "My Shop",
      "region": "US",
      "cipher": "shop_cipher_string"
    }
  ]
}
```

#### `GET /auth/refresh`
Refresh the access token using the stored refresh token

**Response:**
```json
{
  "message": "Token refreshed successfully",
  "expires_in": 86400
}
```

## OAuth Flow Diagram

```
┌─────────┐                                    ┌──────────────┐
│ Browser │                                    │  Your App    │
└────┬────┘                                    └──────┬───────┘
     │                                                 │
     │  1. Click "Authorize"                          │
     ├────────────────────────────────────────────────>
     │                                                 │
     │  2. Redirect to TikTok Shop                    │
     │     with app_key, state, redirect_uri          │
     <─────────────────────────────────────────────────┤
     │                                                 │
┌────▼────┐                                           │
│ TikTok  │                                           │
│  Shop   │                                           │
└────┬────┘                                           │
     │                                                 │
     │  3. User logs in and authorizes                │
     │                                                 │
     │  4. Redirect to callback with code & state     │
     ├────────────────────────────────────────────────>
     │                                                 │
     │                                                 │  5. Exchange code
     │                                                 │     for access token
     │                                                 ├──────────────┐
     │                                                 │              │
     │                                                 <──────────────┘
     │                                                 │
     │  6. Display success with tokens                │
     <─────────────────────────────────────────────────┤
```

## Code Structure

### OAuth Client (`src/oauth.rs`)

The `TikTokShopOAuth` struct handles all OAuth-related operations:

```rust
let oauth = TikTokShopOAuth::new(app_key, app_secret, redirect_uri);

// Get authorization URL
let auth_url = oauth.get_authorization_url()?;

// Exchange code for token
let token = oauth.exchange_code_for_token(&code).await?;

// Get authorized shops
let shops = oauth.get_authorized_shops(&token.access_token).await?;

// Refresh token
let new_token = oauth.refresh_access_token(&refresh_token).await?;
```

### Token Storage (`src/storage.rs`)

In-memory token storage with expiry tracking:

```rust
let mut storage = TokenStorage::new();
storage.store(token_info);

if storage.is_access_token_valid() {
    // Use the token
} else {
    // Refresh the token
}
```

### Error Handling (`src/error.rs`)

Comprehensive error types with automatic HTTP response conversion:

```rust
pub enum AppError {
    InvalidState,
    NoTokenStored,
    TokenExchangeFailed(String),
    ApiError(i32, String),
    // ... more error types
}
```

## TikTok Shop API Endpoints

The implementation uses the following TikTok Shop API endpoints:

- **Authorization**: `https://services.tiktokshop.com/open/authorize`
- **Token Exchange**: `https://auth.tiktok-shops.com/api/v2/token/get`
- **Token Refresh**: `https://auth.tiktok-shops.com/api/v2/token/refresh`
- **Get Shops**: `https://auth.tiktok-shops.com/api/v2/shops/get_authorized`

## Security Features

### CSRF Protection
- Generates random state tokens for each authorization request
- Validates state on callback to prevent CSRF attacks
- Stores states with 10-minute expiration
- Single-use state tokens

### Token Security
- Tokens stored server-side only
- Automatic expiry tracking
- Secure token refresh mechanism

## Production Considerations

### Token Storage
The current implementation uses in-memory storage. For production:

1. **Database Storage**: Store tokens in PostgreSQL/MySQL:
   ```rust
   // Example with SQLx
   sqlx::query!(
       "INSERT INTO tokens (access_token, refresh_token, expires_at) VALUES ($1, $2, $3)",
       token.access_token,
       token.refresh_token,
       token.expires_at
   )
   .execute(&pool)
   .await?;
   ```

2. **Redis Cache**: For fast access and automatic expiry:
   ```rust
   redis::cmd("SETEX")
       .arg(format!("token:{}", shop_id))
       .arg(token.expires_in)
       .arg(token.access_token)
       .query_async(&mut con)
       .await?;
   ```

### Encryption
Encrypt sensitive tokens at rest:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

// Encrypt token before storage
let encrypted = cipher.encrypt(nonce, token.as_bytes())?;
```

### HTTPS
Always use HTTPS in production:
- Update redirect URI to use `https://`
- Use proper TLS certificates
- Consider using a reverse proxy (nginx, Caddy)

### Environment Variables
Never commit `.env` files. Use:
- Docker secrets
- Kubernetes secrets
- AWS Secrets Manager
- HashiCorp Vault

## Testing

Run the included tests:

```bash
cargo test
```

## Extending the Implementation

### Add Database Persistence

```rust
// src/storage.rs
use sqlx::PgPool;

pub struct DatabaseTokenStorage {
    pool: PgPool,
}

impl DatabaseTokenStorage {
    pub async fn store(&self, shop_id: &str, token: TokenInfo) -> Result<()> {
        sqlx::query!(
            "INSERT INTO shop_tokens (shop_id, access_token, refresh_token, expires_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (shop_id) DO UPDATE SET
                access_token = $2,
                refresh_token = $3,
                expires_at = $4",
            shop_id,
            token.access_token,
            token.refresh_token,
            token.expires_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

### Add Webhook Support

```rust
// src/webhook.rs
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn verify_webhook_signature(
    body: &[u8],
    signature: &str,
    secret: &str,
) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());
    expected == signature
}
```

## Troubleshooting

### "Authorization code is expired"
- Authorization codes are single-use and expire quickly (usually 10 minutes)
- Don't refresh the callback page
- Complete the token exchange immediately after receiving the code

### "Invalid redirect_uri"
- Ensure the redirect URI in `.env` exactly matches the one registered in TikTok Shop Partner Center
- Include the protocol (`http://` or `https://`)
- Check for trailing slashes

### "Invalid state parameter"
- CSRF state tokens expire after 10 minutes
- Don't share authorization URLs
- Complete the flow quickly after initiating

### Token refresh fails
- Check if refresh token has expired (typically 30 days)
- Verify app credentials are correct
- Ensure you're using the latest refresh token (they rotate on refresh)

## Dependencies

- `axum` - Web framework
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` - Serialization
- `chrono` - Date/time handling
- `rand` - Random generation for CSRF tokens

## Resources

- [TikTok Shop Partner Center](https://partner.tiktokshop.com/)
- [TikTok Shop API Documentation](https://partner.tiktokshop.com/docv2/page/authorization-overview-202407)
- [OAuth 2.0 RFC](https://datatracker.ietf.org/doc/html/rfc6749)

## License

MIT License - feel free to use this implementation in your projects.

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## Support

For issues related to:
- **This implementation**: Open an issue in this repository
- **TikTok Shop API**: Contact TikTok Shop Partner Support
- **Your TikTok Shop account**: Check the Partner Center

---

Built with ❤️ using Rust
