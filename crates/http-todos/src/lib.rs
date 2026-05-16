//! HTTP handlers for `/todos/*` endpoints.
//!
//! This crate implements todo management API including:
//! - List todos
//! - Create todo
//! - Delete todo
//! - Clear todos
//!
//! # Design Philosophy
//!
//! - **Todo-focused**: Only todo-related handlers
//! - **In-memory storage**: Uses Arc<RwLock<Vec<Todo>>> for simplicity
//! - **Error handling**: Proper error propagation and user messages
//!
//! # Example
//!
//! ```rust,ignore
//! use http_todos::router;
//!
//! let app = Router::new().merge(router(state));
//! ```

mod dto;
mod handlers;
mod state;

use axum::{
    Router,
    routing::{delete, get},
};

pub use dto::CreateTodoRequest;
pub use state::{TodosState, TodosStateTrait};

use handlers::{clear_todos, create_todo, delete_todo, list_todos};

/// Create the todos router.
///
/// This router will be merged into the main application router.
pub fn router<S>(app_state: S) -> Router<S>
where
    S: TodosStateTrait + Clone + 'static,
{
    let todos_state = TodosState::new();

    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", delete(delete_todo))
        .route("/todos/clear", delete(clear_todos))
        .with_state((app_state, todos_state))
}
