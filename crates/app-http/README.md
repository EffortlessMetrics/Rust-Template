# app-http

The main HTTP application for the Rust-as-Spec platform.

## What It Is

`app-http` is the primary HTTP service that exposes both the template's baseline endpoints and the full platform governance API. It is designed to be:

- **Production-Ready**: Includes middleware for security headers, CORS, request tracking, and metrics
- **Governance-Enabled**: Exposes `/platform/*` introspection endpoints via `gov-http`
- **Extensible**: Clean separation between template baseline and domain-specific handlers
- **Observable**: Built-in tracing, metrics, and structured error handling

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Application entry point, config validation, server startup |
| `lib.rs` | Router construction, application state, baseline handlers |
| `platform` | Platform introspection endpoints (`/platform/*`) |
| `middleware` | Request ID, CORS, security headers, platform auth |
| `errors` | Structured error handling with AC tracking |
| `metrics` | Prometheus metrics collection and export |
| `security` | Platform authentication configuration |
| `shutdown` | Graceful shutdown signal handling |
| `todos` | Example domain handlers (todos CRUD) |
| `tasks` | Task management UI and API handlers |
| `agent` | Agent hints and coverage endpoints |

### What It Is Not

- **Not a library**: This is an application crate (binary), not reusable middleware
- **Not domain logic**: Business rules live in `business-core`, not here
- **Not data access**: Database interactions live in `adapters-db-sqlx`
- **Not spec parsing**: Spec loading logic lives in `spec-runtime`

## Quick Start

### Running the Service

```bash
# From workspace root
cargo run -p app-http
```

The service will:
1. Load and validate `config/local.yaml` against `specs/config_schema.yaml`
2. Run database migrations if `database.auto_migrate: true` (disabled in local config by default)
3. Start the HTTP server on the configured port (default: 8080)
4. Expose health, version, and platform introspection endpoints

Local runs do not require Postgres. The default `config/local.yaml` disables auto-migrate so `cargo run -p app-http` boots without a database. If you want DB-backed paths locally, set `DATABASE_URL` and run `docker compose up -d`.

### Configuration

The service requires a valid configuration file:

```yaml
# config/local.yaml
env: "dev"
http_port: 8080

settings:
  database.auto_migrate: false

platform:
  auth_mode: "basic"  # or "jwt" or "none"

cors:
  enabled: true
  allowed_origins: ["http://localhost:3000"]

security_headers:
  enabled: true
  strict_transport_security: "max-age=31536000"
```

Configuration is validated against `specs/config_schema.yaml` at startup.

## Main Endpoints

### Template Baseline

These endpoints are part of the template's core contract:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check (always returns 200 OK) |
| `/version` | GET | Build version and git SHA |
| `/metrics` | GET | Prometheus metrics |
| `/api/echo` | POST | Example endpoint for error handling tests |

### Platform Introspection

Mounted at `/platform/*`, these endpoints expose governance data:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/status` | GET | Governance health, AC coverage, config summary |
| `/platform/graph` | GET | Full governance graph (stories → REQs → ACs → tests → docs) |
| `/platform/schema` | GET | All platform API schemas |
| `/platform/schema/{name}` | GET | Specific schema by name |
| `/platform/docs/index` | GET | Documentation inventory with health validation |
| `/platform/devex/flows` | GET | Developer flows and xtask commands |
| `/platform/coverage` | GET | AC coverage from BDD test results |
| `/platform/tasks` | GET | Task list with filtering |
| `/platform/tasks/suggest-next` | GET | Recommended next work |
| `/platform/tasks/graph` | GET | Task dependency graph |
| `/platform/friction` | GET | Development friction log |
| `/platform/questions` | GET | Design questions and ambiguities |
| `/platform/forks` | GET | Fork/branch information |

### UI Surfaces

HTML dashboards for human consumption:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` or `/ui` | GET | Dashboard overview |
| `/ui/graph` | GET | Governance graph visualization |
| `/ui/flows` | GET | Developer flows view |
| `/ui/coverage` | GET | AC coverage view |
| `/ui/tasks` | GET | Task management UI |

## Architecture

The service follows hexagonal/clean architecture:

