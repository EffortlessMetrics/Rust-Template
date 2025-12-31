//! Unified issues endpoint aggregating friction, questions, and tasks.
//!
//! This module provides a single `/issues` endpoint that normalizes all three
//! artifact types into a common `Issue` representation with unified status,
//! priority, and filtering capabilities.

use crate::error::PlatformError;
use crate::friction::FrictionEntry;
use crate::questions::Question;
use crate::state::PlatformState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use gov_model::TaskStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Issue type discriminator
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    Friction,
    Question,
    Task,
}

impl std::fmt::Display for IssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueKind::Friction => write!(f, "friction"),
            IssueKind::Question => write!(f, "question"),
            IssueKind::Task => write!(f, "task"),
        }
    }
}

/// Normalized status across all issue types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    InProgress,
    Resolved,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Open => write!(f, "open"),
            IssueStatus::InProgress => write!(f, "in_progress"),
            IssueStatus::Resolved => write!(f, "resolved"),
        }
    }
}

/// Unified issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier (FRICTION-XXX, Q-XXX, TASK-XXX)
    pub id: String,
    /// Issue type
    pub kind: IssueKind,
    /// Normalized status
    pub status: IssueStatus,
    /// Original/native status string (for transparency)
    pub native_status: String,
    /// One-line summary
    pub summary: String,
    /// Priority (1=critical/p0, 2=high/p1, 3=medium/p2, 4=low/p3)
    pub priority: u8,
    /// Creation/discovery date (ISO 8601, nullable for tasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Category or flow context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Related REQ/AC IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    /// Owner/assignee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Labels/tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Query parameters for filtering issues
