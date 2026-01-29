# gov-http-forks

Fork registry endpoints for tracking template forks.

## Purpose

This crate provides HTTP endpoints for managing fork registry entries. Forks represent known deployments/customizations of the template, tracking their kernel versions, status, and relationships.

## Endpoints

- `GET /forks` - List all forks (returns summaries)
- `GET /forks/{name}` - Get a specific fork by ID or name (returns full entry)

## Dependencies

- `axum` - Web framework
- `http` - HTTP types
- `http-errors` - Error types and mapping (with axum feature)
- `platform-contract` - Platform contract types
- `gov-model` - Governance domain model
- `gov-http-core` - Shared router foundation
- `serde_yaml` - YAML parsing

## Usage

```rust
use gov_http_forks::router;
use axum::Router;
use gov_http_core::PlatformState;

// Compose the forks router
let app = Router::new()
    .merge(router::<MyState>())
    .with_state(my_state);
```

## Data Types

### ForkEntry

Full fork registry entry with all metadata:
- `id`: Unique identifier (e.g., "FORK-TEST-001")
- `name`: Human-readable fork name
- `domain`: Domain/organization (e.g., "rust-sdk")
- `kernel_version`: Template kernel version
- `status`: Current status (e.g., "active", "archived")
- `url`: Optional repository URL
- `maintainer`: Optional maintainer information
- `forked_at`: Optional fork date
- `last_synced`: Optional last sync date
- `features`: List of features
- `pain_points`: List of known pain points
- `notes`: Optional notes
- `related_items`: Related issues, ADRs, friction entries

### ForkSummary

Lightweight fork summary for list views:
- `id`: Unique identifier
- `name`: Human-readable fork name
- `domain`: Domain/organization
- `status`: Current status
- `kernel_version`: Template kernel version

## Fork File Format

Fork entries are stored as YAML files in the `forks/` directory with the pattern `FORK-*.yaml`:

```yaml
id: FORK-EXAMPLE-001
name: "Example Fork"
domain: rust-sdk
kernel_version: v3.3.3
status: active
url: https://github.com/example/fork
maintainer:
  name: "Example Maintainer"
  contact: maintainer@example.com
forked_at: "2025-11-26"
features:
  - feature1
  - feature2
```
