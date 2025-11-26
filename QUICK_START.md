# Quick Start Guide - TikTok Shop OAuth in Rust

Get up and running in 5 minutes!

## Prerequisites

- **Rust** (1.70+) - Install from [rustup.rs](https://rustup.rs/)
- **TikTok Shop Partner Account** - Sign up at [partner.tiktokshop.com](https://partner.tiktokshop.com/)

## Step 1: Get Your TikTok Shop Credentials

1. Go to [TikTok Shop Partner Center](https://partner.tiktokshop.com/)
2. Navigate to **App & Service**
3. Create a new **Public App** (or use existing)
4. Enable **API** access
5. Set redirect URI: `http://localhost:3000/auth/callback`
6. Note your **App Key** and **App Secret**

## Step 2: Configure the Application

```bash
# Copy environment template
cp .env.example .env

# Edit .env and add your credentials
nano .env  # or use your favorite editor
```

Update these values in `.env`:
```env
TIKTOK_APP_KEY=your_actual_app_key
TIKTOK_APP_SECRET=your_actual_app_secret
TIKTOK_REDIRECT_URI=http://localhost:3000/auth/callback
```

## Step 3: Run the Application

### Option A: Using the Quick Start Script

```bash
chmod +x quickstart.sh
./quickstart.sh
```

### Option B: Manual Run

```bash
# Build and run
cargo run --release
```

The server will start at `http://localhost:3000`

## Step 4: Test the OAuth Flow

1. Open your browser: `http://localhost:3000`
2. Click **"Authorize with TikTok Shop"**
3. Log in with your TikTok Shop seller account
4. Grant permissions to your app
5. You'll be redirected back with your tokens!

## What You'll See

After successful authorization:
- ✅ Access Token (valid for 24 hours)
- ✅ Refresh Token (valid for 30 days)
- ✅ List of authorized shops with shop IDs and ciphers

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Home page with authorization button |
| `GET /auth/tiktok` | Start OAuth flow |
| `GET /auth/callback` | OAuth callback handler |
| `GET /auth/status` | Check authorization status |
| `GET /auth/refresh` | Refresh access token |

## Common Issues

### "Authorization code expired"
- Don't refresh the callback page
- Complete authorization flow quickly

### "Invalid redirect_uri"
- Ensure redirect URI in `.env` matches Partner Center
- Include protocol: `http://` or `https://`

### "TIKTOK_APP_KEY not set"
- Make sure `.env` file exists
- Check environment variables are properly set

## Next Steps

### Deploy to Production

1. **Use HTTPS**: Update redirect URI to use `https://`
2. **Persistent Storage**: Replace in-memory storage with database
3. **Environment Variables**: Use secrets management (AWS Secrets Manager, etc.)

### Add Features

- **Database Storage**: See `README.md` for PostgreSQL/Redis integration
- **Webhooks**: Handle real-time events from TikTok Shop
- **API Calls**: Use access tokens to call TikTok Shop API endpoints

## Using as a Library

```rust
use tiktok_shop_oauth::oauth::TikTokShopOAuth;

let oauth = TikTokShopOAuth::new(
    app_key,
    app_secret,
    redirect_uri,
);

// Generate authorization URL
let auth_url = oauth.get_authorization_url()?;

// Exchange code for token
let token = oauth.exchange_code_for_token(&code).await?;

// Get authorized shops
let shops = oauth.get_authorized_shops(&token.access_token).await?;
```

## Docker Deployment

```bash
# Build and run with Docker
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Development Commands

```bash
make build      # Build release version
make test       # Run tests
make fmt        # Format code
make lint       # Run clippy
make dev        # Run with auto-reload
```

## Support

- **Documentation**: See `README.md` for comprehensive docs
- **Examples**: Check `examples/basic_usage.rs`
- **Issues**: Check TikTok Shop API documentation

## Resources

- [TikTok Shop Partner Center](https://partner.tiktokshop.com/)
- [TikTok Shop API Docs](https://partner.tiktokshop.com/docv2)
- [Rust Axum Framework](https://github.com/tokio-rs/axum)

---

**That's it!** You now have a working TikTok Shop OAuth implementation in Rust. 🎉
