# app-http – CLAUDE.md

**Tier:** Application (Layer 6)
**Publish:** Yes
**Dependencies:** All http-*crates, gov-http, adapters-*, business-core, spec-runtime, axum, tokio

## Purpose

Main HTTP router and application entry point. Aggregates all routes, applies middleware, and provides the runnable HTTP server.

## Key Exports

- `main()` – Application entry point
- `create_router()` – Full application router
- Server configuration and startup

## Aggregated Routes

- `/` – Health check
- `/platform/*` – Platform status, governance, specs (http-platform, gov-http)
- `/tasks/*` – Task CRUD (http-tasks)
- `/todos/*` – Todo CRUD (http-todos)
- `/agents/*` – Agent hints (http-agents)

## When to Modify

- Adding new route aggregations
- Changing server configuration
- Modifying middleware stack

## When NOT to Modify

- Adding specific endpoint logic (put in appropriate http-* crate)
- Adding business logic (put in business-core)

## Architectural Notes

- **Entry point**: Where the binary starts
- **Aggregator**: Combines all sub-routers
- **Configuration**: Server bind, TLS, timeouts

## Running

```bash
cargo run -p app-http
# Server starts at http://localhost:8080
```

## Consumers

This is the top-level HTTP application—no other crates depend on it.

## See Also

- `crates/http-*/` for individual endpoint crates
- `crates/gov-http/` for governance endpoints
- `CLAUDE.md` (root) for platform API usage
