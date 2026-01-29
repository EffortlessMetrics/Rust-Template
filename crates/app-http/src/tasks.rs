use crate::{AppError, ErrorCode};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use business_core::governance::{TaskId, TaskService, TaskStatus};
use serde::Deserialize;
use tracing::instrument;
use std::fmt::Write;

use crate::AppState;

/// Execute a blocking closure on a dedicated thread pool to avoid blocking the
/// Tokio async runtime. This wraps sync repository I/O (fs2 file locks, std::fs)
/// that cannot be safely called from async context.
///
/// The label is used for error messages on join failure.
async fn spawn_blocking_io<T, F>(label: &'static str, f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, business_core::governance::GovernanceError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::internal_error(format!("spawn_blocking({label}) join: {e}")))?
        .map_err(AppError::from)
}

#[derive(Deserialize)]
pub struct UpdateTaskStatusRequest {
    status: TaskStatus,
}

#[instrument(skip(state, headers, body), fields(task_id = %id))]
pub async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let payload = parse_update_task_status(&headers, &body)?;
    let repo = state.governance_repo.clone();
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

#[allow(clippy::result_large_err)] // AppError is shared across handlers; keep signature consistent
fn parse_update_task_status(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<UpdateTaskStatusRequest, AppError> {
    let content_type =
        headers.get(axum::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("");

    if content_type.starts_with("application/json") {
        return serde_json::from_slice(body).map_err(|err| {
            AppError::validation_error(ErrorCode::InvalidRequest, format!("Invalid JSON: {}", err))
        });
    }

    if content_type.starts_with("application/x-www-form-urlencoded") {
        return serde_urlencoded::from_bytes(body).map_err(|err| {
            AppError::validation_error(
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

    Err(AppError::validation_error(
        ErrorCode::InvalidRequest,
        "Unsupported body format; use JSON or x-www-form-urlencoded",
    ))
}

pub async fn tasks_ui(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let repo = state.governance_repo.clone();

    // Offload blocking file I/O to spawn_blocking to avoid starving the executor.
    let tasks = spawn_blocking_io("list_tasks", move || {
        let service = TaskService::new(repo);
        service.list_tasks()
    })
    .await?;

    let mut html = String::with_capacity(tasks.len() * 250 + 1000);

    html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <title>Task Board</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        body { font-family: sans-serif; padding: 20px; }
        .board { display: flex; gap: 20px; }
        .column { flex: 1; background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .task-card { background: white; padding: 10px; margin-bottom: 10px; border-radius: 3px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .actions { margin-top: 10px; }
        button { cursor: pointer; padding: 5px 10px; }
    </style>
</head>
<body>
    <h1>Task Board</h1>
    <div class="board">"#);

    let render_column = |title: &str, filter: TaskStatus, buffer: &mut String| {
        let _ = write!(buffer, r#"<div class="column">
                <h2>{}</h2>
                <div class="task-list">"#, title);

        for task in &tasks {
            if task.status == filter {
                let _ = write!(buffer, r#"<div class="task-card">
                    <h3>{}</h3>
                    <p>{}</p>
                    <div class="actions">"#, task.id.0, task.title);

                match task.status {
                    TaskStatus::Todo => {
                        let _ = write!(buffer, r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "InProgress"}}' hx-target="body">Start</button>"#, task.id.0);
                    },
                    TaskStatus::InProgress => {
                        let _ = write!(buffer, r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Review"}}' hx-target="body">Review</button>"#, task.id.0);
                    },
                    TaskStatus::Review => {
                        let _ = write!(buffer, r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Done"}}' hx-target="body">Done</button>"#, task.id.0);
                    },
                    TaskStatus::Done => {},
                }
                buffer.push_str("</div></div>");
            }
        }
        buffer.push_str("</div></div>");
    };

    render_column("Todo", TaskStatus::Todo, &mut html);
    render_column("In Progress", TaskStatus::InProgress, &mut html);
    render_column("Review", TaskStatus::Review, &mut html);
    render_column("Done", TaskStatus::Done, &mut html);

    html.push_str(r#"</div></body></html>"#);

    Ok(Html(html))
}
