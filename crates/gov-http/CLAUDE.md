# gov-http – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** Yes
**Dependencies:** gov-http-core, gov-http-types, gov-http-forks, gov-http-friction, gov-http-issues, gov-http-questions, axum

## Purpose

Platform HTTP router aggregating all governance endpoint sub-routes. Main entry point for `/platform/*` governance endpoints.

## Key Exports

- `governance_router()` – Combined governance router
- Route mounting utilities

## Aggregated Routes

- `/platform/forks/*` – Fork registry (gov-http-forks)
- `/platform/friction/*` – Friction tracking (gov-http-friction)
- `/platform/issues/*` – Issue management (gov-http-issues)
- `/platform/questions/*` – Question tracking (gov-http-questions)

## When to Modify

- Adding new governance endpoint sub-crates
- Changing route aggregation

## When NOT to Modify

- Adding individual endpoint logic (put in specific gov-http-* crate)

## Architectural Notes

- **Aggregator**: Combines sub-routers, doesn't implement endpoints
- **Modular**: Each domain has its own sub-crate

## Consumers

`app-http`

## See Also

- `crates/gov-http-*/` for individual endpoint implementations
- `crates/app-http/` for main HTTP router
