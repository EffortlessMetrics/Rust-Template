# gov-http-core

Shared foundation for gov-http subrouter crates.

## Purpose

This crate provides common router glue, extractors, error mapping, and shared types used by all gov-http-* subrouter crates. It is the foundation that enables modular, independent subrouter crates while maintaining consistency.

## Features

- **Shared error types** - `PlatformError` and `ErrorResponse` for consistent error handling
- **Common extractors** - `RequestId` extractor for request correlation
- **State trait** - `PlatformState` trait for dependency injection
- **Health endpoints** - `/health` and `/status` endpoints for monitoring

## Dependencies

- `axum` - Web framework
- `http` - HTTP types
- `http-errors` - Error types and mapping (with axum feature)
- `platform-contract` - Platform contract types
- `gov-model` - Governance domain model

## Usage

```rust
use gov_http_core::{PlatformState, PlatformError, RequestId};
use axum::{Router, routing::get};

// Define your state implementing PlatformState
struct MyState {
    // ... your fields
}

impl PlatformState for MyState {
    fn context(&self) -> &gov_model::RepoContext {
        // ...
    }

    fn governance_repo(&self) -> std::sync::Arc<dyn gov_model::GovernanceRepository> {
        // ...
    }
}

// Use RequestId in handlers
async fn my_handler(
    RequestId(request_id): RequestId,
) -> Result<Json<YourResponse>, PlatformError> {
    // Use request_id for tracing
    Ok(Json(response))
}

// Compose with health router
let app = Router::new()
    .merge(gov_http_core::health_router())
    .route("/my-endpoint", get(my_handler))
    .with_state(MyState::new(...));
```

## Design Principles

- **Minimal dependencies** - Only essential HTTP and model types
- **No domain-specific handlers** - Those belong in subrouter crates
- **Trait-based state** - Use traits for state requirements, not concrete types
- **Re-export compatibility** - Compatible types from http-errors
