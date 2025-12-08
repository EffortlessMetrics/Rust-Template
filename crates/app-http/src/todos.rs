//! Todo management handlers for MYSERV domain
//!
//! This module implements AC-MYSERV-001: GET /todos returns a JSON array of todos.
//!
//! # Design Notes
//!
//! - Uses in-memory storage (Arc<RwLock<Vec<Todo>>>) for simplicity
//! - Follows the hexagonal architecture pattern from lib.rs
//! - Demonstrates proper error handling with AppError
//! - Links to AC-MYSERV-001 for traceability

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use std::sync::{Arc, RwLock};
use tracing::{info, instrument};

use crate::{AppError, AppState};
use model::Todo;

/// Shared state for todos - in-memory storage
#[derive(Clone)]
pub struct TodosState {
    todos: Arc<RwLock<Vec<Todo>>>,
}

impl TodosState {
    /// Create a new TodosState with sample data
    pub fn new() -> Self {
        let todos = vec![
            Todo { id: "todo-1".to_string(), title: "Learn Rust-as-Spec patterns".to_string() },
            Todo { id: "todo-2".to_string(), title: "Implement AC-MYSERV-001".to_string() },
        ];

        Self { todos: Arc::new(RwLock::new(todos)) }
    }

    /// Get all todos
    fn get_all(&self) -> Result<Vec<Todo>, AppError> {
        self.todos
            .read()
            .map_err(|e| AppError::internal_error(format!("Failed to acquire read lock: {}", e)))
            .map(|guard| guard.clone())
    }
}

impl Default for TodosState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the todos router
///
/// This router will be merged into the main application router.
pub fn router(app_state: AppState) -> Router<AppState> {
    let todos_state = TodosState::new();

    Router::new().route("/todos", get(list_todos)).with_state((app_state, todos_state))
}

/// GET /todos - List all todos
///
/// Implements AC-MYSERV-001: "GET /todos returns a JSON array of the user's todos"
///
/// # BDD Reference
/// Tagged with @AC-MYSERV-001 in specs/features/myserv_todos.feature
///
/// # Response
/// Returns a JSON array of todos, each with:
/// - `id`: Unique todo identifier
/// - `title`: Todo description
#[instrument(skip(state))]
async fn list_todos(
    State(state): State<(AppState, TodosState)>,
) -> Result<impl IntoResponse, AppError> {
    info!("Listing all todos");

    let todos = state.1.get_all()?;

    info!(count = todos.len(), "Retrieved todos");

    Ok(Json(todos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todos_state_initialization() {
        let state = TodosState::new();
        let todos = state.get_all().expect("Should get todos");

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, "todo-1");
        assert_eq!(todos[0].title, "Learn Rust-as-Spec patterns");
    }

    #[test]
    fn test_todo_has_required_fields() {
        let state = TodosState::new();
        let todos = state.get_all().expect("Should get todos");

        // AC-MYSERV-001: Each todo must have id and title
        for todo in todos {
            assert!(!todo.id.is_empty(), "Todo must have id");
            assert!(!todo.title.is_empty(), "Todo must have title");
        }
    }
}
