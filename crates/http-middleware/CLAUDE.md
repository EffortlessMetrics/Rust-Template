# http-middleware – CLAUDE.md

**Tier:** Adapter (Layer 4)
**Publish:** No (internal)
**Dependencies:** http-core, axum, tower, tracing

## Purpose

Axum middleware components. Provides logging, tracing, metrics, and other cross-cutting HTTP concerns.

## Key Exports

- Logging middleware
- Tracing middleware
- Metrics middleware
- Request ID middleware

## When to Modify

- Adding new middleware components
- Extending existing middleware

## When NOT to Modify

- Adding routes (those go in http-* route crates)
- Adding business logic

## Architectural Notes

- **Tower middleware**: Compatible with Tower middleware ecosystem
- **Cross-cutting**: Applied to all routes via router

## Consumers

`app-http` (applies middleware to router)

## See Also

- `crates/http-core/` for base HTTP types
- `crates/app-http/` for middleware application
