# http-errors

Canonical error types and error mapping traits for HTTP handlers.

## Purpose

This crate provides a foundation for HTTP error handling with:
- Machine-readable error codes for programmatic handling
- Standardized error response format matching platform contract
- Error mapping traits for converting domain errors to HTTP responses
- Framework-agnostic core with optional axum support

## Design Philosophy

Errors should be:
- **Actionable**: Include enough context to debug issues
- **Structured**: Use typed fields instead of string concatenation
- **Correlated**: Include request ID for tracing
- **Secure**: Don't leak internal details to clients

## Usage

### Basic Error Creation

```rust
use http_errors::{ErrorCode, HttpError};

// Simple error
let error = HttpError::bad_request("Invalid input");

// Error with specific code
let error = HttpError::validation_error(
    ErrorCode::InvalidFormat,
    "Task ID must match pattern TASK-\\d+"
);

// Not found error
let error = HttpError::not_found("Task not found");

// Internal error
let error = HttpError::internal_error("Database connection failed");
```

### Error Response Format

All errors serialize to a standard format:

```json
{
  "error": "INVALID_FORMAT",
  "message": "Task ID must match pattern TASK-\\d+",
  "requestId": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `INVALID_REQUEST` | 400 | Generic bad request |
| `INVALID_AMOUNT` | 400 | Invalid numeric value |
| `MISSING_FIELD` | 400 | Required field missing |
| `INVALID_FORMAT` | 400 | Invalid data format |
| `UNAUTHORIZED` | 401 | Authentication required |
| `RESOURCE_NOT_FOUND` | 404 | Resource does not exist |
| `INVALID_STATE` | 422 | Invalid state transition |
| `INVALID_TRANSITION` | 422 | Invalid state change |
| `CONFLICT` | 409 | Resource conflict |
| `DUPLICATE_REQUEST` | 409 | Duplicate request |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Service unavailable |
| `DATABASE_ERROR` | 500 | Database error |
| `EXTERNAL_SERVICE_ERROR` | 500 | External service error |

### Axum Integration

Enable the `axum` feature for automatic `IntoResponse` implementation:

```toml
[dependencies]
http-errors = { path = "../http-errors", features = ["axum"] }
```

```rust
use axum::response::IntoResponse;
use http_errors::HttpError;

async fn handler() -> Result<String, HttpError> {
    Err(HttpError::not_found("Task not found"))
}
```

### Error Mapping Trait

Implement `ToHttpError` for your domain errors:

```rust
use http_errors::{ToHttpError, HttpError, ErrorCode};

enum DomainError {
    TaskNotFound(String),
    InvalidTransition { from: String, to: String },
}

impl ToHttpError for DomainError {
    fn to_http_error(&self) -> HttpError {
        match self {
            DomainError::TaskNotFound(id) => {
                HttpError::not_found(format!("Task not found: {}", id))
            }
            DomainError::InvalidTransition { from, to } => {
                HttpError::business_logic_error(
                    ErrorCode::InvalidTransition,
                    format!("Invalid transition from {} to {}", from, to)
                )
            }
        }
    }
}
```

## Features

- `axum`: Provides `IntoResponse` implementation for `HttpError` (requires `uuid`)

## License

Apache-2.0 OR MIT
