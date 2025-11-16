# Observability Patterns in app-http

This document explains the observability patterns implemented in the `app-http` crate.

## Overview

The app-http implementation provides a comprehensive observability story through:

1. **OTLP Tracing** - Distributed tracing via OTLP exporter (configurable)
2. **Prometheus Metrics** - Full HTTP metrics + `/metrics` endpoint
3. **Request ID Correlation** - Track requests across services and log aggregations
4. **Structured Logging** - Consistent, queryable log data with rich context
5. **Error Tracking** - Machine-readable error codes with AC/Feature correlation

1. **Request ID Correlation** - Track requests across services and log aggregations
2. **Structured Logging** - Consistent, queryable log data with rich context
3. **Error Tracking** - Machine-readable error codes with AC/Feature correlation
4. **Metrics Integration Points** - Stubbed locations for adding metrics
5. **Distributed Tracing** - Span-based instrumentation with automatic context propagation

## Request ID Correlation

### Implementation: `middleware/request_id.rs`

The request ID middleware provides end-to-end request tracking:

```rust
use app_http::{RequestId, REQUEST_ID_HEADER};
use axum::extract::Extension;

async fn handler(Extension(request_id): Extension<RequestId>) {
    tracing::info!(request_id = %request_id, "Processing request");
}
```

### Features

- **Reads `X-Request-ID` header** if provided by client/proxy
- **Generates UUID** if no request ID is present
- **Stores in request extensions** for handler access
- **Adds to tracing span** automatically for log correlation
- **Returns in response header** for client tracking

### Design Note: Custom vs. Library Implementation

This template implements request ID middleware manually for **educational purposes** - it shows the complete pattern and is easy to understand and customize.

**Alternative:** If you prefer a batteries-included approach, `tower-http` provides equivalent functionality:

```rust
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use http::HeaderName;

let header = HeaderName::from_static("x-request-id");
let make_request_id = MakeRequestUuid::default();

Router::new()
    .route("/health", get(health))
    .layer(PropagateRequestIdLayer::new(header.clone()))
    .layer(SetRequestIdLayer::new(header, make_request_id))
```

Both approaches are valid:
- **Custom middleware** (current): Full control, clear pattern, easier to extend with custom logic
- **tower-http**: Less code, well-tested library, standard ecosystem tool

Choose based on your team's preference for explicitness vs. convenience.

### Example Flow

```
Client Request
  ├─> X-Request-ID: abc-123 (optional)
  │
  └─> Middleware
       ├─> Extract or generate: abc-123
       ├─> Add to span: request_id=abc-123
       ├─> Store in extensions
       └─> Process request
            │
            ├─> Handler logs: [request_id=abc-123] Processing...
            ├─> Core logs: [request_id=abc-123] Business logic...
            └─> Error logs: [request_id=abc-123] Error occurred
                 │
                 └─> Response
                      └─> X-Request-ID: abc-123
```

### Benefits

1. **Client correlation**: Clients can track their requests across retries
2. **Log aggregation**: Group all logs for a single request
3. **Debugging**: Support can search logs by request ID from user reports
4. **Distributed tracing**: Pass request ID to downstream services

## Structured Error Handling

### Implementation: `errors.rs`

Enhanced error type with full observability support:

```rust
use app_http::{AppError, ErrorCode};

// Simple validation error
return Err(AppError::validation_error(
    ErrorCode::InvalidAmount,
    "Amount must be positive"
));

// Error with full context
return Err(
    AppError::validation_error(
        ErrorCode::InvalidAmount,
        "Amount must be greater than 0"
    )
    .with_context("field", "amount_cents")
    .with_context("value", payload.amount_cents)
    .with_ac_id("AC-REFUND-001")
    .with_feature_id("FT-REFUND-CREATION")
);
```

### Features

