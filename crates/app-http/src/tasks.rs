use crate::AppError;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use business_core::governance::{GovernanceRepository, TaskId, TaskService, TaskStatus};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateTaskStatusRequest {
    status: TaskStatus,
}

pub async fn update_task_status(
    State(repo): State<Arc<dyn GovernanceRepository>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTaskStatusRequest>,
) -> Result<impl IntoResponse, AppError> {
    let service = TaskService::new(repo);
    service.move_task(&TaskId(id), body.status)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn tasks_ui(
    State(repo): State<Arc<dyn GovernanceRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let service = TaskService::new(repo);
    let tasks = service.list_tasks()?;

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
