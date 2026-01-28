# http-core

HTTP core foundation for the Rust-as-Spec platform.

## Purpose

This crate provides shared HTTP infrastructure used across all `http-*` crates. It contains:

- **App state traits**: `AppState` and `PlatformState` traits for dependency injection
- **Router composition helpers**: `base_router()` for common endpoints
- **Shared handlers**: `health()` and `version()` endpoints
- **Shutdown signal handling**: `shutdown_signal()` for graceful server shutdown
- **Workspace root resolution**: `resolve_workspace_root()` for path resolution

## Design Philosophy

- **Minimal dependencies**: Only essential HTTP and platform deps (axum, http, http-errors, platform-contract)
- **Trait-based state**: Use traits for state requirements, not concrete types
- **Shared foundation**: Common types and helpers used across all http-* crates

## Dependencies

- `axum` - HTTP web framework
- `http` - HTTP types
- `http-errors` - Error types with axum feature
- `platform-contract` - Platform contract types
- `prometheus` - Metrics support
- `serde`, `serde_json` - Serialization
- `tokio` - Async runtime
- `tracing` - Structured logging
- `uuid` - UUID generation

## Usage

```rust
use http_core::{AppState, router, shutdown_signal};

let state = AppState::new(repo)?;
let app = router(state);

// Serve with graceful shutdown
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

## Public API

### Traits

- `AppState` - Core application state trait
- `PlatformState` - Platform state trait for governance integration

### Functions

- `base_router()` - Create router with common endpoints (`/health`, `/version`)
- `health()` - Health check endpoint handler
- `version()` - Version information endpoint handler
- `shutdown_signal()` - Graceful shutdown signal future
- `resolve_workspace_root()` - Resolve workspace root from environment or CARGO_MANIFEST_DIR

### Re-exports

- `http_errors::{AppError, ErrorCode, ErrorSummary, get_error_summary, RequestId}`
- `platform_contract` types for platform status and error responses

## License

Internal crate (publish = false)
