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
use todo_store::TodosState;
use tracing::{info, instrument, warn};

use crate::{AppError, AppState, ErrorCode};
use model::Todo;

/// Request body for creating a new todo
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub id: String,
    pub title: String,
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

    let todos = state.1.get_all().await;

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
        return Err(
            AppError::bad_request("Missing required field: title").with_ac_id("AC-MYSERV-006")
        );
    }

    // AC-MYSERV-006: Validate title length (max 256 characters)
    const MAX_TITLE_LENGTH: usize = 256;
    if payload.title.len() > MAX_TITLE_LENGTH {
        warn!(
            title_len = payload.title.len(),
            max_len = MAX_TITLE_LENGTH,
            "Title exceeds maximum length"
        );
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidFormat,
            format!("Title must not exceed {} characters", MAX_TITLE_LENGTH),
        )
        .with_ac_id("AC-MYSERV-006"));
    }

    // AC-MYSERV-005: Check for duplicate ID
    if state.1.exists(&payload.id).await {
        warn!(id = %payload.id, "Duplicate todo ID");
        return Err(AppError::new(
            StatusCode::CONFLICT,
            ErrorCode::Conflict,
            format!("Todo with id '{}' already exists", payload.id),
        )
        .with_ac_id("AC-MYSERV-005"));
    }

    let todo = Todo { id: payload.id.clone(), title: payload.title.clone() };

    state.1.add(todo.clone()).await;

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

    let deleted = state.1.delete(&id).await;

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

    state.1.clear().await;

    info!("All todos cleared");
    Ok(StatusCode::NO_CONTENT)
}

