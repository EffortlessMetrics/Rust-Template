# http-middleware

Shared HTTP middleware for cross-cutting concerns.

## Purpose

This crate provides reusable middleware for common HTTP concerns:
- Request ID correlation for distributed tracing
- CORS (Cross-Origin Resource Sharing) protection
- Security headers (CSP, XSS protection, clickjacking prevention)

## Design Philosophy

Middleware should be:
- **Composable**: Easy to combine and layer
- **Configurable**: Clear configuration structs
- **Framework-agnostic**: Where possible, avoid framework-specific state
- **Observable**: Integrate with tracing

## Usage

### Request ID Middleware

```rust
use axum::Router;
use http_middleware::request_id::request_id_layer;

let app = Router::new()
    .route("/api/endpoint", get(handler))
    .layer(request_id_layer());
```

Access the request ID in handlers:

```rust
use axum::extract::Extension;
use http_middleware::request_id::RequestId;

async fn handler(Extension(request_id): Extension<RequestId>) -> impl IntoResponse {
    info!(request_id = %request_id, "Processing request");
    // ... handler logic
}
```

### CORS Middleware

```rust
use http_middleware::{cors::CorsConfig, cors::cors_layer};

let config = CorsConfig::development();
let app = Router::new()
    .route("/api/endpoint", get(handler))
    .layer(cors_layer(config));
```

Production configuration:

```rust
let config = CorsConfig::production(vec![
    "https://example.com".to_string(),
    "https://api.example.com".to_string(),
]);
```

### Security Headers Middleware

```rust
use http_middleware::{security_headers::SecurityHeadersConfig, security_headers::security_headers_layer};

let config = SecurityHeadersConfig::production();
let app = Router::new()
    .route("/api/endpoint", get(handler))
    .layer(security_headers_layer(config.cache()));
```

Development configuration (more permissive):

```rust
let config = SecurityHeadersConfig::development();
```

### Combining Middleware

```rust
use axum::Router;
use http_middleware::{
    cors::cors_layer,
    request_id::request_id_layer,
    security_headers::security_headers_layer,
};

let app = Router::new()
    .route("/api/endpoint", get(handler))
    .layer(security_headers_layer(SecurityHeadersConfig::production().cache()))
    .layer(cors_layer(CorsConfig::development()))
    .layer(request_id_layer());
```

## Configuration

### CORS Configuration

| Field | Type | Default | Description |
|--------|-------|----------|-------------|
| `allowed_origins` | `Vec<String>` | `["http://localhost:3000", "http://localhost:8080"]` | Allowed origins |
| `allowed_methods` | `Vec<String>` | `["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"]` | Allowed HTTP methods |
| `allowed_headers` | `Vec<String>` | `["authorization", "content-type", "x-request-id", ...]` | Allowed request headers |
| `exposed_headers` | `Vec<String>` | `["x-request-id"]` | Headers exposed to clients |
| `allow_credentials` | `bool` | `false` | Allow credentials |
| `max_age` | `Option<u64>` | `Some(86400)` | Preflight cache age (seconds) |
| `enabled` | `bool` | `true` | Enable CORS |

### Security Headers Configuration

| Field | Type | Default | Description |
|--------|-------|----------|-------------|
| `content_security_policy` | `Option<String>` | CSP string | Content Security Policy |
| `x_frame_options` | `String` | `"DENY"` | Clickjacking protection |
| `x_content_type_options` | `String` | `"nosniff"` | MIME sniffing prevention |
| `x_xss_protection` | `String` | `"1; mode=block"` | XSS filtering |
| `strict_transport_security` | `Option<String>` | HSTS string | HTTPS enforcement |
| `referrer_policy` | `String` | `"strict-origin-when-cross-origin"` | Referrer policy |
| `permissions_policy` | `Option<String>` | Permissions string | Feature permissions |
| `cross_origin_embedder_policy` | `Option<String>` | COEP string | COEP header |
| `cross_origin_opener_policy` | `Option<String>` | COOP string | COOP header |
| `cross_origin_resource_policy` | `String` | `"same-origin"` | CORP header |
| `enabled` | `bool` | `true` | Enable headers |

## Features

All middleware is framework-agnostic where possible, with axum integration provided.

## License

Apache-2.0 OR MIT
