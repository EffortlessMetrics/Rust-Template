//! HTTP handler functions for todo endpoints.

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use http_errors::{ErrorCode, HttpError};
use tracing::{info, instrument, warn};

use crate::{CreateTodoRequest, TodosState, TodosStateTrait};

/// GET /todos - List all todos.
///
/// Returns a JSON array of todos.
#[instrument(skip(state))]
pub(crate) async fn list_todos<S>(
    State(state): State<(S, TodosState)>,
) -> Result<impl IntoResponse, HttpError>
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
pub(crate) async fn create_todo<S>(
    State(state): State<(S, TodosState)>,
    payload: Result<Json<CreateTodoRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError>
where
    S: TodosStateTrait,
{
    let Json(payload) = payload.map_err(|rejection| {
        warn!("Invalid JSON payload: {}", rejection);
        HttpError::validation_error(
            ErrorCode::InvalidRequest,
            format!("Invalid request: {}", rejection),
        )
    })?;

    info!(id = %payload.id, title = %payload.title, "Creating todo");

    if let Err(error) = payload.validate() {
        warn!(id = %payload.id, title_len = payload.title.len(), "Invalid todo payload");
        return Err(error);
    }

    match state.1.try_add(payload.id, payload.title).await {
        Ok(todo) => {
            info!(id = %todo.id, "Todo created successfully");
            Ok((StatusCode::CREATED, Json(todo)))
        }
        Err(id) => {
            warn!(id = %id, "Duplicate todo ID");
            Err(HttpError::new(
                409,
                ErrorCode::Conflict,
                format!("Todo with id '{}' already exists", id),
            ))
        }
    }
}

/// DELETE /todos/:id - Delete a todo by ID.
///
/// Deletes the todo with the given ID.
#[instrument(skip(state))]
pub(crate) async fn delete_todo<S>(
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
pub(crate) async fn clear_todos<S>(
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
