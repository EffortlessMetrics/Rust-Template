---
id: HOWTO-TPL-ADD-ENDPOINT-001
title: Add a New HTTP Endpoint
doc_type: how-to
status: published
audience: developers
tags: [axum, http, onboarding, platform]
stories: [US-TPL-001]
requirements: [REQ-PLT-ONBOARDING]
acs: []
adrs: [ADR-0001]
last_updated: 2025-12-05
---

# How to Add a New HTTP Endpoint

**Time:** 10-15 minutes
**Prerequisites:** Template running, basic Rust/Axum knowledge

This guide shows you how to add HTTP endpoints to the Rust-as-Spec platform cell.

> **Already know Axum?** See the [Axum Mental Map](../explanation/axum-mental-map.md) for a quick
> orientation to where things live in this repo.

---

## Quick Start: Add GET /hello

This is the simplest possible example—add a handler, wire it, done.

### Step 1: Add handler to lib.rs

Open `crates/app-http/src/lib.rs` and add a handler function near the other handlers:

```rust
/// Simple hello endpoint
#[instrument]
async fn hello() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "Hello from Rust-as-Spec!"
    }))
}
```

### Step 2: Wire it into the router

In the same file, find `build_router()` and add the route:

```rust
fn build_router(app_state: AppState) -> Router {
    // ... existing code ...

    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/hello", get(hello))  // <-- Add this line
        // ... rest of routes ...
}
```

### Step 3: Test it

```bash
cargo run -p app-http
# In another terminal:
curl http://localhost:8080/hello
# -> {"message":"Hello from Rust-as-Spec!"}
```

That's it. You've added an endpoint using normal Axum patterns. No specs, no BDD, no governance—yet.

---

## Full Example: GET /platform/debug/info (Real Endpoint)

This endpoint actually exists in the template—try it right now:

```bash
cargo run -p app-http &
curl http://localhost:8080/platform/debug/info
# -> {"kernel_version":"0.1.0","template_version":"v3.3.6"} # doclint:disable orphan-version
```

Here's the real implementation in `crates/app-http/src/platform.rs`:

### The DTO

```rust
#[derive(Debug, Serialize)]
struct DebugInfo {
    kernel_version: String,
    template_version: String,
}
```

### The handler

```rust
async fn debug_info(State(state): State<AppState>) -> Json<DebugInfo> {
    let root = &state.workspace_root;

    let template_version = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .ok()
        .and_then(|m| m.template_version)
        .unwrap_or_else(|| "unknown".to_string());

    Json(DebugInfo {
        kernel_version: env!("CARGO_PKG_VERSION").to_string(),
        template_version,
    })
}
```

### The route (in the platform router)

```rust
.route("/debug/info", get(debug_info))
```

> **Note:** `/platform/debug/info` is a **convenience endpoint** for development—it's
> not in OpenAPI and has no AC. Most `/platform/*` endpoints (like `/platform/status`)
> are contracted and appear in OpenAPI. This one exists purely for quick debugging.
> For domain-specific endpoints, follow the same pattern but under your own path prefix.

---

## Common Patterns

### Query Parameters

```rust
use axum::extract::Query;

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[instrument(skip(pagination))]
async fn list_items(Query(pagination): Query<Pagination>) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    // ...
}
```

### Request Body

```rust
#[derive(Debug, Deserialize)]
struct CreateItemRequest {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[instrument(skip(payload))]
async fn create_item(
    Json(payload): Json<CreateItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate
    if payload.name.is_empty() {
        return Err(AppError::validation_error(
            ErrorCode::MissingField,
            "Name is required"
        ).with_context("field", "name"));
    }
    // ...
}
```

### Using AppState

```rust
use axum::extract::State;

#[instrument(skip(state))]
async fn get_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Access state.governance_repo, state.config, etc.
    let config = state.config.as_ref();
    // ...
}
```

### Headers

```rust
use axum::http::HeaderMap;

async fn my_handler(headers: HeaderMap) -> impl IntoResponse {
    if let Some(auth) = headers.get("authorization") {
        // Handle auth
    }
    // ...
}
```

---

## Error Handling

Use `AppError` from `crates/app-http/src/errors.rs`:

