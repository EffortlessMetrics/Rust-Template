//! Task board HTML rendering for `/ui/tasks` endpoint.
//!
//! This crate is intentionally focused on presentation concerns only.

use business_core::governance::{Task, TaskStatus};

/// Render a kanban-style task board as full HTML document.
pub fn render_task_board(tasks: Vec<Task>) -> String {
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

    format!(
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
    )
}

fn render_column(title: &str, tasks: Vec<Task>) -> String {
    let tasks_html = tasks
        .into_iter()
        .map(|task| {
            let buttons = match task.status {
                TaskStatus::Todo => format!(
                    r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "InProgress"}}' hx-target="body">Start</button>"#,
                    task.id.0
                ),
                TaskStatus::InProgress => format!(
                    r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Review"}}' hx-target="body">Review</button>"#,
                    task.id.0
                ),
                TaskStatus::Review => format!(
                    r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Done"}}' hx-target="body">Done</button>"#,
                    task.id.0
                ),
                TaskStatus::Done => String::new(),
            };

            format!(
                r#"<div class="task-card">
                    <h3>{}</h3>
                    <p>{}</p>
                    <div class="actions">{}</div>
                </div>"#,
                task.id.0, task.title, buttons
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div class="column">
                <h2>{}</h2>
                <div class="task-list">
                    {}
                </div>
            </div>"#,
        title, tasks_html
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use business_core::governance::{Task, TaskId};

    #[test]
    fn renders_each_status_column() {
        let html = render_task_board(vec![]);
        assert!(html.contains("<h2>Todo</h2>"));
        assert!(html.contains("<h2>In Progress</h2>"));
        assert!(html.contains("<h2>Review</h2>"));
        assert!(html.contains("<h2>Done</h2>"));
    }

    #[test]
    fn renders_transition_buttons_for_non_done_tasks() {
        let tasks = vec![
            task("t1", "todo", TaskStatus::Todo),
            task("t2", "in progress", TaskStatus::InProgress),
            task("t3", "review", TaskStatus::Review),
            task("t4", "done", TaskStatus::Done),
        ];

        let html = render_task_board(tasks);
        assert!(html.contains("/platform/tasks/t1/status"));
        assert!(html.contains("Start</button>"));
        assert!(html.contains("Review</button>"));
        assert!(html.contains("Done</button>"));

        let done_card_start = html.find("<h3>t4</h3>").unwrap();
        let done_card = &html[done_card_start..html.len().min(done_card_start + 200)];
        assert!(!done_card.contains("<button"));
    }

    fn task(id: &str, title: &str, status: TaskStatus) -> Task {
        Task { id: TaskId(id.to_string()), title: title.to_string(), status }
    }
}
