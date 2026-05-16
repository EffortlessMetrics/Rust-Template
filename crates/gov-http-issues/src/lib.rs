//! Unified issues endpoint aggregating friction, questions, and tasks.
//!
//! This crate provides a single `/issues` endpoint that normalizes all three
//! artifact types into a common `Issue` representation with unified status,
//! priority, and filtering capabilities.
//!
//! # Endpoints
//!
//! - `GET /issues` - List all issues with filtering and pagination

mod conversion;
mod filtering;
mod loading;
mod models;

use axum::{Json, Router, extract::Query, extract::State, routing::get};
use gov_http_core::{PlatformError, PlatformState};
use gov_model::TaskStatus;
use std::collections::HashMap;

use conversion::task_to_issue;
use filtering::{apply_filters, calculate_summary, sort_issues};
use loading::{load_friction_entries, load_question_entries};

pub use models::{
    Issue, IssueFilters, IssueKind, IssueStatus, IssuesResponse, IssuesSummary, KindCounts,
    Pagination, StatusCounts,
};

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
    let ctx = state.context();
    let root = ctx.root().to_path_buf(); // Clone for spawn_blocking
    let repo = state.governance_repo();

    // Load friction and questions in a blocking task to avoid blocking Tokio workers
    // on filesystem I/O (YAML parsing).
    let (friction_entries, questions) = tokio::task::spawn_blocking(move || {
        let friction = load_friction_entries(&root)?;
        let questions = load_question_entries(&root)?;
        Ok::<_, PlatformError>((friction, questions))
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    // Load tasks with status overlay (spec_runtime uses ctx which may not be Send)
    let tasks_spec = spec_runtime::load_tasks_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("tasks.yaml", e))?;

    let all_tasks = repo
        .find_all_tasks()
        .map_err(|e| PlatformError::internal(format!("Failed to load task states: {}", e)))?;

    let status_map: HashMap<String, TaskStatus> =
        all_tasks.into_iter().map(|t| (t.id.0, t.status)).collect();

    let issues = collect_issues(friction_entries, questions, &tasks_spec.tasks, &status_map);
    let filtered = apply_filters(issues, &filters);
    let summary = calculate_summary(&filtered);

    let mut sorted = filtered;
    sort_issues(&mut sorted);

    let (pagination, issues) = paginate_issues(sorted, filters.page, filters.per_page);

    Ok(Json(IssuesResponse { issues, pagination, summary }))
}

fn collect_issues(
    friction_entries: Vec<gov_http_types::FrictionEntry>,
    questions: Vec<gov_http_types::Question>,
    tasks: &[spec_runtime::Task],
    status_map: &HashMap<String, TaskStatus>,
) -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();
    issues.extend(friction_entries.into_iter().map(Issue::from));
    issues.extend(questions.into_iter().map(Issue::from));
    issues.extend(tasks.iter().map(|task| {
        let effective_status = status_map
            .get(&task.id)
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| task.status.clone());
        task_to_issue(task, &effective_status)
    }));
    issues
}

fn paginate_issues(
    sorted: Vec<Issue>,
    requested_page: u32,
    requested_per_page: u32,
) -> (Pagination, Vec<Issue>) {
    let total_items = sorted.len();
    let per_page = requested_per_page.min(100);
    let total_pages = if total_items == 0 { 0 } else { (total_items as u32).div_ceil(per_page) };
    let page = requested_page.max(1);
    let skip = ((page - 1) * per_page) as usize;
    let paginated = sorted.into_iter().skip(skip).take(per_page as usize).collect();

    (Pagination { page, per_page, total_items, total_pages }, paginated)
}