```rust
use crate::errors::{AppError, ErrorCode};

async fn my_handler() -> Result<Json<Response>, AppError> {
    // Validation error (400)
    Err(AppError::validation_error(ErrorCode::MissingField, "Field X is required")
        .with_context("field", "x"))

    // Not found (404)
    Err(AppError::not_found("Item", "item-123"))

    // Internal error (500)
    Err(AppError::internal("Database connection failed"))
}
```

The error response follows the template's error envelope format (AC-TPL-003).

---

## Promote to Governed Contract

If your endpoint is just a scratch/debug endpoint, you can stop here.

If you want it to be part of the **real platform contract** (documented, tested, versioned),
follow these additional steps:

### Step 1: Add REQ and AC to spec_ledger.yaml

Edit `specs/spec_ledger.yaml`:

```yaml
stories:
  - id: US-PLT-DEBUG
    title: "Platform Debug Endpoints"
    requirements:
      - id: REQ-PLT-DEBUG-INFO
        title: "Debug Info API"
        tags: [api, platform, debug]
        must_have_ac: false  # Start as non-kernel
        acceptance_criteria:
          - id: AC-PLT-DEBUG-INFO
            text: "GET /platform/debug/info returns 200 with kernel version and metadata"
            tags: [api]
            must_have_ac: false
            tests:
              - { type: bdd, tag: "@AC-PLT-DEBUG-INFO", file: "specs/features/platform_debug.feature" }
```

### Step 2: Add BDD scenario

Create `specs/features/platform_debug.feature`:

```gherkin
Feature: Platform Debug API

  @AC-PLT-DEBUG-INFO
  Scenario: Get debug info
    Given the platform HTTP server is running
    When I GET "/platform/debug/info"
    Then the response status is 200
    And the JSON body has field "kernel_version"
    And the JSON body has field "template_version"
```

### Step 3: Add OpenAPI schema (optional but recommended)

Edit `specs/openapi/openapi.yaml`:

```yaml
paths:
  /platform/debug/info:
    get:
      summary: Platform debug info
      operationId: getDebugInfo
      tags: [platform]
      responses:
        "200":
          description: Debug information
          content:
            application/json:
              schema:
                type: object
                required: [kernel_version, template_version]
                properties:
                  kernel_version:
                    type: string
                    description: "Kernel crate version"
                  template_version:
                    type: string
                    description: "Template version used to create this service"
        "500":
          $ref: "#/components/responses/InternalError"
```

### Step 4: Run governance checks

```bash
cargo xtask ac-status      # See your AC in the list
cargo xtask bdd            # Run BDD scenarios
cargo xtask selftest       # Full governance check
cargo xtask idp-check      # Validate OpenAPI + TS consumer
```

If all gates are green, your endpoint is now governed.

---

## Testing

### Unit test (handler logic)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_returns_message() {
        let response = hello().await;
        // Assert on response
    }
}
```

### Integration test (full HTTP stack)

Create `crates/app-http/tests/platform_debug_test.rs`:

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_debug_info_returns_kernel_version() {
    let app = app_http::app(/* governance_repo */);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/debug/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Best Practices

### DO

- Use `#[instrument]` on all handlers for tracing
- Keep handlers thin—delegate to `business-core` for logic
- Validate inputs early at the HTTP boundary
- Use strong types with Axum extractors
- Add integration tests for important endpoints

### DON'T

- Put business logic in handlers—handlers translate, core decides
- Ignore errors—propagate with `?` or convert to `AppError`
- Skip validation—validate at HTTP boundary, not deep in core
- Bypass observability—always use `#[instrument]`

---

## Summary

For new developers:

1. **Start with Axum**: add a handler in `lib.rs`, wire it in `build_router()`, test it.
2. **When it matters**: add REQ/AC to `spec_ledger.yaml`, write BDD scenarios, update OpenAPI.
3. **Run the gates**: `ac-status`, `selftest`, `idp-check`.

That's Rust-as-Spec in action: you never lose the simplicity of "add an Axum handler",
but you always have a path to turn it into a **provable contract** when you're ready.

---

## Related Docs

- [Axum Mental Map](../explanation/axum-mental-map.md) – where things live in this repo
- [Architecture overview](../explanation/architecture.md) – hexagonal architecture explained
- [First AC change tutorial](../tutorials/first-ac-change.md) – full AC-first workflow
