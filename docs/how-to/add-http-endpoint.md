# How-to: Add a New HTTP Endpoint

**Time:** 15 minutes
**Prerequisites:** Template running, basic Rust knowledge

This guide shows you how to add a new HTTP endpoint following template patterns.

---

## Example: Add GET /refunds/:id endpoint

We'll add an endpoint to retrieve a refund by ID.

### Step 1: Add Route Handler

**File:** `crates/app-http/src/main.rs`

Add the handler function:

```rust
/// Get refund by ID endpoint
#[instrument]
async fn get_refund(
    Path(refund_id): Path<String>,
) -> Result<Json<GetRefundResponse>, AppError> {
    info!(refund_id = %refund_id, "Fetching refund");

    // Call domain logic (in real system, this would query DB)
    // For now, simulate found vs not found
    if refund_id.starts_with("REF-") {
        Ok(Json(GetRefundResponse {
            refund_id: refund_id.clone(),
            order_id: "ORD-123".to_string(),
            amount_cents: 5000,
            status: "completed".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }))
    } else {
        Err(AppError::NotFound(format!("Refund {} not found", refund_id)))
    }
}
```

### Step 2: Add Response DTO

Add the DTO after `CreateRefundResponse`:

```rust
#[derive(Debug, Serialize)]
struct GetRefundResponse {
    refund_id: String,
    order_id: String,
    amount_cents: u64,
    status: String,
    created_at: String,
}
```

### Step 3: Add Error Variant

Update the `AppError` enum:

```rust
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),  // ← Add this
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),  // ← Add this
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
```

### Step 4: Add Import

At the top of the file, add to imports:

```rust
use axum::{
    extract::{Json, Path},  // ← Add Path
    // ... rest
};
```

### Step 5: Register Route

In the `main()` function, add the route:

```rust
let app = Router::new()
    .route("/health", get(health))
    .route("/refunds", post(create_refund))
    .route("/refunds/:id", get(get_refund))  // ← Add this
    .layer(TraceLayer::new_for_http());
```

### Step 6: Validate

Run checks:
```bash
cargo run -p xtask -- check
```

Start the server:
```bash
cargo run -p app-http
```

Test the endpoint:
```bash
# Should return refund
curl http://localhost:3000/refunds/REF-12345

# Should return 404
curl http://localhost:3000/refunds/INVALID
```

---

## Pattern Summary

When adding HTTP endpoints, follow this pattern:

### 1. Handler Signature

```rust
#[instrument(skip(payload))]  // ← Add tracing
async fn my_handler(
    Path(id): Path<String>,      // Path parameters
    Json(req): Json<MyRequest>,  // Request body
) -> Result<Json<MyResponse>, AppError> {  // Response or error
    // Handler logic
}
```

### 2. DTOs

```rust
#[derive(Debug, Deserialize)]  // ← For requests
struct MyRequest {
    field: String,
}

#[derive(Debug, Serialize)]  // ← For responses
struct MyResponse {
    result: String,
}
```

### 3. Error Handling

- Use `AppError` variants for different HTTP status codes
- Convert domain errors to `AppError`
- Let Axum's `IntoResponse` handle the rest

### 4. Routing

```rust
Router::new()
    .route("/path", get(handler))     // GET
    .route("/path", post(handler))    // POST
    .route("/path/:id", get(handler)) // Path param
```

---

## Common Patterns

### Query Parameters

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn list_refunds(Query(pagination): Query<Pagination>) -> Result<...> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    // ...
}
```

### Headers

```rust
use axum::http::HeaderMap;

async fn my_handler(headers: HeaderMap) -> Result<...> {
    if let Some(auth) = headers.get("authorization") {
        // Validate auth
    }
    // ...
}
```

### Middleware

```rust
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

let app = Router::new()
    .route("/slow", get(slow_handler))
    .layer(
        ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(TraceLayer::new_for_http())
    );
```

---

## Best Practices

### ✅ DO:

- **Use `#[instrument]`** on all handlers for tracing
- **Keep handlers thin** - delegate to core domain logic
- **Validate inputs** in handler, business logic in core
- **Use strong types** - leverage Axum's extractors
- **Add integration tests** for endpoints

### ❌ DON'T:

- **Don't put business logic in handlers** - handlers translate, core decides
- **Don't ignore errors** - propagate with `?` or convert to AppError
- **Don't skip validation** - validate early at HTTP boundary
- **Don't bypass observability** - always use `#[instrument]`

---

## Testing Your Endpoint

### Unit Test (if handler has logic)

```rust
#[cfg(test)]
mod tests {
    use super::*

;

    #[tokio::test]
    async fn test_get_refund_found() {
        let response = get_refund(Path("REF-123".to_string())).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_refund_not_found() {
        let response = get_refund(Path("INVALID".to_string())).await;
        assert!(response.is_err());
    }
}
```

### Integration Test

Create `crates/app-http/tests/integration_test.rs`:

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_get_refund_endpoint() {
    let app = app_http::create_app(); // You'd need to expose this

    let response = app
        .oneshot(
            Request::builder()
                .uri("/refunds/REF-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Next Steps

- **Add to BDD scenario:** Map endpoint to an AC with Gherkin test
- **Update API docs:** Document in OpenAPI/Proto if you're using them
- **Add to feature flag:** If endpoint should be gradual rollout
- **Monitor in production:** Ensure tracing captures key metrics

---

## Related Guides

- `docs/tutorials/first-ac-change.md` - AC-first workflow
- `docs/explanation/architecture.md` - Hexagonal architecture details
- `TEMPLATE_API.md` - xtask commands reference
