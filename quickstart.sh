#!/bin/bash

# Quick Start Script for TikTok Shop OAuth

set -e

echo "=== TikTok Shop OAuth - Quick Start ==="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed."
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust is installed"

# Check if .env file exists
if [ ! -f .env ]; then
    echo ""
    echo "Creating .env file from template..."
    cp .env.example .env
    echo "✓ Created .env file"
    echo ""
    echo "⚠️  IMPORTANT: Please edit .env and add your TikTok Shop credentials:"
    echo "   - TIKTOK_APP_KEY"
    echo "   - TIKTOK_APP_SECRET"
    echo "   - TIKTOK_REDIRECT_URI"
    echo ""
    read -p "Press Enter when you've updated the .env file..."
else
    echo "✓ .env file exists"
fi

# Check if credentials are set
source .env
if [ "$TIKTOK_APP_KEY" = "your_app_key_here" ] || [ -z "$TIKTOK_APP_KEY" ]; then
    echo ""
    echo "⚠️  Warning: TIKTOK_APP_KEY is not set properly in .env"
    echo "Please update your credentials before running the application."
    exit 1
fi

echo "✓ Credentials configured"
echo ""

# Build the project
echo "Building the project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
    echo ""
    echo "=== Starting the application ==="
    echo ""
    echo "The server will start at: http://localhost:3000"
    echo ""
    echo "To test the OAuth flow:"
    echo "1. Open http://localhost:3000 in your browser"
    echo "2. Click 'Authorize with TikTok Shop'"
    echo "3. Log in with your TikTok Shop seller account"
    echo "4. Authorize the application"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    cargo run --release
else
    echo "❌ Build failed"
    exit 1
fi
