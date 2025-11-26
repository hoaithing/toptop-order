# TikTok Shop OAuth - Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     TikTok Shop OAuth System                │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐        ┌──────────────┐        ┌──────────────┐
│   Browser    │◄──────►│  Rust App    │◄──────►│  TikTok API  │
│   (User)     │        │   (Axum)     │        │     Shop     │
└──────────────┘        └──────────────┘        └──────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │    Token     │
                        │   Storage    │
                        └──────────────┘
```

## Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Rust Application                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │              HTTP Layer (Axum)                     │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐           │    │
│  │  │   Home   │ │   Auth   │ │ Callback │  Routes   │    │
│  │  └──────────┘ └──────────┘ └──────────┘           │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │           Business Logic Layer                     │    │
│  │                                                     │    │
│  │  ┌──────────────────────────────────────────┐     │    │
│  │  │     TikTokShopOAuth Client               │     │    │
│  │  │  - Authorization URL generation          │     │    │
│  │  │  - CSRF state management                 │     │    │
│  │  │  - Token exchange                        │     │    │
│  │  │  - Token refresh                         │     │    │
│  │  │  - Shop information retrieval            │     │    │
│  │  └──────────────────────────────────────────┘     │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │            Data Layer                              │    │
│  │  ┌──────────────┐  ┌──────────────┐               │    │
│  │  │   Config     │  │   Storage    │               │    │
│  │  │   Manager    │  │   (Tokens)   │               │    │
│  │  └──────────────┘  └──────────────┘               │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## OAuth Flow Sequence

```
┌────────┐          ┌────────┐          ┌────────┐          ┌────────┐
│Browser │          │  App   │          │TikTok  │          │Storage │
└───┬────┘          └───┬────┘          └───┬────┘          └───┬────┘
    │                   │                   │                   │
    │ 1. GET /          │                   │                   │
    ├──────────────────►│                   │                   │
    │                   │                   │                   │
    │ 2. Show UI        │                   │                   │
    │◄──────────────────┤                   │                   │
    │                   │                   │                   │
    │ 3. Click Auth     │                   │                   │
    ├──────────────────►│                   │                   │
    │                   │                   │                   │
    │                   │ 4. Generate State │                   │
    │                   ├───────────────────┼──────────────────►│
    │                   │                   │                   │
    │ 5. Redirect to    │                   │                   │
    │    TikTok         │                   │                   │
    │◄──────────────────┤                   │                   │
    │                   │                   │                   │
    │ 6. Authorization  │                   │                   │
    ├───────────────────┼──────────────────►│                   │
    │                   │                   │                   │
    │                   │                   │ 7. User Login &   │
    │                   │                   │    Authorize      │
    │                   │                   │                   │
    │ 8. Redirect with  │                   │                   │
    │    auth code      │                   │                   │
    │◄──────────────────┼───────────────────┤                   │
    ├──────────────────►│                   │                   │
    │                   │                   │                   │
    │                   │ 9. Verify State   │                   │
    │                   ├───────────────────┼──────────────────►│
    │                   │                   │                   │
    │                   │ 10. Exchange Code │                   │
    │                   ├──────────────────►│                   │
    │                   │                   │                   │
    │                   │ 11. Access Token  │                   │
    │                   │◄──────────────────┤                   │
    │                   │                   │                   │
    │                   │ 12. Get Shops     │                   │
    │                   ├──────────────────►│                   │
    │                   │                   │                   │
    │                   │ 13. Shop List     │                   │
    │                   │◄──────────────────┤                   │
    │                   │                   │                   │
    │                   │ 14. Store Token   │                   │
    │                   ├───────────────────┼──────────────────►│
    │                   │                   │                   │
    │ 15. Show Success  │                   │                   │
    │◄──────────────────┤                   │                   │
    │                   │                   │                   │
```

## Token Refresh Flow

```
┌────────┐          ┌────────┐          ┌────────┐          ┌────────┐
│ Client │          │  App   │          │TikTok  │          │Storage │
└───┬────┘          └───┬────┘          └───┬────┘          └───┬────┘
    │                   │                   │                   │
    │ GET /auth/refresh │                   │                   │
    ├──────────────────►│                   │                   │
    │                   │                   │                   │
    │                   │ Check Token       │                   │
    │                   ├───────────────────┼──────────────────►│
    │                   │                   │                   │
    │                   │ Token Expired?    │                   │
    │                   │◄──────────────────┼───────────────────┤
    │                   │                   │                   │
    │                   │ Refresh Request   │                   │
    │                   ├──────────────────►│                   │
    │                   │                   │                   │
    │                   │ New Access Token  │                   │
    │                   │◄──────────────────┤                   │
    │                   │                   │                   │
    │                   │ Update Storage    │                   │
    │                   ├───────────────────┼──────────────────►│
    │                   │                   │                   │
    │ Success Response  │                   │                   │
    │◄──────────────────┤                   │                   │
    │                   │                   │                   │
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         Request Flow                         │
└─────────────────────────────────────────────────────────────┘

