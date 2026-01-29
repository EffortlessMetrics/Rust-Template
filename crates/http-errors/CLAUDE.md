# http-errors – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal)
**Dependencies:** Minimal; Axum/UUID are feature-gated

## Purpose

HTTP error types with optional framework integration. Provides consistent error envelopes across all HTTP endpoints.

## Key Exports

- `HttpError` – Core HTTP error type
- Error response envelopes
- Status code mapping

## Features

- `axum` – Enables Axum `IntoResponse` impl
- `uuid` – Enables request ID generation

## When to Modify

- Adding new error categories
- Extending error metadata

## When NOT to Modify

- Adding Axum-specific logic without feature gate
- Adding business logic to errors

## Architectural Notes

- **Feature-gated**: Axum dependency is optional
- **Consistent envelope**: All errors have same JSON shape
- **Foundation**: Used by all HTTP crates

## Consumers

`http-core`, `http-middleware`, `app-http`, `gov-http`

## See Also

- `crates/platform-contract/` for `ErrorCode` and `ErrorResponse`
- `crates/http-core/` for HTTP utilities using these errors