- **Error Codes**: Machine-readable codes for client handling and metrics
- **AC ID Tracking**: Link errors to acceptance criteria for product tracking
- **Feature ID Tracking**: Associate errors with specific features
- **Structured Context**: Type-safe context fields (logged, not exposed to clients)
- **Automatic Logging**: Errors log with appropriate severity (warn for 4xx, error for 5xx)
- **JSON Responses**: Consistent error response format

### Error Response Format

```json
{
  "code": "INVALID_AMOUNT",
  "message": "Amount must be greater than 0",
  "ac_id": "AC-REFUND-001",
  "feature_id": "FT-REFUND-CREATION"
}
```

### Log Output

```
WARN http_error: error_code=INVALID_AMOUNT status_code=400
  message="Amount must be greater than 0"
  context={"field": "amount_cents", "value": 0}
  ac_id="AC-REFUND-001"
  feature_id="FT-REFUND-CREATION"
  request_id="550e8400-e29b-41d4-a716-446655440000"
```

## Instrumentation Patterns

### Handler Instrumentation

The `create_refund` handler demonstrates best practices:

```rust
#[instrument(
    skip(_request_id, payload),
    fields(
        order_id = %payload.order_id,
        amount_cents = payload.amount_cents,
    )
)]
async fn create_refund(
    Extension(_request_id): Extension<RequestId>,
    Json(payload): Json<CreateRefundRequest>,
) -> Result<(StatusCode, Json<CreateRefundResponse>), AppError> {
    info!("Processing refund creation request");

    // ... validation with detailed errors ...

    info!(
        refund_id = %refund.id,
        order_id = %payload.order_id,
        amount_cents = payload.amount_cents,
        "Refund created successfully"
    );

    Ok(response)
}
```

### Key Patterns

1. **Use `#[instrument]`** on all handler functions
2. **Add business context** as span fields (order_id, amount, etc.)
3. **Skip large payloads** to avoid logging sensitive/verbose data
4. **Log outcomes** with structured fields
5. **Request ID is automatic** - no need to manually add it

## Metrics Integration Points

The implementation includes stubbed locations for metrics:

### Health Check Counter

```rust
// In health handler
// metrics::counter!("health_checks_total").increment(1);
```

### Error Metrics

```rust
// In errors.rs - AppError::log_error()
// counter!(
//     "http_errors_total",
//     "status" => self.status.as_str(),
//     "code" => self.code.to_string(),
//     "ac_id" => self.ac_id.as_deref().unwrap_or("none"),
// ).increment(1);
```

### Request Latency

```rust
// In refund handler
// let _timer = metrics::histogram!("refund_creation_duration_seconds").start_timer();
```

### Validation Errors

```rust
// In validation logic
// metrics::counter!("refund_validation_errors_total", "field" => "amount").increment(1);
```

### Business Metrics

```rust
// In successful path
// metrics::counter!("refunds_created_total", "status" => "success").increment(1);
// metrics::histogram!("refund_amount_cents").record(payload.amount_cents as f64);
```

## Complete Observability Story

### 1. Request Arrives

```
[INFO] http_request: method=POST uri=/refunds request_id=550e8400-...
```

### 2. Handler Processing

```
[INFO] create_refund: request_id=550e8400-... order_id=ORD-123 amount_cents=1000
  Processing refund creation request
```

### 3. Validation Failure (Example)

```
[WARN] http_error: request_id=550e8400-... error_code=INVALID_AMOUNT status_code=400
  message="Amount must be greater than 0"
  context={"field": "amount_cents", "value": 0}
  ac_id="AC-REFUND-001"
  feature_id="FT-REFUND-CREATION"
```

### 4. Successful Creation

```
[INFO] create_refund: request_id=550e8400-... refund_id=REF-789 order_id=ORD-123 amount_cents=1000
  Refund created successfully
```

### 5. Response

```
HTTP/1.1 201 Created
X-Request-ID: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json

{
  "refundId": "REF-789",
  "orderId": "ORD-123",
  "amountCents": 1000,
  "status": "pending"
}
```

## Querying Logs

With this structure, you can easily query logs:

### Find all logs for a request
```
request_id="550e8400-e29b-41d4-a716-446655440000"
```

