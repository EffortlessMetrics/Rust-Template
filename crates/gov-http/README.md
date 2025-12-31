# gov-http

Platform HTTP router for governance introspection endpoints.

## What It Is

`gov-http` is a reusable Axum router that provides all `/platform/*` governance endpoints. It is designed to be:

- **Composable**: Mounts into any Axum-based HTTP service via `.nest()`
- **State-Agnostic**: Works with any state implementing the `PlatformState` trait
- **Contract-Stable**: Versioned schemas with backward compatibility guarantees
- **Minimal**: No business logic, just HTTP-to-spec translation

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `handlers` | Core platform endpoints (status, schema, graph, coverage, tasks) |
| `state` | `PlatformState` trait and default implementation |
| `error` | `PlatformError` type for governance endpoint errors |
| `friction` | Friction log endpoints (`/platform/friction`) |
| `questions` | Design question endpoints (`/platform/questions`) |
| `forks` | Fork registry endpoints (`/platform/forks`) |

### What It Is Not

- **Not a standalone service**: This is a library that provides routers for composition
- **Not spec loading**: Uses `spec-runtime` for all spec parsing
- **Not business logic**: Only translates specs to HTTP/JSON responses
- **Not authentication**: Consuming services must apply auth middleware

## Quick Start

### Basic Integration

Compose the governance router into your service:

```rust
use gov_http::{platform_router, DefaultPlatformState};
use gov_model::RepoContext;
use axum::Router;
use std::sync::Arc;

let ctx = RepoContext::new("/workspace");
let repo = my_governance_repo(); // Arc<dyn GovernanceRepository>
let state = Arc::new(DefaultPlatformState::new(ctx, repo));

let app = Router::new()
    .nest("/platform", platform_router(state));
```

This mounts all governance endpoints at `/platform/*`.

### Custom State Integration

If you have your own state type, implement `PlatformState`:

```rust
use gov_http::PlatformState;
use gov_model::{RepoContext, GovernanceRepository};
use std::sync::Arc;

struct MyAppState {
    repo_context: RepoContext,
    governance_repo: Arc<dyn GovernanceRepository>,
    // ... your other fields
}

impl PlatformState for MyAppState {
    fn context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn governance_repo(&self) -> Arc<dyn GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

// Now use your state directly
let app = Router::new()
    .nest("/platform", gov_http::platform_router(my_app_state));
```

### Stateless Routing

For more complex router composition, use the stateless variant:

```rust
use gov_http::platform_routes;

let gov_routes = platform_routes::<MyAppState>();
let app_routes = my_app_routes::<MyAppState>();

let app = gov_routes
    .merge(app_routes)
    .with_state(my_state);
```

## Endpoint Categories

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/health` | GET | Health check (always returns 200 OK) |
| `/platform/status` | GET | Simplified governance status |

### Contract Anchor Endpoints

These define the "governed service cell" contract:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/schema` | GET | All platform API schemas |
| `/platform/schema/{name}` | GET | Specific schema by name |
| `/platform/openapi` | GET | OpenAPI spec (YAML) |
| `/platform/openapi.yaml` | GET | OpenAPI spec (YAML) alias |
| `/platform/docs/index` | GET | Documentation inventory with health validation |
| `/platform/ui/contract` | GET | UI contract specification |

### Governance Introspection

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/graph` | GET | Full governance graph (stories → REQs → ACs → tests → docs) |
| `/platform/devex/flows` | GET | Developer experience flows and commands |
| `/platform/coverage` | GET | AC coverage from BDD test results |

### Tasks

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/tasks` | GET | Task list with optional filtering |
| `/platform/tasks/suggest-next` | GET | Recommended next work based on task dependencies |
| `/platform/tasks/graph` | GET | Task dependency graph (JSON or Mermaid format) |

