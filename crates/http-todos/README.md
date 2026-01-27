# http-todos

HTTP handlers for `/todos/*` endpoints.

## Purpose

This crate implements todo management API including:

- **List todos** (`GET /todos`) - Get all todos as JSON array
- **Create todo** (`POST /todos`) - Create a new todo
- **Delete todo** (`DELETE /todos/:id`) - Delete a todo by ID
- **Clear todos** (`DELETE /todos/clear`) - Clear all todos

## Design Philosophy

- **Todo-focused**: Only todo-related handlers
- **In-memory storage**: Uses `Arc<RwLock<Vec<Todo>>>` for simplicity
- **Error handling**: Proper error propagation with structured error responses
- **Validation**: Validates required fields and title length

## Dependencies

- `axum` - HTTP web framework
- `http` - HTTP types
- `http-errors` - Error types with axum feature
- `platform-contract` - Platform contract types
- `http-core` - Core HTTP types
- `model` - Todo model type
- `serde` - Serialization
- `tokio` - Async runtime
- `uuid` - UUID generation

## Usage

```rust
use http_todos::{router, TodosStateTrait};

let app = Router::new()
    .merge(router(state));
```

## Public API

### Traits

- `TodosStateTrait` - Todos state trait for handlers

### Types

- `TodosState` - In-memory todo storage with CRUD operations
- `CreateTodoRequest` - Request body for creating a new todo

### Functions

- `router()` - Create the todos router
- `list_todos()` - List all todos endpoint handler
- `create_todo()` - Create a new todo endpoint handler
- `delete_todo()` - Delete a todo by ID endpoint handler
- `clear_todos()` - Clear all todos endpoint handler

## License

Internal crate (publish = false)