### Find all validation errors
```
error_code="INVALID_AMOUNT"
```

### Find errors for an AC
```
ac_id="AC-REFUND-001"
```

### Find errors for a feature
```
feature_id="FT-REFUND-CREATION"
```

### Find slow requests (with metrics)
```
refund_creation_duration_seconds > 1.0
```

## Adding Metrics

## Production Observability Configuration

### OTLP Tracing
```bash
# Enable OTLP export (default: console fallback)
export OTLP_ENDPOINT=http://otel-collector:4317

# Log level filtering
export RUST_LOG=info,app_http=debug
```

### Prometheus Metrics
- **Endpoint**: `GET /metrics`
- **Metrics**: `http_requests_total`, `http_requests_duration_seconds`, `http_errors_total`
- **Dimensions**: `method`, `status`, `uri`

### Example Metrics Output
```bash
curl http://localhost:3000/metrics | grep http_requests
# http_requests_total{method="GET",outcome="success",status="200",uri="/health"} 5 1734251234567
# http_requests_duration_seconds_bucket{...le="0.005"} 2
```

### Complete Stack
```
Client → app-http:3000 → OTLP(gRPC):4317 → Collector → Backend
                           ↓
                       Prometheus:9090 ← /metrics
```

To add actual metrics (currently stubbed):

1. Add metrics crate to `Cargo.toml`:
   ```toml
   metrics = "0.21"
   metrics-exporter-prometheus = "0.13"
   ```

2. Initialize metrics in `main.rs`:
   ```rust
   let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
       .install()
       .expect("Failed to install Prometheus exporter");
   ```

3. Uncomment metric stubs in:
   - `lib.rs` - Handler metrics
   - `errors.rs` - Error metrics
   - Add middleware metrics as needed

4. Expose metrics endpoint:
   ```rust
   .route("/metrics", get(|| async move {
       metrics_handle.render()
   }))
   ```

## Testing Observability

### Test Request ID Generation

```bash
# Without request ID
curl -v http://localhost:3000/health
# Response will include: X-Request-ID: <generated-uuid>

# With request ID
curl -v -H "X-Request-ID: my-test-id" http://localhost:3000/health
# Response will include: X-Request-ID: my-test-id
```

### Test Error Responses

```bash
# Trigger validation error
curl -X POST http://localhost:3000/refunds \
  -H "Content-Type: application/json" \
  -d '{"orderId": "ORD-123", "amountCents": 0}'

# Response:
{
  "code": "INVALID_AMOUNT",
  "message": "Amount must be greater than 0",
  "ac_id": "AC-REFUND-001",
  "feature_id": "FT-REFUND-CREATION"
}
```

### View Structured Logs

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p app-http

# Filter specific request
RUST_LOG=app_http=trace cargo run -p app-http 2>&1 | grep "request_id=abc-123"
```

## Best Practices Summary

1. **Always use request ID middleware** - It's already configured in the router
2. **Extract RequestId in handlers** - Even if not used, it ensures middleware works
3. **Use AppError for all errors** - Provides consistent observability
4. **Add AC/Feature IDs** - Links errors to product features
5. **Use structured logging** - `tracing::info!()` with fields, not string formatting
6. **Instrument all handlers** - Use `#[instrument]` macro
7. **Add business context** - Include domain-specific fields in spans
8. **Stub metrics early** - Mark where metrics should go during development
9. **Log outcomes** - Success and failure paths should both log
10. **Keep errors safe** - Use `with_context()` for internal details, not in messages

## Architecture Integration

This observability implementation follows the template's hexagonal architecture:

- **HTTP Layer** (`app-http`): Request ID correlation, error responses
- **Domain Layer** (`core`): Business logic instrumentation (add `#[instrument]` there too)
- **Telemetry Layer** (`telemetry`): Centralized logging configuration
- **Cross-cutting**: Request ID flows through all layers automatically via span context

The request ID and structured logging work across layer boundaries, providing full request lifecycle visibility.