### Governance Artifacts

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/platform/friction` | GET | Development friction log |
| `/platform/questions` | GET | Design questions and ambiguities |
| `/platform/forks` | GET | Fork/branch information |

## Response Types

All response types are exported for downstream use:

```rust
use gov_http::{
    // Coverage types
    CoverageResponse, CoverageSummary, CoverageDetail,
    // Docs types
    DocsIndexResponse, DocInfoWithHealth, DocHealthSummary,
    // Task types
    TasksResponse, TaskOut, TaskDocsOut, TaskGraphResponse,
    // Friction types
    FrictionListResponse, FrictionEntry, FrictionContext,
    // Question types
    QuestionsListResponse, QuestionSummary, QuestionContext,
    // Fork types
    ForksListResponse, ForkEntry, ForkSummary,
};
```

These types can be used by services that want to extend or wrap the responses.

## Router Variants

### Full Router

Includes all endpoints including the simplified `/status`:

```rust
let router = gov_http::platform_router(state);
```

### No Status Router

Excludes `/status` so services can provide a richer status endpoint:

```rust
let gov_routes = gov_http::platform_routes_no_status::<AppState>();
let app = Router::new()
    .merge(gov_routes)
    .route("/platform/status", get(my_rich_status_handler))
    .with_state(state);
```

Use this when you want to add service-specific health metrics or metadata.

### Minimal Router

Only health and status, no governance:

```rust
let router = gov_http::minimal_router();
```

Useful for lightweight services that don't need full governance introspection.

## Query Parameters

### Tasks Endpoint

Filter tasks by status, priority, or tags:

```bash
# Filter by status
GET /platform/tasks?status=Todo

# Filter by priority
GET /platform/tasks?priority=High

# Filter by tags
GET /platform/tasks?tags=feature,security

# Combine filters
GET /platform/tasks?status=InProgress&priority=High
```

### Suggest Next Endpoint

Get task recommendations based on current work:

```bash
# Get suggestions for a specific task
GET /platform/tasks/suggest-next?current_task=TASK-001

# Get general suggestions
GET /platform/tasks/suggest-next
```

### Task Graph Endpoint

Control graph output format:

```bash
# JSON format (default)
GET /platform/tasks/graph?format=json

# Mermaid diagram format
GET /platform/tasks/graph?format=mermaid
```

### Friction Endpoint

Filter friction entries:

```bash
# Filter by status
GET /platform/friction?status=open

# Filter by severity
GET /platform/friction?severity=high

# Combine filters
GET /platform/friction?status=open&severity=critical
```

### Questions Endpoint

Filter design questions:

```bash
# Filter by status
GET /platform/questions?status=open

# Filter by flow
GET /platform/questions?flow=feature_development

# Combine filters
GET /platform/questions?status=open&flow=release_prep
```

## Error Handling

All endpoints use `PlatformError` for consistent error responses:

```rust
use gov_http::PlatformError;

// Not found error
return Err(PlatformError::not_found("Schema 'Foo' not found"));

// Spec load error
return Err(PlatformError::spec_load("doc index", err));

// Internal error
return Err(PlatformError::internal("Failed to build graph"));
```

Errors are automatically converted to appropriate HTTP status codes:
- `not_found` → 404
- `spec_load` → 500
- `internal` → 500

Error responses include:
- `error`: Machine-readable error code
- `message`: Human-readable description
- `details`: Optional context (only in dev mode)

## Authentication

`gov-http` does not enforce authentication. Consuming services should apply auth middleware:

```rust
use axum::middleware::from_fn;

let protected_platform = gov_http::platform_router(state)
    .layer(from_fn(my_auth_middleware));

let app = Router::new()
    .nest("/platform", protected_platform);