```
HTTP Request
    ↓
Middleware (request_id, metrics, CORS, security_headers)
    ↓
Router (axum)
    ↓
Handler (this crate) - validates, extracts, responds
    ↓
Core Domain Logic (business-core) - pure business rules
    ↓
Repository (adapters-spec-fs, adapters-db-sqlx) - data access
```

**Dependency flow**: `app-http` → `business-core` → `model`

The HTTP layer never contains business logic. It only:
- Deserializes HTTP requests into domain types
- Calls core domain functions
- Serializes domain responses back to HTTP/JSON
- Handles HTTP-specific concerns (status codes, headers, errors)

## Error Handling

All errors use the `AppError` type from `errors.rs`:

```rust
use app_http::{AppError, ErrorCode};

// Validation error with AC tracking
return Err(AppError::validation_error(
    ErrorCode::MissingField,
    "Message cannot be empty"
)
.with_context("field", "message")
.with_ac_id("AC-TPL-003")
.with_request_id(request_id.as_str()));

// Domain error with correlation
return Err(AppError::domain_error(
    ErrorCode::ResourceNotFound,
    "Todo not found"
)
.with_context("id", &todo_id)
.with_feature_id("FT-TODO-001"));
```

Errors are automatically serialized to JSON with:
- Machine-readable error codes
- Human-readable messages
- Request ID for correlation
- AC/Feature ID for traceability
- Structured context fields

## Middleware

Middleware is applied in layers (bottom-up execution):

1. **Security Headers** (innermost) - CSP, HSTS, X-Frame-Options
2. **CORS** - Cross-origin request handling
3. **Metrics** - Request duration and status code tracking
4. **Request ID** (outermost) - Correlation ID generation

All middleware is configurable via `config/local.yaml`.

### Platform Authentication

The `/platform/*` endpoints support three auth modes:

| Mode | Description | Header |
|------|-------------|--------|
| `none` | No authentication (dev mode) | N/A |
| `basic` | Static bearer token | `Authorization: Bearer <token>` |
| `jwt` | JWT validation | `Authorization: Bearer <jwt>` |

Set via `platform.auth_mode` in config. Token is read from:
1. `platform.auth_token` in config
2. `PLATFORM_AUTH_TOKEN` environment variable

## State Management

The `AppState` struct provides shared context:

```rust
pub struct AppState {
    pub governance_repo: Arc<dyn GovernanceRepository>,
    pub workspace_root: PathBuf,
    pub config: Option<ValidatedConfig>,
    pub platform_auth: PlatformAuthConfig,
    pub cors_config: CorsConfig,
    pub security_headers_config: CachedSecurityHeaders,
    pub repo_context: RepoContext,
}
```

State is constructed once at startup and cloned (Arc) to each handler.

## Testing

The crate provides test helpers for integration tests:

```rust
use app_http::{app_with_workspace_root, test_helpers};
use adapters_spec_fs::FsGovernanceRepository;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_my_endpoint() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| Binary | `cargo run -p app-http` for local development |
| Docker | `FROM rust:... RUN cargo build -p app-http` in production images |
| Tests | Integration tests for HTTP contract validation |

## Stability

The HTTP API is versioned and considered the public contract:

| Endpoint | Schema Version | Notes |
|----------|----------------|-------|
| `/platform/schema` | 1.0 | Platform schemas |
| `/platform/graph` | 1.0 | Governance graph |
| `/platform/status` | 2.0 | Governance status (v2: AC coverage changes) |
| `/platform/coverage` | 1.0 | AC coverage |

**Breaking changes** to these endpoints require updating schema versions in `spec-runtime`.

## Adding New Endpoints

To add a new domain endpoint:

1. **Define the handler** in a new module (e.g., `src/my_domain.rs`)
2. **Add DTOs** for request/response (use `serde` attributes)
3. **Call core domain logic** from `business-core`
4. **Return structured errors** using `AppError`
5. **Add tests** to validate AC coverage
6. **Update router** in `lib.rs` to mount your routes

See `docs/how-to/add-http-endpoint.md` for a complete walkthrough.

## See Also

- `docs/how-to/add-http-endpoint.md` - Step-by-step guide for adding endpoints
- `docs/explanation/TEMPLATE-CONTRACTS.md` - HTTP API contracts and guarantees
- `crates/gov-http/README.md` - Governance endpoint implementation
- `crates/spec-runtime/README.md` - Spec loading and validation
