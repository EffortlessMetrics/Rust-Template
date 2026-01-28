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
use business_core::governance::{TaskId, TaskService, TaskStatus};
use http_errors::{ErrorCode, HttpError};
use serde::Deserialize;
use tracing::instrument;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request body for updating task status.
#[derive(Deserialize)]
pub struct UpdateTaskStatusRequest {
    /// New task status
    pub status: TaskStatus,
}

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

    let mut todo = Vec::new();
    let mut in_progress = Vec::new();
    let mut review = Vec::new();
    let mut done = Vec::new();

    for task in tasks {
        match task.status {
            TaskStatus::Todo => todo.push(task),
            TaskStatus::InProgress => in_progress.push(task),
            TaskStatus::Review => review.push(task),
            TaskStatus::Done => done.push(task),
        }
    }

    let render_column = |title: &str, tasks: Vec<business_core::governance::Task>| -> String {
        let tasks_html = tasks.into_iter().map(|t| {
            let buttons = match t.status {
                TaskStatus::Todo => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "InProgress"}}' hx-target="body">Start</button>"#, t.id.0),
                TaskStatus::InProgress => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Review"}}' hx-target="body">Review</button>"#, t.id.0),
                TaskStatus::Review => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Done"}}' hx-target="body">Done</button>"#, t.id.0),
                TaskStatus::Done => String::new(),
            };

            format!(
                r#"<div class="task-card">
                    <h3>{}</h3>
                    <p>{}</p>
                    <div class="actions">{}</div>
                </div>"#,
                t.id.0, t.title, buttons
            )
        }).collect::<Vec<_>>().join("\n");

        format!(
            r#"<div class="column">
                <h2>{}</h2>
                <div class="task-list">
                    {}
                </div>
            </div>"#,
            title, tasks_html
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Task Board</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        body {{ font-family: sans-serif; padding: 20px; }}
        .board {{ display: flex; gap: 20px; }}
        .column {{ flex: 1; background: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .task-card {{ background: white; padding: 10px; margin-bottom: 10px; border-radius: 3px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .actions {{ margin-top: 10px; }}
        button {{ cursor: pointer; padding: 5px 10px; }}
    </style>
</head>
<body>
    <h1>Task Board</h1>
    <div class="board">
        {}
        {}
        {}
        {}
    </div>
</body>
</html>"#,
        render_column("Todo", todo),
        render_column("In Progress", in_progress),
        render_column("Review", review),
        render_column("Done", done)
    );

    Ok(Html(html))
}

// ============================================================================
// Internal Helpers
// ============================================================================

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
        .map_err(|e| HttpError::internal_error(format!("{:?}", e)))
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
        headers.get(axum::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("");

    if content_type.starts_with("application/json") {
        return serde_json::from_slice(body).map_err(|err| {
            HttpError::validation_error(ErrorCode::InvalidRequest, format!("Invalid JSON: {}", err))
        });
    }

    if content_type.starts_with("application/x-www-form-urlencoded") {
        return serde_urlencoded::from_bytes(body).map_err(|err| {
            HttpError::validation_error(
                ErrorCode::InvalidRequest,
                format!("Invalid form data: {}", err),
            )
        });
    }

    // Fallback: try to parse as JSON first, then form data to be forgiving
    if let Ok(value) = serde_json::from_slice(body) {
        return Ok(value);
    }

    if let Ok(value) = serde_urlencoded::from_bytes(body) {
        return Ok(value);
    }

    Err(HttpError::validation_error(
        ErrorCode::InvalidRequest,
        "Unsupported body format; use JSON or x-www-form-urlencoded",
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