```

See `app-http::middleware::platform_auth` for an example implementation.

## Health Validation

The `/platform/docs/index` endpoint validates doc contracts:

```rust
// Each doc entry includes health status
{
  "docs": [
    {
      "id": "DOC-001",
      "file": "docs/how-to/example.md",
      "doc_type": "how_to",
      "health": {
        "valid": true,
        "issue": null
      },
      "stories": ["US-001"],
      "requirements": ["REQ-001"],
      "acs": ["AC-001"]
    }
  ],
  "summary": {
    "total": 42,
    "healthy": 40,
    "unhealthy": 2
  }
}
```

**Doc contract rules** (enforced per `doc_type`):
- `how_to`: Must reference at least one REQ or AC
- `explanation`: Must reference at least one story or REQ
- `design_doc`: Must reference at least one REQ
- `reference`: Must reference at least one REQ or AC
- `status`: Must reference both REQs and ACs
- `adr`: Must reference at least one REQ
- `guide`: Must reference at least one REQ or AC
- `impl_plan`: Must reference both REQs and ACs
- `requirements_doc`: Must reference at least one REQ

Unhealthy docs are flagged with validation errors in the response.

## Coverage Details

The `/platform/coverage` endpoint provides AC coverage from BDD tests:

```rust
{
  "summary": {
    "total": 120,
    "passing": 100,
    "failing": 10,
    "unknown": 10,
    "pass_rate": 83.3
  },
  "details": [
    {
      "ac_id": "AC-001",
      "status": "Pass",
      "scenarios_passed": 2,
      "scenarios_failed": 0,
      "scenarios_total": 2
    }
  ]
}
```

Status values:
- `Pass`: All BDD scenarios passed
- `Fail`: At least one scenario failed
- `Unknown`: No BDD coverage found

## Governance Graph

The `/platform/graph` endpoint returns a full governance graph:

```rust
{
  "nodes": [
    {"id": "US-001", "type": "Story", "label": "User can login"},
    {"id": "REQ-001", "type": "Requirement", "label": "Auth system"},
    {"id": "AC-001", "type": "AC", "label": "Password validation"}
  ],
  "edges": [
    {"from": "US-001", "to": "REQ-001", "type": "has_requirement"},
    {"from": "REQ-001", "to": "AC-001", "type": "has_ac"}
  ]
}
```

Use this to:
- Visualize governance relationships
- Validate traceability
- Find orphaned ACs or REQs
- Generate compliance reports

## PlatformState Trait

The `PlatformState` trait provides the minimum contract:

```rust
pub trait PlatformState: Clone + Send + Sync + 'static {
    /// Get the repository context (workspace paths).
    fn context(&self) -> &RepoContext;

    /// Get the governance repository (spec loading).
    fn governance_repo(&self) -> Arc<dyn GovernanceRepository>;
}
```

Implement this trait to use `gov-http` routers with your state.

## Default Implementation

If you don't have custom state, use `DefaultPlatformState`:

```rust
use gov_http::DefaultPlatformState;

let state = DefaultPlatformState::new(repo_context, governance_repo);
let router = gov_http::platform_router(Arc::new(state));
```

This is a minimal implementation suitable for standalone services.

## Consumers

This crate is used by:

| Crate | Usage |
|-------|-------|
| `app-http` | Primary consumer, mounts all `/platform/*` endpoints |
| Future services | Any Axum-based service needing governance introspection |

## Stability

The HTTP contract is versioned and stable:

| Response Type | Schema Version | Breaking Change Policy |
|---------------|----------------|------------------------|
| `/platform/schema` | 1.0 | Major version bump required |
| `/platform/graph` | 1.0 | Major version bump required |
| `/platform/coverage` | 1.0 | Major version bump required |
| `/platform/docs/index` | 1.0 | Major version bump required |

**Backward compatibility**: New fields can be added without version bumps. Removing or renaming fields requires a major version bump and deprecation period.

## Testing

Test your integration using `tower::ServiceExt`:

```rust
use gov_http::platform_router;
use tower::ServiceExt;
use axum::http::{Request, StatusCode};

#[tokio::test]
async fn test_platform_health() {
    let state = my_test_state();
    let app = platform_router(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## See Also

- `crates/spec-runtime/README.md` - Spec loading library used by handlers
- `crates/app-http/README.md` - Primary consumer of this library
- `docs/reference/platform-api-schema.md` - Full API schema documentation
- `docs/how-to/add-platform-endpoint.md` - Guide for extending governance endpoints
