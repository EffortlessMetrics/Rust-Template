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

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
};
use http_errors::{ErrorCode, HttpError};
use model::Todo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

// ============================================================================
// Request DTOs
// ============================================================================

/// Request body for creating a new todo.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoRequest {
    /// Unique todo identifier
    pub id: String,
    /// Todo description
    pub title: String,
}

// ============================================================================
// State
// ============================================================================

/// Shared state for todos - in-memory storage.
#[derive(Clone)]
pub struct TodosState {
    todos: Arc<RwLock<Vec<Todo>>>,
}

/// Todos state trait for handlers.
///
/// This trait defines minimal interface required for todo handlers.
pub trait TodosStateTrait: Clone + Send + Sync + 'static {
    /// Get the inner TodosState.
    fn todos_state(&self) -> TodosState;
}

// ============================================================================
// Router
// ============================================================================

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

// ============================================================================
// Handlers
// ============================================================================

/// GET /todos - List all todos.
///
/// Returns a JSON array of todos.
#[instrument(skip(state))]
async fn list_todos<S>(State(state): State<(S, TodosState)>) -> Result<impl IntoResponse, HttpError>
where
    S: TodosStateTrait,
{
    info!("Listing all todos");

    let todos = state.1.get_all().await;

    info!(count = todos.len(), "Retrieved todos");

    Ok(Json(todos))
}

/// POST /todos - Create a new todo.
///
/// Creates a new todo with the provided id and title.
#[instrument(skip(state, payload))]
async fn create_todo<S>(
    State(state): State<(S, TodosState)>,
    payload: Result<Json<CreateTodoRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError>
where
    S: TodosStateTrait,
{
    // Handle JSON parsing/validation errors
    let Json(payload) = payload.map_err(|rejection| {
        warn!("Invalid JSON payload: {}", rejection);
        HttpError::validation_error(
            ErrorCode::InvalidRequest,
            format!("Invalid request: {}", rejection),
        )
    })?;

    info!(id = %payload.id, title = %payload.title, "Creating todo");

    // Validate required fields (empty string check)
    if payload.id.is_empty() {
        warn!("Missing required field: id");
        return Err(HttpError::bad_request("Missing required field: id"));
    }
    if payload.title.is_empty() {
        warn!("Missing required field: title");
        return Err(HttpError::bad_request("Missing required field: title"));
    }

    // Validate title length (max 256 characters)
    const MAX_TITLE_LENGTH: usize = 256;
    if payload.title.len() > MAX_TITLE_LENGTH {
        warn!(
            title_len = payload.title.len(),
            max_len = MAX_TITLE_LENGTH,
            "Title exceeds maximum length"
        );
        return Err(HttpError::new(
            400,
            ErrorCode::InvalidFormat,
            format!("Title must not exceed {} characters", MAX_TITLE_LENGTH),
        ));
    }

    // Check for duplicate ID
    if state.1.exists(&payload.id).await {
        warn!(id = %payload.id, "Duplicate todo ID");
        return Err(HttpError::new(
            409,
            ErrorCode::Conflict,
            format!("Todo with id '{}' already exists", payload.id),
        ));
    }

    let todo = Todo { id: payload.id.clone(), title: payload.title.clone() };

    state.1.add(todo.clone()).await;

    info!(id = %payload.id, "Todo created successfully");

    Ok((StatusCode::CREATED, Json(todo)))
}

/// DELETE /todos/:id - Delete a todo by ID.
///
/// Deletes the todo with the given ID.
#[instrument(skip(state))]
async fn delete_todo<S>(
    State(state): State<(S, TodosState)>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError>
where
    S: TodosStateTrait,
{
    info!(id = %id, "Deleting todo");

    let deleted = state.1.delete(&id).await;

    if deleted {
        info!(id = %id, "Todo deleted successfully");
        Ok(StatusCode::NO_CONTENT)
    } else {
        warn!(id = %id, "Todo not found");
        Err(HttpError::not_found(format!("Todo with id '{}' not found", id)))
    }
}

/// DELETE /todos/clear - Clear all todos.
///
/// Clears all todos from the in-memory store.
#[instrument(skip(state))]
async fn clear_todos<S>(
    State(state): State<(S, TodosState)>,
) -> Result<impl IntoResponse, HttpError>
where
    S: TodosStateTrait,
{
    info!("Clearing all todos");

    state.1.clear().await;

    info!("All todos cleared");
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// TodosState Implementation
// ============================================================================

impl TodosState {
    /// Create a new TodosState with sample data.
    pub fn new() -> Self {
        let todos = vec![
            Todo { id: "todo-1".to_string(), title: "Learn Rust-as-Spec patterns".to_string() },
            Todo { id: "todo-2".to_string(), title: "Implement AC-MYSERV-001".to_string() },
        ];

        Self { todos: Arc::new(RwLock::new(todos)) }
    }

    /// Create a new TodosState with empty todos.
    ///
    /// Used for testing empty array scenario.
    pub fn empty() -> Self {
        Self { todos: Arc::new(RwLock::new(vec![])) }
    }

    /// Get all todos.
    pub async fn get_all(&self) -> Vec<Todo> {
        self.todos.read().await.clone()
    }

    /// Add a new todo.
    pub async fn add(&self, todo: Todo) {
        self.todos.write().await.push(todo);
    }

    /// Check if a todo with the given ID exists.
    pub async fn exists(&self, id: &str) -> bool {
        self.todos.read().await.iter().any(|t| t.id == id)
    }

    /// Delete a todo by ID, returns true if found and deleted.
    pub async fn delete(&self, id: &str) -> bool {
        let mut guard = self.todos.write().await;
        let original_len = guard.len();
        guard.retain(|t| t.id != id);
        guard.len() < original_len
    }

    /// Clear all todos.
    pub async fn clear(&self) {
        self.todos.write().await.clear();
    }
}

impl Default for TodosState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todos_state_initialization() {
        let state = TodosState::new();
        let todos = state.get_all().await;

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, "todo-1");
        assert_eq!(todos[0].title, "Learn Rust-as-Spec patterns");
    }

    #[tokio::test]
    async fn test_todo_has_required_fields() {
        let state = TodosState::new();
        let todos = state.get_all().await;

        // Each todo must have id and title
        for todo in todos {
            assert!(!todo.id.is_empty(), "Todo must have id");
            assert!(!todo.title.is_empty(), "Todo must have title");
        }
    }

    #[tokio::test]
    async fn test_empty_todos_state() {
        // Empty list is valid
        let state = TodosState::empty();
        let todos = state.get_all().await;

        assert_eq!(todos.len(), 0);
    }

    #[tokio::test]
    async fn test_add_todo() {
        let state = TodosState::empty();
        let todo = Todo { id: "test-1".to_string(), title: "Test todo".to_string() };

        state.add(todo).await;

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let state = TodosState::new();

        let deleted = state.delete("todo-1").await;
        assert!(deleted, "Todo should be found and deleted");

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "todo-2");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_todo() {
        let state = TodosState::new();

        let deleted = state.delete("non-existent").await;
        assert!(!deleted, "Non-existent todo should not be found");

        let todos = state.get_all().await;
        assert_eq!(todos.len(), 2); // Original count unchanged
    }
}
