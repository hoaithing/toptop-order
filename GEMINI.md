# GEMINI.md

## Project Overview

This project is a Rust application for interacting with the TikTok Shop API. It consists of two main parts:

1.  A **CLI application** (`src/bin/cli.rs`) for fetching orders from the TikTok Shop API.
2.  A **web server** (`src/main.rs`) built with the Axum framework that handles OAuth 2.0 token management (refreshing tokens and checking their status).

The project uses `reqwest` for making HTTP requests, `serde` for JSON serialization/deserialization, and `hmac` for signing API requests. Configuration is managed through a `.env` file.

## Building and Running

### Prerequisites

-   Rust and Cargo
-   A `.env` file with the necessary TikTok Shop API credentials. You can use `.env.example` as a template.

### Running the CLI

To run the CLI application for fetching orders, use the following command:

```bash
cargo run --bin cli
```

### Running the Web Server

To run the web server for handling OAuth token operations, use the following command:

```bash
cargo run --bin tiktok-shop-oauth
```

The web server will start on `0.0.0.0:3000`.

## Testing

To run the tests for this project, use:

```bash
cargo test
```

## Development Conventions

-   **Formatting:** The project uses `rustfmt` for code formatting. Please run `cargo fmt` before committing changes.
-   **Linting:** The project uses `clippy` for linting. Please run `cargo clippy` to check for common mistakes and style issues.
-   **Error Handling:** The project uses the `thiserror` crate for custom error types.
-   **Configuration:** Application configuration is loaded from a `.env` file using the `dotenvy` crate.
-   **Known Issue:** The `README.md` mentions a known issue with API signature validation. Please refer to the `README.md` for more details.
