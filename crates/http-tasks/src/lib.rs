//! HTTP handlers for `/tasks/*` endpoints.
//!
//! This crate implements task management API including:
//! - Task status update endpoint
//! - Tasks UI endpoint (HTML-based task board)
//!
//! # Design Philosophy
//!
//! - **Task-focused**: Only task-related handlers
//! - **Async-safe**: Uses `spawn_blocking` for blocking I/O
//! - **Error handling**: Proper error propagation and user messages
//!
//! # Example
//!
//! ```rust,ignore
//! use http_tasks::router;
//!
//! let app = Router::new().merge(router(state));
//! ```

use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
};
use business_core::governance::{TaskId, TaskService};
use http_errors::{ErrorCode, HttpError};
use http_task_board::render_task_board;
use http_task_status_parser::parse_update_task_status as parse_task_status_request;
use tracing::instrument;

// ============================================================================
// Request DTOs
// ============================================================================

pub use http_task_status_parser::UpdateTaskStatusRequest;

// ============================================================================
// State Trait
// ============================================================================

/// Tasks state trait for handlers.
///
/// This trait defines minimal interface required for task handlers.
pub trait TasksState: Clone + Send + Sync + 'static {
    /// Get governance repository.
    fn governance_repo(
        &self,
    ) -> std::sync::Arc<dyn business_core::governance::GovernanceRepository>;
}

// ============================================================================
// Router
// ============================================================================

/// Create the tasks router.
///
/// This router will be merged into the main application router.
pub fn router<S>(state: S) -> Router<S>
where
    S: TasksState + Clone + 'static,
{
    Router::new().route("/ui/tasks", get(tasks_ui::<S>)).with_state(state)
}

// ============================================================================
// Handlers
// ============================================================================

/// Update task status endpoint.
///
/// Handles both JSON and form-urlencoded request bodies.
#[instrument(skip(state, headers, body), fields(task_id = %id))]
pub async fn update_task_status<S>(
    State(state): State<S>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, HttpError>
where
    S: TasksState,
{
    let payload = parse_update_task_status(&headers, &body)?;
    let repo = state.governance_repo();
    let task_id = TaskId(id);
    let new_status = payload.status;

    // Offload blocking file I/O (fs2 locks + std::fs) to spawn_blocking
    // to avoid starving the Tokio executor under concurrent load.
    spawn_blocking_io("move_task", move || {
        let service = TaskService::new(repo);
        service.move_task(&task_id, new_status)
    })
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Tasks UI endpoint (HTML task board).
///
/// Returns an HTML page with a Kanban-style task board.
#[instrument(skip(state))]
pub async fn tasks_ui<S>(State(state): State<S>) -> Result<impl IntoResponse, HttpError>
where
    S: TasksState,
{
    let repo = state.governance_repo();

    // Offload blocking file I/O to spawn_blocking to avoid starving the executor.
    let tasks = spawn_blocking_io("list_tasks", move || {
        let service = TaskService::new(repo);
        service.list_tasks()
    })
    .await?;

    Ok(Html(render_task_board(tasks)))
}

// = ===========================================================================
// Internal Helpers
// ============================================================================

fn map_gov_error(error: business_core::governance::GovernanceError) -> HttpError {
    use business_core::governance::GovernanceError::*;
    match error {
        TaskNotFound(id) => HttpError::not_found(format!("Task not found: {:?}", id)),
        InvalidTransition { from, to } => HttpError::new(
            400,
            ErrorCode::InvalidTransition,
            format!("Invalid status transition from {} to {}", from, to),
        ),
        Lock(msg) => HttpError::internal_error(format!("Lock error: {}", msg)),
        Io(e) => HttpError::internal_error(format!("IO error: {}", e)),
        Serialization(msg) => HttpError::internal_error(format!("Serialization error: {}", msg)),
    }
}

/// Execute a blocking closure on a dedicated thread pool.
///
/// This wraps sync repository I/O (fs2 file locks, std::fs)
/// that cannot be safely called from async context.
///
/// The label is used for error messages on join failure.
async fn spawn_blocking_io<T, F>(label: &'static str, f: F) -> Result<T, HttpError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, business_core::governance::GovernanceError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| HttpError::internal_error(format!("spawn_blocking({label}) join: {e}")))?
        .map_err(map_gov_error)
}

/// Parse update task status request from headers and body.
///
/// Supports both JSON and form-urlencoded content types.
#[allow(clippy::result_large_err)] // AppError is shared across handlers; keep signature consistent
fn parse_update_task_status(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<UpdateTaskStatusRequest, HttpError> {
    let content_type =
        headers.get(axum::http::header::CONTENT_TYPE).and_then(|header| header.to_str().ok());

    parse_task_status_request(content_type, body)
        .map_err(|error| HttpError::validation_error(ErrorCode::InvalidRequest, error.to_string()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use business_core::governance::TaskStatus;

    #[test]
    fn test_parse_update_task_status_json() {
        let body = br#"{"status":"InProgress"}"#;
        let headers = HeaderMap::from_iter([(
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        )]);

        let result = parse_update_task_status(&headers, body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::InProgress);
    }

    #[test]
    fn test_parse_update_task_status_form() {
        let body = b"status=InProgress";
        let headers = HeaderMap::from_iter([(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        )]);

        let result = parse_update_task_status(&headers, body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::InProgress);
    }

    #[test]
    fn test_parse_update_task_status_fallback() {
        let body = br#"{"status":"Done"}"#;
        let headers = HeaderMap::new();

        let result = parse_update_task_status(&headers, body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::Done);
    }
}
