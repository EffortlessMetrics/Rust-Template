# http-tasks

HTTP handlers for `/tasks/*` endpoints.

## Purpose

This crate implements task management API including:

- **Task status update** (`/platform/tasks/{id}/status`) - Update task status via POST
- **Tasks UI endpoint** (`/ui/tasks`) - HTML-based Kanban task board

## Design Philosophy

- **Task-focused**: Only task-related handlers
- **Async-safe**: Uses `spawn_blocking` for blocking I/O operations
- **Error handling**: Proper error propagation and user messages
- **Dual format support**: Accepts both JSON and form-urlencoded request bodies

## Dependencies

- `axum` - HTTP web framework
- `http` - HTTP types
- `http-errors` - Error types with axum feature
- `platform-contract` - Platform contract types
- `http-core` - Core HTTP types
- `business-core` - Task operations via TaskService
- `serde_urlencoded` - Form data parsing
- `tracing` - Structured logging
- `tokio` - Async runtime

## Usage

```rust
use http_tasks::{router, TasksState, update_task_status};

let app = Router::new()
    .merge(router(state));
```

## Public API

### Traits

- `TasksState` - Tasks state trait for handlers

### Functions

- `router()` - Create tasks router
- `update_task_status()` - Update task status endpoint handler
- `tasks_ui()` - Tasks UI endpoint handler (HTML Kanban board)

### Request DTOs

- `UpdateTaskStatusRequest` - Request body for updating task status

## License

Internal crate (publish = false)
