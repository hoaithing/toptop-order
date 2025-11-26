# TikTok Shop Order API Client

A Rust CLI application for fetching TikTok Shop orders using the TikTok Shop API.

## ✅ Completed Features

- ✅ File-based token loading from `token.json`
- ✅ Shop cipher and shop ID configuration via `.env`
- ✅ Signed API requests with HMAC-SHA256
- ✅ Complete order data structures
- ✅ CLI interface for fetching orders
- ✅ Token expiration checking
- ✅ Request headers (`x-tts-access-token`, `Content-Type`)
- ✅ Removed web server and OAuth flow (simplified to CLI only)

## Quick Start

### 1. Setup Environment Variables

Create/edit `.env`:
```env
TIKTOK_APP_KEY=your_app_key
TIKTOK_APP_SECRET=your_app_secret
TIKTOK_SHOP_CIPHER=your_shop_cipher
TIKTOK_SHOP_ID=your_shop_id
TIKTOK_TOKEN_FILE=token.json
```

### 2. Provide Token File

Place your `token.json` file in the project root:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "access_token": "ROW_...",
    "access_token_expire_in": 1764759323,
    "refresh_token": "ROW_...",
    "refresh_token_expire_in": 4875145688,
    "seller_name": "Your Shop Name",
    "seller_base_region": "VN",
    "granted_scopes": ["seller.order.info", "seller.authorization.info"]
  }
}
```

### 3. Run the CLI

```bash
cargo run --bin cli
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `TIKTOK_APP_KEY` | Your TikTok Shop app key | Yes |
| `TIKTOK_APP_SECRET` | Your TikTok Shop app secret | Yes |
| `TIKTOK_SHOP_CIPHER` | Shop cipher for API requests | Optional* |
| `TIKTOK_SHOP_ID` | Shop ID | Optional |
| `TIKTOK_TOKEN_FILE` | Path to token JSON file | No (default: token.json) |

*Note: shop_cipher may be required for some API endpoints

## Project Structure

```
src/
├── bin/
│   └── cli.rs              # Main CLI application (simplified entry point)
├── config.rs               # Configuration from environment variables
├── error.rs                # Error types
├── order.rs                # Order API client and data structures
├── requests.rs             # Signed API request client
├── storage.rs              # Token persistence (file-based)
└── lib.rs                  # Library exports

examples/
├── check_token.rs          # Check token expiration
├── debug_signature.rs      # Debug signature generation
└── test_*.rs               # Various API test utilities
```

## API Implementation

### Signature Generation

```rust
sign_string = app_key + timestamp + access_token + shop_cipher + path + body
signature = HMAC-SHA256(app_secret, sign_string)
```

### Request Format

**POST /api/orders/search**

Query Parameters:
- `app_key`
- `timestamp`
- `access_token`
- `shop_cipher` (optional)
- `sign` (HMAC-SHA256 signature)

Headers:
- `Content-Type: application/json`
- `x-tts-access-token: {access_token}`

Request Body:
```json
{
  "page_size": 10,
  "order_status": 111,
  "create_time_ge": 1234567890,
  "create_time_lt": 1234567890
}
```

## Order Data Structures

### GetOrderListRequest

Builder pattern for constructing order list requests:

```rust
let request = GetOrderListRequest::new()
    .with_status(OrderStatus::AwaitingShipment)
    .with_page_size(20)
    .with_create_time_range(start_timestamp, end_timestamp)
    .sort_by("create_time".to_string(), SortOrder::Descending);
```

### Order Status Codes

| Code | Status |
|------|--------|
| 100 | Unpaid |
| 111 | Awaiting Shipment |
| 112 | Awaiting Collection |
| 114 | Partially Shipped |
| 121 | In Transit |
| 122 | Delivered |
| 130 | Completed |
| 140 | Cancelled |

### Order Response Fields

- `id` - Order ID
- `status` - Order status code
- `create_time` / `update_time` - Unix timestamps
- `payment` - Payment info (total, currency, fees)
- `recipient_address` - Shipping address
- `item_list` - List of ordered items
- `buyer` - Buyer information
- `delivery` - Tracking information

## Development Commands

### Build
```bash
cargo build
cargo build --release
```

### Run
```bash
cargo run --bin cli
```

### Test
```bash
cargo test
```

### Debug Tools

Check token expiration:
```bash
cargo run --example check_token
```

Debug signature formats:
```bash
cargo run --example debug_signature
```

## ⚠️ Current Known Issue

**API Signature Validation Error (Code 106001)**

The TikTok Shop API returns "Invalid credentials. The 'sign' query parameter is invalid" for all requests.

**What's Been Tested:**
- ✅ Token loading and parsing
- ✅ Token expiration checking
- ✅ Request structure and headers
- ✅ Multiple signature algorithms
- ❌ Signature validation with TikTok API

**Possible Causes:**
1. Signature algorithm doesn't match TikTok's exact requirements
2. Missing required parameters in signature string
3. API version mismatch (using 202309)
4. Parameter encoding issues

**Next Steps:**
1. Verify exact signature format from official TikTok Shop API documentation
2. Check if API version or endpoint has changed
3. Contact TikTok Shop support with error request_id
4. Compare with working implementation if available

## Architecture Decisions

### Why CLI Instead of Web Server?

The original implementation included a web server for OAuth flow. We simplified this because:
- Token can be obtained externally and provided via `token.json`
- No need for callback handling
- Simpler deployment and usage
- Focused on order fetching functionality

### File-Based Token Storage

Tokens are saved to `tiktok_tokens.json` (if using the conversion tool) or loaded directly from `token.json`. This allows:
- Persistence across restarts
- Easy token management
- No database required

## Dependencies

```toml
[dependencies]
reqwest = "0.11"        # HTTP client
tokio = "1"             # Async runtime
serde = "1.0"           # Serialization
serde_json = "1.0"      # JSON
hmac = "0.12"           # HMAC
sha2 = "0.10"           # SHA256
hex = "0.4"             # Hex encoding
chrono = "0.4"          # Timestamps
tracing = "0.1"         # Logging
dotenvy = "0.15"        # .env loading
```

## Troubleshooting

### Token Expired
Run `cargo run --example check_token` to verify token expiration.

### Missing Configuration
Ensure all required variables are set in `.env`.

### Signature Errors
This is a known issue - see "Current Known Issue" section above.

## Contributing

When contributing, please:
1. Run `cargo fmt` before committing
2. Ensure `cargo test` passes
3. Update documentation for new features

## Resources

- [TikTok Shop API Documentation](https://partner.tiktokshop.com/docv2/)
- [API Signature Guide](https://partner.tiktokshop.com/docv2/page/sign-your-api-request)
- [Order API Reference](https://partner.tiktokshop.com/docv2/page/get-order-list-202309)

## License

MIT
