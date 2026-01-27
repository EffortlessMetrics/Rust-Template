# http-platform

HTTP handlers for `/platform/*` endpoints.

## Purpose

This crate implements the platform API including:

- **Platform status endpoint** (`/platform/status`) - Service health and governance metrics
- **IDP snapshot endpoint** (`/platform/idp/snapshot`) - Machine-readable contract for IDPs
- **UI routes** (`/`, `/ui`, `/ui/graph`, `/ui/flows`, `/ui/coverage`) - HTML-based platform visualization
- **Debug info endpoint** (`/platform/debug/info`) - Kernel and template version information
- **Gov-http integration** - Reuses gov-http handlers for governance-generic endpoints

## Design Philosophy

- **Platform-focused**: Only platform-related handlers and UI
- **Contract-based**: Uses `platform-contract` types for stable API
- **Gov-http integration**: Reuses gov-http handlers for governance endpoints
- **HTML UI**: Uses maud for type-safe HTML templates

## Dependencies

- `axum` - HTTP web framework
- `http` - HTTP types
- `http-errors` - Error types with axum feature
- `platform-contract` - Platform contract types
- `http-core` - Core HTTP types
- `gov-http` - Governance HTTP handlers
- `spec-runtime` - Spec loading and runtime
- `anyhow`, `serde`, `serde_json`, `serde_yaml` - Serialization
- `tracing` - Structured logging

## Usage

```rust
use http_platform::{router, ui_router, PlatformState};

let app = Router::new()
    .nest("/platform", router(state))
    .merge(ui_router(state));
```

## Public API

### Traits

- `PlatformState` - Platform state trait for handlers
- `PlatformAuthConfig` - Platform auth config trait

### Functions

- `router()` - Create platform API router (mounted at `/platform`)
- `ui_router()` - Create UI routes (mounted at root)
- `config_summary()` - Get config summary from state

### Handlers

- `debug_info()` - Platform debug info endpoint
- `get_status()` - Platform status endpoint

### Modules

- `idp` - IDP snapshot endpoint
- `ui` - UI routes (dashboard, graph, flows, coverage)

### Re-exports

- `gov-http` types (CoverageDetail, CoverageResponse, DocInfoWithHealth, etc.)
- `IdpSnapshot` - IDP snapshot contract type

## License

Internal crate (publish = false)
