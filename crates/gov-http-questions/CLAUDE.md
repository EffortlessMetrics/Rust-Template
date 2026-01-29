# gov-http-questions – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** gov-http-core, gov-http-types, axum

## Purpose

Question/clarification HTTP endpoints. Tracks questions that arise during development needing resolution.

## Endpoints

- `GET /platform/questions` – List questions
- `POST /platform/questions` – Create question
- `GET /platform/questions/:id` – Get question details
- `PATCH /platform/questions/:id` – Update question status

## When to Modify

- Adding new question-related endpoints
- Changing question management logic

## Consumers

`gov-http` (aggregates this router)

## See Also

- `crates/gov-http/` for router aggregation
- `cargo xtask question-new` for CLI access