#[derive(Debug, Default, Deserialize)]
pub struct IssueFilters {
    /// Filter by issue kind (friction, question, task)
    pub kind: Option<IssueKind>,
    /// Filter by status (open, in_progress, resolved)
    pub status: Option<IssueStatus>,
    /// Filter by priority (1-4)
    pub priority: Option<u8>,
    /// Filter by minimum priority (inclusive, 1=highest)
    pub min_priority: Option<u8>,
    /// Filter by date range start (ISO 8601)
    pub from_date: Option<String>,
    /// Filter by date range end (ISO 8601)
    pub to_date: Option<String>,
    /// Text search in id, summary, category
    pub q: Option<String>,
    /// Page number (1-indexed, default 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (default 50, max 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Response for /platform/issues
#[derive(Debug, Serialize)]
pub struct IssuesResponse {
    pub issues: Vec<Issue>,
    pub pagination: Pagination,
    pub summary: IssuesSummary,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: usize,
    pub total_pages: u32,
}

/// Summary counts by kind and status
#[derive(Debug, Serialize)]
pub struct IssuesSummary {
    pub total: usize,
    pub by_kind: KindCounts,
    pub by_status: StatusCounts,
}

#[derive(Debug, Serialize)]
pub struct KindCounts {
    pub friction: usize,
    pub question: usize,
    pub task: usize,
}

#[derive(Debug, Serialize)]
pub struct StatusCounts {
    pub open: usize,
    pub in_progress: usize,
    pub resolved: usize,
}

// ============================================================================
// Conversion implementations
// ============================================================================

impl From<FrictionEntry> for Issue {
    fn from(f: FrictionEntry) -> Self {
        let status = match f.status.as_str() {
            "resolved" | "wont_fix" => IssueStatus::Resolved,
            "investigating" | "in_progress" => IssueStatus::InProgress,
            _ => IssueStatus::Open,
        };
        let priority = match f.severity.as_str() {
            "critical" => 1,
            "high" => 2,
            "medium" => 3,
            "low" => 4,
            _ => 3,
        };
        Issue {
            id: f.id,
            kind: IssueKind::Friction,
            status,
            native_status: f.status,
            summary: f.summary,
            priority,
            created_at: Some(f.date),
            category: Some(f.category),
            refs: f.refs,
            owner: f.context.as_ref().and_then(|c| c.discovered_by.clone()),
            labels: vec![],
        }
    }
}

impl From<Question> for Issue {
    fn from(q: Question) -> Self {
        let status = match q.status.as_str() {
            "resolved" | "obsolete" => IssueStatus::Resolved,
            "answered" => IssueStatus::InProgress,
            _ => IssueStatus::Open,
        };
        let mut refs = q.req_ids.clone();
        refs.extend(q.ac_ids.clone());
        refs.extend(q.refs.clone());
        Issue {
            id: q.id,
            kind: IssueKind::Question,
            status,
            native_status: q.status,
            summary: q.summary,
            priority: 3, // Default medium for questions
            created_at: Some(q.created_at),
            category: Some(q.context.flow),
            refs,
            owner: Some(q.created_by),
            labels: vec![],
        }
    }
}

/// Convert a task to an Issue
fn task_to_issue(task: &spec_runtime::Task, effective_status: &str) -> Issue {
    let status = match effective_status.to_lowercase().as_str() {
        "done" => IssueStatus::Resolved,
        "inprogress" | "in_progress" | "review" => IssueStatus::InProgress,
        _ => IssueStatus::Open,
    };

    let priority = extract_priority_from_labels(&task.labels);

    let mut refs = vec![task.requirement.clone()];
    refs.extend(task.acs.iter().cloned());

    Issue {
        id: task.id.clone(),
        kind: IssueKind::Task,
        status,
        native_status: effective_status.to_string(),
        summary: task.summary.clone(),
        priority,
        created_at: None, // Tasks don't have creation date in spec
        category: Some(task.requirement.clone()),
        refs,
        owner: task.owner.clone(),
        labels: task.labels.clone(),
    }
}

fn extract_priority_from_labels(labels: &[String]) -> u8 {
    for label in labels {
        match label.to_lowercase().as_str() {
            "p0" => return 1,
            "p1" => return 2,
            "p2" => return 3,
            "p3" => return 4,
            _ => {}
        }
    }
    3 // Default medium
}

// ============================================================================
// Filtering
// ============================================================================

fn apply_filters(issues: Vec<Issue>, filters: &IssueFilters) -> Vec<Issue> {
    issues
        .into_iter()
        .filter(|issue| {
            // Kind filter
            if let Some(kind) = &filters.kind
                && &issue.kind != kind
            {
                return false;
            }
            // Status filter
            if let Some(status) = &filters.status
                && &issue.status != status
            {
                return false;
            }
            // Priority filter (exact)
            if let Some(p) = filters.priority
                && issue.priority != p
            {
                return false;
            }
            // Min priority filter (1=highest, so <= comparison)
            if let Some(min_p) = filters.min_priority
                && issue.priority > min_p
            {
                return false;
            }
            // Date range filter (from)
            if let Some(from) = &filters.from_date
                && let Some(created) = &issue.created_at
                && created < from
            {
                return false;
            }
            // Date range filter (to)
            if let Some(to) = &filters.to_date
                && let Some(created) = &issue.created_at
                && created > to
            {
                return false;
            }
            // Text search
            if let Some(q) = &filters.q {
                let q_lower = q.to_lowercase();
                let matches = issue.id.to_lowercase().contains(&q_lower)
                    || issue.summary.to_lowercase().contains(&q_lower)
                    || issue.category.as_ref().is_some_and(|c| c.to_lowercase().contains(&q_lower))
                    || issue.labels.iter().any(|l| l.to_lowercase().contains(&q_lower));
                if !matches {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn calculate_summary(issues: &[Issue]) -> IssuesSummary {
    let mut by_kind = KindCounts { friction: 0, question: 0, task: 0 };
    let mut by_status = StatusCounts { open: 0, in_progress: 0, resolved: 0 };

    for issue in issues {
        match issue.kind {
            IssueKind::Friction => by_kind.friction += 1,
            IssueKind::Question => by_kind.question += 1,
            IssueKind::Task => by_kind.task += 1,
        }
        match issue.status {
            IssueStatus::Open => by_status.open += 1,
            IssueStatus::InProgress => by_status.in_progress += 1,
            IssueStatus::Resolved => by_status.resolved += 1,
        }
    }

    IssuesSummary { total: issues.len(), by_kind, by_status }
}

// ============================================================================
// Router and Handler
// ============================================================================

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

/// Load all friction entries (wrapper to call module-private function)
fn load_friction_entries(root: &std::path::Path) -> Result<Vec<FrictionEntry>, PlatformError> {
    let friction_dir = root.join("friction");

    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let dir_entries = std::fs::read_dir(&friction_dir).map_err(|e| {
        PlatformError::internal(format!("Failed to read friction directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_friction_entry(&path) {
            Ok(friction) => entries.push(friction),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load friction entry"
                );
            }
        }
    }

    Ok(entries)
}

fn load_friction_entry(path: &std::path::Path) -> Result<FrictionEntry, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read friction file {}: {}", path.display(), e))
    })?;

    let entry: FrictionEntry = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse friction YAML {}: {}", path.display(), e))
    })?;

    Ok(entry)
}

/// Load all question entries
fn load_question_entries(root: &std::path::Path) -> Result<Vec<Question>, PlatformError> {
    let questions_dir = root.join("questions");

    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    let dir_entries = std::fs::read_dir(&questions_dir).map_err(|e| {
        PlatformError::internal(format!("Failed to read questions directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_question_entry(&path) {
            Ok(question) => questions.push(question),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load question entry"
                );
            }
        }
    }

    Ok(questions)
}

fn load_question_entry(path: &std::path::Path) -> Result<Question, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read question file {}: {}", path.display(), e))
    })?;

    let question: Question = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse question YAML {}: {}", path.display(), e))
    })?;

    Ok(question)
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

    // Convert to unified issues
    let mut issues: Vec<Issue> = Vec::new();
    issues.extend(friction_entries.into_iter().map(Issue::from));
    issues.extend(questions.into_iter().map(Issue::from));

    // Convert tasks with effective status
    for task in &tasks_spec.tasks {
        let effective_status = status_map
            .get(&task.id)
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| task.status.clone());
        issues.push(task_to_issue(task, &effective_status));
    }

    // Apply filters
    let filtered = apply_filters(issues, &filters);

    // Calculate summary before pagination
    let summary = calculate_summary(&filtered);

    // Sort by priority (highest first), then by date (most recent first), then by ID (for determinism)
    let mut sorted = filtered;
    sorted.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });

    // Apply pagination
    let total_items = sorted.len();
    let per_page = filters.per_page.min(100);
    let total_pages = if total_items == 0 { 0 } else { (total_items as u32).div_ceil(per_page) };
    let page = filters.page.max(1);
    let skip = ((page - 1) * per_page) as usize;
    let paginated: Vec<Issue> = sorted.into_iter().skip(skip).take(per_page as usize).collect();

    Ok(Json(IssuesResponse {
        issues: paginated,
        pagination: Pagination { page, per_page, total_items, total_pages },
        summary,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friction_to_issue_conversion() {
        let friction = FrictionEntry {
            id: "FRICTION-TEST-001".to_string(),
            date: "2025-11-26".to_string(),
            category: "tooling".to_string(),
            severity: "high".to_string(),
            summary: "Test friction".to_string(),
            description: "Test description".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "open".to_string(),
            resolution: None,
            refs: vec!["REQ-001".to_string()],
            related_items: None,
        };

        let issue: Issue = friction.into();
        assert_eq!(issue.id, "FRICTION-TEST-001");
        assert_eq!(issue.kind, IssueKind::Friction);
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.priority, 2); // high = 2
        assert_eq!(issue.category, Some("tooling".to_string()));
    }

    #[test]
    fn test_friction_resolved_status() {
        let friction = FrictionEntry {
            id: "FRICTION-TEST-002".to_string(),
            date: "2025-11-26".to_string(),
            category: "tooling".to_string(),
            severity: "medium".to_string(),
            summary: "Resolved friction".to_string(),
            description: "Test".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "resolved".to_string(),
            resolution: None,
            refs: vec![],
            related_items: None,
        };

        let issue: Issue = friction.into();
        assert_eq!(issue.status, IssueStatus::Resolved);
    }

    #[test]
    fn test_priority_from_labels() {
        assert_eq!(extract_priority_from_labels(&["p0".to_string()]), 1);
        assert_eq!(extract_priority_from_labels(&["p1".to_string()]), 2);
        assert_eq!(extract_priority_from_labels(&["P2".to_string()]), 3);
        assert_eq!(extract_priority_from_labels(&["p3".to_string()]), 4);
        assert_eq!(extract_priority_from_labels(&["other".to_string()]), 3);
        assert_eq!(extract_priority_from_labels(&[]), 3);
    }

    #[test]
    fn test_filter_by_kind() {
        let issues = vec![
            Issue {
                id: "FRICTION-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let filters = IssueFilters { kind: Some(IssueKind::Friction), ..Default::default() };

        let filtered = apply_filters(issues, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "FRICTION-001");
    }

    #[test]
    fn test_filter_by_text_search() {
        let issues = vec![
            Issue {
                id: "FRICTION-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Build failure in CI".to_string(),
                priority: 3,
                created_at: None,
                category: Some("ci_cd".to_string()),
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "API design question".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let filters = IssueFilters { q: Some("CI".to_string()), ..Default::default() };

        let filtered = apply_filters(issues, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "FRICTION-001");
    }

    #[test]
    fn test_summary_calculation() {
        let issues = vec![
            Issue {
                id: "F-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::InProgress,
                native_status: "answered".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "T-001".to_string(),
                kind: IssueKind::Task,
                status: IssueStatus::Resolved,
                native_status: "Done".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let summary = calculate_summary(&issues);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_kind.friction, 1);
        assert_eq!(summary.by_kind.question, 1);
        assert_eq!(summary.by_kind.task, 1);
        assert_eq!(summary.by_status.open, 1);
        assert_eq!(summary.by_status.in_progress, 1);
        assert_eq!(summary.by_status.resolved, 1);
    }

    #[test]
    fn test_stable_ordering_with_id_tiebreaker() {
        // Create issues with same priority and date to test ID tiebreaker
        let mut issues = vec![
            Issue {
                id: "FRICTION-003".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Third".to_string(),
                priority: 2,
                created_at: Some("2025-01-01".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "FRICTION-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "First".to_string(),
                priority: 2,
                created_at: Some("2025-01-01".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "FRICTION-002".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Second".to_string(),
                priority: 2,
                created_at: Some("2025-01-01".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        // Sort using the same logic as the handler
        issues.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });

        // Verify deterministic order by ID
        assert_eq!(issues[0].id, "FRICTION-001");
        assert_eq!(issues[1].id, "FRICTION-002");
        assert_eq!(issues[2].id, "FRICTION-003");

        // Run sort again to verify stability
        issues.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });

        // Order should be identical
        assert_eq!(issues[0].id, "FRICTION-001");
        assert_eq!(issues[1].id, "FRICTION-002");
        assert_eq!(issues[2].id, "FRICTION-003");
    }

    #[test]
    fn test_ordering_priority_then_date_then_id() {
        let mut issues = vec![
            Issue {
                id: "A-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Low priority old".to_string(),
                priority: 3,
                created_at: Some("2025-01-01".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "B-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "High priority".to_string(),
                priority: 1,
                created_at: Some("2025-01-01".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "C-001".to_string(),
                kind: IssueKind::Task,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Low priority new".to_string(),
                priority: 3,
                created_at: Some("2025-01-15".to_string()),
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        issues.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });

        // High priority first
        assert_eq!(issues[0].id, "B-001");
        // Then by date (newest first), C-001 is 2025-01-15, A-001 is 2025-01-01
        assert_eq!(issues[1].id, "C-001");
        assert_eq!(issues[2].id, "A-001");
    }
}
