//! Todo management handlers for MYSERV domain
//!
//! This module implements:
//! - AC-MYSERV-001: GET /todos returns a JSON array of todos
//! - AC-MYSERV-002: GET /todos returns empty array when no todos exist
//! - AC-MYSERV-003: Invalid payload returns 400 with structured error
//! - AC-MYSERV-004: DELETE /todos/:id removes the todo from the list
//!
//! # Design Notes
//!
//! - Uses in-memory storage (Arc<RwLock<Vec<Todo>>>) for simplicity
//! - Follows the hexagonal architecture pattern from lib.rs
//! - Demonstrates proper error handling with AppError
//! - Links to ACs for traceability

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{info, instrument, warn};

use crate::{AppError, AppState, ErrorCode};
use model::Todo;

/// Shared state for todos - in-memory storage
#[derive(Clone)]
pub struct TodosState {
    todos: Arc<RwLock<Vec<Todo>>>,
}

/// Request body for creating a new todo
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub id: String,
    pub title: String,
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

    /// Create a new TodosState with empty todos (for AC-MYSERV-002 testing)
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self { todos: Arc::new(RwLock::new(vec![])) }
    }

    /// Get all todos
    fn get_all(&self) -> Result<Vec<Todo>, AppError> {
        self.todos
            .read()
            .map_err(|e| AppError::internal_error(format!("Failed to acquire read lock: {}", e)))
            .map(|guard| guard.clone())
    }

    /// Add a new todo
    fn add(&self, todo: Todo) -> Result<(), AppError> {
        self.todos
            .write()
            .map_err(|e| AppError::internal_error(format!("Failed to acquire write lock: {}", e)))
            .map(|mut guard| guard.push(todo))
    }

    /// Delete a todo by ID, returns true if found and deleted
    fn delete(&self, id: &str) -> Result<bool, AppError> {
        self.todos
            .write()
            .map_err(|e| AppError::internal_error(format!("Failed to acquire write lock: {}", e)))
            .map(|mut guard| {
                let original_len = guard.len();
                guard.retain(|t| t.id != id);
                guard.len() < original_len
            })
    }

    /// Clear all todos (for AC-MYSERV-002 testing)
    fn clear(&self) -> Result<(), AppError> {
        self.todos
            .write()
            .map_err(|e| AppError::internal_error(format!("Failed to acquire write lock: {}", e)))
            .map(|mut guard| guard.clear())
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

    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", delete(delete_todo))
        .route("/todos/clear", delete(clear_todos))
        .with_state((app_state, todos_state))
}

/// GET /todos - List all todos
///
/// Implements AC-MYSERV-001: "GET /todos returns a JSON array of the user's todos"
/// Implements AC-MYSERV-002: "GET /todos returns an empty array when no todos exist"
///
/// # BDD Reference
/// Tagged with @AC-MYSERV-001 and @AC-MYSERV-002 in specs/features/myserv_todos.feature
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

/// POST /todos - Create a new todo
///
/// Implements AC-MYSERV-003: "Invalid payload returns 400 with structured error message"
///
/// # BDD Reference
/// Tagged with @AC-MYSERV-003 in specs/features/myserv_todos.feature
///
/// # Request Body
/// - `id`: Unique todo identifier (required)
/// - `title`: Todo description (required)
///
/// # Response
/// - 201 Created: Returns the created todo
/// - 400 Bad Request: Invalid JSON or missing required fields
#[instrument(skip(state, payload))]
async fn create_todo(
    State(state): State<(AppState, TodosState)>,
    payload: Result<Json<CreateTodoRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    // Handle JSON parsing/validation errors (AC-MYSERV-003)
    let Json(payload) = payload.map_err(|rejection| {
        warn!("Invalid JSON payload: {}", rejection);
        AppError::validation_error(
            ErrorCode::MissingField,
            format!("Invalid request: {}", rejection),
        )
        .with_ac_id("AC-MYSERV-003")
    })?;

    info!(id = %payload.id, title = %payload.title, "Creating todo");

    // Validate required fields (empty string check)
    if payload.id.is_empty() {
        warn!("Missing required field: id");
        return Err(AppError::bad_request("Missing required field: id"));
    }
    if payload.title.is_empty() {
        warn!("Missing required field: title");
        return Err(AppError::bad_request("Missing required field: title"));
    }

    let todo = Todo { id: payload.id.clone(), title: payload.title.clone() };

    state.1.add(todo.clone())?;

    info!(id = %payload.id, "Todo created successfully");

    Ok((StatusCode::CREATED, Json(todo)))
}

/// DELETE /todos/:id - Delete a todo by ID
///
/// Implements AC-MYSERV-004: "DELETE /todos/:id removes the todo from the list"
///
/// # BDD Reference
/// Tagged with @AC-MYSERV-004 in specs/features/myserv_todos.feature
///
/// # Response
/// - 204 No Content: Todo deleted successfully
/// - 404 Not Found: Todo with the given ID does not exist
#[instrument(skip(state))]
async fn delete_todo(
    State(state): State<(AppState, TodosState)>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!(id = %id, "Deleting todo");

    let deleted = state.1.delete(&id)?;

    if deleted {
        info!(id = %id, "Todo deleted successfully");
        Ok(StatusCode::NO_CONTENT)
    } else {
        warn!(id = %id, "Todo not found");
        Err(AppError::not_found(format!("Todo with id '{}' not found", id)))
    }
}

/// DELETE /todos/clear - Clear all todos (for testing AC-MYSERV-002)
///
/// This endpoint is primarily for testing to reset the todos state.
///
/// # Response
/// - 204 No Content: All todos cleared
#[instrument(skip(state))]
async fn clear_todos(
    State(state): State<(AppState, TodosState)>,
) -> Result<impl IntoResponse, AppError> {
    info!("Clearing all todos");

    state.1.clear()?;

    info!("All todos cleared");
    Ok(StatusCode::NO_CONTENT)
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

    #[test]
    fn test_empty_todos_state() {
        // AC-MYSERV-002: Empty list is valid
        let state = TodosState::empty();
        let todos = state.get_all().expect("Should get todos");

        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_add_todo() {
        let state = TodosState::empty();
        let todo = Todo { id: "test-1".to_string(), title: "Test todo".to_string() };

        state.add(todo).expect("Should add todo");

        let todos = state.get_all().expect("Should get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "test-1");
    }

    #[test]
    fn test_delete_todo() {
        // AC-MYSERV-004: Delete removes todo from list
        let state = TodosState::new();

        let deleted = state.delete("todo-1").expect("Should delete");
        assert!(deleted, "Todo should be found and deleted");

        let todos = state.get_all().expect("Should get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "todo-2");
    }

    #[test]
    fn test_delete_nonexistent_todo() {
        // AC-MYSERV-004: Deleting non-existent todo returns false
        let state = TodosState::new();

        let deleted = state.delete("non-existent").expect("Should execute");
        assert!(!deleted, "Non-existent todo should not be found");

        let todos = state.get_all().expect("Should get todos");
        assert_eq!(todos.len(), 2); // Original count unchanged
    }
}
