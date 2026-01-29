# http-core – CLAUDE.md

**Tier:** Adapter (Layer 4)
**Publish:** No (internal)
**Dependencies:** http-errors, axum, serde

## Purpose

Base HTTP types, middleware, and platform integration. Non-routable utilities shared across HTTP crates.

## Key Exports

- Common HTTP types
- Request/response utilities
- Platform integration helpers
- Shared extractors

## When to Modify

- Adding shared HTTP utilities
- Extending common types

## When NOT to Modify

- Adding routes (those go in specific http-* crates)
- Adding domain logic

## Architectural Notes

- **Non-routable**: Provides utilities, not endpoints
- **Shared foundation**: Used by all http-* crates
- **Axum-based**: Built on Axum web framework

## Consumers

`http-middleware`, `http-platform`, `http-tasks`, `http-todos`, `http-agents`, `app-http`

## See Also

- `crates/http-errors/` for error types
- `crates/http-middleware/` for middleware components
