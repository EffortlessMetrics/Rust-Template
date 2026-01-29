# gov-http-forks – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** gov-http-core, gov-http-types, axum

## Purpose

Fork registry HTTP endpoints. Manages fork tracking and registration for the platform.

## Endpoints

- `GET /platform/forks` – List registered forks
- `POST /platform/forks` – Register a fork
- `GET /platform/forks/:id` – Get fork details

## When to Modify

- Adding new fork-related endpoints
- Changing fork management logic

## Consumers

`gov-http` (aggregates this router)

## See Also

- `crates/gov-http/` for router aggregation
- `crates/gov-http-core/` for shared utilities
