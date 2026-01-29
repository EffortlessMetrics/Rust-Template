# gov-http-issues – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** gov-http-core, gov-http-types, axum

## Purpose

Issue tracking HTTP endpoints. Manages issues linked to governance artifacts.

## Endpoints

- `GET /platform/issues` – List issues
- `POST /platform/issues` – Create issue
- `GET /platform/issues/:id` – Get issue details

## When to Modify

- Adding new issue-related endpoints
- Changing issue management logic

## Consumers

`gov-http` (aggregates this router)

## See Also

- `crates/gov-http/` for router aggregation
- GitHub issues integration