HTTP Request
    │
    ▼
┌───────────────┐
│ Axum Router   │ ──► Route matching
└───────┬───────┘
        │
        ▼
┌───────────────┐
│   Handler     │ ──► Extract state & params
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ OAuth Client  │ ──► Make API calls
└───────┬───────┘
        │
        ▼
┌───────────────┐
│   Storage     │ ──► Persist/retrieve data
└───────┬───────┘
        │
        ▼
┌───────────────┐
│   Response    │ ──► HTML or JSON
└───────────────┘
```

## Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│                       Security Stack                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │           Transport Layer (HTTPS)                  │    │
│  │  - TLS encryption for all communications           │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │        CSRF Protection (State Token)               │    │
│  │  - Random state generation                         │    │
│  │  - State verification on callback                  │    │
│  │  - Single-use tokens                               │    │
│  │  - 10-minute expiration                            │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │         Token Management                           │    │
│  │  - Server-side only storage                        │    │
│  │  - Automatic expiry tracking                       │    │
│  │  - Secure refresh mechanism                        │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │        Error Handling                              │    │
│  │  - Type-safe errors                                │    │
│  │  - No sensitive data in logs                       │    │
│  │  - Graceful failure modes                          │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Module Dependencies

```
main.rs
  ├── config.rs      (Configuration)
  ├── oauth.rs       (OAuth client)
  │   └── reqwest    (HTTP client)
  ├── storage.rs     (Token storage)
  │   └── chrono     (Timestamps)
  └── error.rs       (Error types)
      └── thiserror  (Error derive)

Axum Framework
  ├── Tower          (Middleware)
  ├── Tokio          (Async runtime)
  └── Hyper          (HTTP server)
```

## Deployment Architecture

### Development
```
┌────────────────┐
│   localhost    │
│   :3000        │
│                │
│  ┌──────────┐  │
│  │Rust App  │  │
│  │(Axum)    │  │
│  └──────────┘  │
│       │        │
│  ┌──────────┐  │
│  │In-Memory │  │
│  │ Storage  │  │
│  └──────────┘  │
└────────────────┘
```

### Production
```
┌─────────────────────────────────────────┐
│           Load Balancer (HTTPS)         │
└──────────────┬──────────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────┐           ┌───▼────┐
│App     │           │App     │
│Instance│           │Instance│
│  1     │           │  2     │
└───┬────┘           └───┬────┘
    │                     │
    └──────────┬──────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────────┐      ┌────▼─────┐
│PostgreSQL  │      │  Redis   │
│(Tokens DB) │      │ (Cache)  │
└────────────┘      └──────────┘
```

## Extension Points

```
Current Implementation
        │
        ├── Database Storage (PostgreSQL/Redis)
        │
        ├── Webhook Handling
        │   └── Event processing
        │       └── Order updates
        │       └── Inventory changes
        │
        ├── API Client
        │   └── Product management
        │   └── Order processing
        │   └── Analytics
        │
        └── Monitoring
            └── Metrics (Prometheus)
            └── Tracing (OpenTelemetry)
            └── Logging (structured)
```

## Performance Considerations

```
┌─────────────────────────────────────────┐
│          Optimization Strategies         │
├─────────────────────────────────────────┤
│                                         │
│  Connection Pooling                     │
│  ├── HTTP client reuse                  │
│  └── Database connections               │
│                                         │
│  Caching                                │
│  ├── Token caching (Redis)              │
│  └── Shop info caching                  │
│                                         │
│  Async I/O                              │
│  ├── Tokio runtime                      │
│  └── Non-blocking operations            │
│                                         │
│  Rate Limiting                          │
│  ├── API call throttling                │
│  └── Token refresh backoff              │
│                                         │
└─────────────────────────────────────────┘
```

---

This architecture provides a solid foundation for building TikTok Shop integrations while maintaining security, scalability, and maintainability.
