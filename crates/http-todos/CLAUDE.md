# http-todos – CLAUDE.md

**Tier:** Application HTTP (Layer 6)
**Publish:** No (internal)
**Dependencies:** http-core, business-core, model, axum

## Purpose

Todo item HTTP handlers and routes. Example CRUD implementation for simple todo items.

## Key Endpoints

- `GET /todos` – List todos
- `POST /todos` – Create todo
- `GET /todos/:id` – Get todo
- `PUT /todos/:id` – Update todo
- `DELETE /todos/:id` – Delete todo

## When to Modify

- Adding todo-related endpoints
- Changing todo response format

## Architectural Notes

- **Example implementation**: Demonstrates CRUD patterns
- **Simple domain**: Good reference for new endpoints

## Consumers

`app-http`

## See Also

- `crates/http-tasks/` for similar task implementation
- `crates/business-core/` for repository patterns
