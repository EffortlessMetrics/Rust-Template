# gov-http-friction – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** gov-http-core, gov-http-types, axum

## Purpose

Friction entry HTTP endpoints. Manages friction log entries tracking developer experience issues.

## Endpoints

- `GET /platform/friction` – List friction entries
- `POST /platform/friction` – Create friction entry
- `GET /platform/friction/:id` – Get friction details
- `PATCH /platform/friction/:id` – Update friction status

## When to Modify

- Adding new friction-related endpoints
- Changing friction management logic

## Key Files

- `friction/` directory for friction YAML files

## Consumers

`gov-http` (aggregates this router)

## See Also

- `crates/gov-http/` for router aggregation
- `cargo xtask friction-list` for CLI access
