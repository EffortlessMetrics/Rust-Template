# http-tasks – CLAUDE.md

**Tier:** Application HTTP (Layer 6)
**Publish:** No (internal)
**Dependencies:** http-core, business-core, model, axum

## Purpose

Task CRUD HTTP handlers. Provides REST endpoints for task management.

## Key Endpoints

- `GET /tasks` – List tasks
- `POST /tasks` – Create task
- `GET /tasks/:id` – Get task
- `PUT /tasks/:id` – Update task
- `DELETE /tasks/:id` – Delete task

## When to Modify

- Adding task-related endpoints
- Changing task response format

## When NOT to Modify

- Adding business logic (put in business-core)
- Adding governance tasks (put in gov-http)

## Consumers

`app-http`

## See Also

- `crates/business-core/` for task repository
- `crates/model/` for task types
