use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use gov_http_core::{PlatformError, PlatformState};
use gov_model::TaskStatus;

use crate::conversion::task_to_issue;
use crate::filtering::{apply_filters, calculate_summary};
use crate::loading::{load_friction_entries, load_question_entries};
use crate::models::{Issue, IssueFilters, IssuesResponse, Pagination};

/// Router for unified issues endpoint.
///
/// Returns a router that handles:
/// - `GET /issues` - List all issues with filtering and pagination
pub fn router<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new().route("/issues", get(get_issues::<S>))
}

/// GET /issues - Get all issues with filtering and pagination
async fn get_issues<S>(
    State(state): State<S>,
    Query(filters): Query<IssueFilters>,
) -> Result<Json<IssuesResponse>, PlatformError>
where
    S: PlatformState,
{
    let issues = collect_issues(state).await?;
    let filtered = apply_filters(issues, &filters);
    let summary = calculate_summary(&filtered);
    let (issues, pagination) = sort_and_paginate(filtered, &filters);

    Ok(Json(IssuesResponse { issues, pagination, summary }))
}

async fn collect_issues<S>(state: S) -> Result<Vec<Issue>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let root = ctx.root().to_path_buf();
    let repo = state.governance_repo();

    let (friction_entries, questions) = tokio::task::spawn_blocking(move || {
        let friction = load_friction_entries(&root)?;
        let questions = load_question_entries(&root)?;
        Ok::<_, PlatformError>((friction, questions))
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    let tasks_spec = spec_runtime::load_tasks_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("tasks.yaml", e))?;

    let all_tasks = repo
        .find_all_tasks()
        .map_err(|e| PlatformError::internal(format!("Failed to load task states: {}", e)))?;

    let status_map = task_status_map(all_tasks);
    let mut issues = Vec::new();
    issues.extend(friction_entries.into_iter().map(Issue::from));
    issues.extend(questions.into_iter().map(Issue::from));
    issues.extend(tasks_spec.tasks.iter().map(|task| {
        let effective_status = status_map
            .get(&task.id)
            .map(|status| format!("{:?}", status))
            .unwrap_or_else(|| task.status.clone());
        task_to_issue(task, &effective_status)
    }));

    Ok(issues)
}

fn task_status_map(tasks: Vec<gov_model::Task>) -> HashMap<String, TaskStatus> {
    tasks.into_iter().map(|task| (task.id.0, task.status)).collect()
}

fn sort_and_paginate(mut issues: Vec<Issue>, filters: &IssueFilters) -> (Vec<Issue>, Pagination) {
    issues.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });

    let total_items = issues.len();
    let per_page = filters.per_page.min(100);
    let total_pages = if total_items == 0 { 0 } else { (total_items as u32).div_ceil(per_page) };
    let page = filters.page.max(1);
    let skip = ((page - 1) * per_page) as usize;
    let issues = issues.into_iter().skip(skip).take(per_page as usize).collect();

    (issues, Pagination { page, per_page, total_items, total_pages })
}
