//! HTTP handlers for `/platform/*` endpoints.
//!
//! This crate implements the platform API including:
//! - Platform status endpoint
//! - IDP snapshot endpoint
//! - UI routes (dashboard, graph, flows, coverage)
//! - Debug info endpoint
//!
//! # Design Philosophy
//!
//! - **Platform-focused**: Only platform-related handlers
//! - **Contract-based**: Uses `platform-contract` types for stable API
//! - **Gov-http integration**: Reuses gov-http handlers for governance endpoints
//!
//! # Example
//!
//! ```rust,ignore
//! use http_platform::router;
//!
//! let app = Router::new().nest("/platform", router(state));
//! ```

use axum::{Json, Router, extract::State, routing::get};
use http_errors::HttpError;
use http_idp_snapshot::load_ac_coverage;
use platform_contract::{
    AcCoverageInfo, AuthSummary, ConfigSummary, DevExCounts, DocCounts, ErrorStats, ErrorSummary,
    ForkCounts, FrictionCounts, FrictionSummary, GovernanceStatus, LedgerCounts, PolicyStatus,
    QuestionBrief, QuestionCounts, ServiceInfo, SeverityCounts, TaskCounts, TaskStatusBreakdown,
};
use serde::{Deserialize, Serialize};
use spec_runtime::{ValidatedConfig, load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;
use tracing::instrument;

mod idp;
mod ui;

// Re-export gov-http types for backwards compatibility
pub use gov_http::{
    // Coverage types
    CoverageDetail,
    CoverageResponse,
    CoverageSummary,
    // Docs types
    DocHealthSummary,
    DocInfoWithHealth,
    DocsIndexResponse,
    // Forks types
    ForkEntry,
    ForkSummary,
    ForksListResponse,
    // Friction types
    FrictionContext,
    FrictionEntry,
    FrictionListResponse,
    // Question types
    Question,
    QuestionContext,
    QuestionFilters,
    QuestionSummary,
    QuestionsListResponse,
    // Query types
    SuggestNextQuery,
    // Task types
    TaskDocsOut,
    TaskFilters,
    TaskGraphQuery,
    TaskGraphResponse,
    TaskOut,
    TasksResponse,
};

// Re-export IDP snapshot types
pub use http_idp_snapshot::IdpSnapshot;

// ============================================================================
// State Trait
// ============================================================================

/// Platform state trait for handlers.
///
/// This trait defines the minimal interface required for platform handlers.
pub trait PlatformState: Clone + Send + Sync + 'static {
    /// Get workspace root path.
    fn workspace_root(&self) -> &std::path::Path;

    /// Get validated config (if available).
    fn config(&self) -> Option<&ValidatedConfig>;

    /// Get platform auth config.
    fn platform_auth(&self) -> &dyn PlatformAuthConfig;
}

/// Platform auth config trait.
pub trait PlatformAuthConfig {
    /// Get auth mode label.
    fn mode_label(&self) -> &str;

    /// Check if token is present.
    fn token_present(&self) -> bool;
}

impl PlatformAuthConfig for http_auth::PlatformAuthConfig {
    fn mode_label(&self) -> &str {
        http_auth::PlatformAuthConfig::mode_label(self)
    }

    fn token_present(&self) -> bool {
        http_auth::PlatformAuthConfig::token_present(self)
    }
}

// ============================================================================
// Platform Router
// ============================================================================

/// Create the platform API router (mounted at /platform).
///
/// Uses gov-http's `platform_routes_no_status` for governance-generic endpoints,
/// then adds service-specific endpoints like rich `/status` and debug info.
pub fn router<S>(state: S) -> Router<S>
where
    S: PlatformState + Clone + 'static + gov_http::PlatformState,
{
    gov_http::platform_routes_no_status::<S>()
        .merge(idp::router::<S>())
        // Service-specific endpoints
        .route("/debug/info", get(debug_info::<S>))
        .route("/status", get(get_status::<S>))
        .with_state(state)
}

/// Create the UI routes (mounted at root).
///
/// Returns routes for:
/// - `/` - Dashboard
/// - `/ui` - Dashboard
/// - `/ui/graph` - Graph view
/// - `/ui/flows` - Flows view
/// - `/ui/coverage` - Coverage view
pub fn ui_router<S>(state: S) -> Router<S>
where
    S: PlatformState + Clone + 'static,
{
    Router::new()
        .route("/", get(ui::dashboard::<S>))
        .route("/ui", get(ui::dashboard::<S>))
        .route("/ui/graph", get(ui::graph_view::<S>))
        .route("/ui/flows", get(ui::flows_view::<S>))
        .route("/ui/coverage", get(ui::coverage_view::<S>))
        .with_state(state)
}

// ============================================================================
// Debug Info Handler
// ============================================================================

/// Debug info response DTO.
#[derive(Debug, Clone, Serialize)]
pub struct DebugInfo {
    /// Kernel version
    pub kernel_version: String,
    /// Template version
    pub template_version: String,
}

/// Platform debug info endpoint.
///
/// Returns basic kernel and template version information.
#[instrument(skip(state))]
async fn debug_info<S>(State(state): State<S>) -> Json<DebugInfo>
where
    S: PlatformState,
{
    let root = state.workspace_root();

    let template_version = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .ok()
        .and_then(|m| m.template_version)
        .unwrap_or_else(|| "unknown".to_string());

    Json(DebugInfo { kernel_version: env!("CARGO_PKG_VERSION").to_string(), template_version })
}

// ============================================================================
// Platform Status Handler
// ============================================================================

/// Platform status response.
#[derive(Debug, Clone, Serialize)]
struct PlatformStatusResponse {
    /// Service information
    pub service: ServiceInfo,
    /// Governance status
    pub governance: GovernanceStatus,
    /// Optional config summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConfigSummary>,
    /// Error summary
    pub errors: ErrorSummary,
}

/// Get platform status endpoint.
#[instrument(skip(state))]
async fn get_status<S>(State(state): State<S>) -> Result<Json<PlatformStatusResponse>, HttpError>
where
    S: PlatformState,
{
    let root = state.workspace_root();
    let specs = load_all_specs(root)
        .map_err(|e| HttpError::internal_error(format!("Failed to load specs: {}", e)))?;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .map_err(|e| HttpError::internal_error(format!("Failed to load tasks: {}", e)))?;

    let ledger_counts = LedgerCounts::new(
        specs.ledger.stories.len(),
        specs.ledger.stories.iter().map(|s| s.requirements.len()).sum(),
        specs
            .ledger
            .stories
            .iter()
            .flat_map(|s| s.requirements.iter())
            .map(|r| r.acceptance_criteria.len())
            .sum(),
    );

    let devex_counts = DevExCounts::new(specs.devex.commands.len(), specs.devex.flows.len());

    let doc_type_issues =
        specs.docs.docs.iter().filter(|d| !validate_doc_type_contract(d).0).count();
    let doc_counts = DocCounts::new(
        specs.docs.docs.len(),
        specs.docs.docs.iter().filter(|d| d.doc_type == "design_doc").count(),
        doc_type_issues,
    );

    // Calculate task status breakdown
    let task_breakdown = calculate_task_breakdown(&tasks_spec);
    let task_counts = TaskCounts::new(tasks_spec.tasks.len(), Some(task_breakdown));

    // Load AC coverage from idp module
    let ac_cov = load_ac_coverage(root);
    let ac_coverage =
        Some(AcCoverageInfo::new(ac_cov.total, ac_cov.passing, ac_cov.failing, ac_cov.unknown));

    // Load question counts
    let question_counts = load_question_counts(root);

    // Load friction counts
    let friction_counts = load_friction_counts(root);

    // Load fork counts
    let fork_counts = load_fork_counts(root);

    // Read policy status from last policy-test run
    let policy_path = root.join("target/policy_status.json");
    let policy_status = if let Ok(content) = fs::read_to_string(policy_path) {
        serde_json::from_str::<PolicyStatusReport>(&content)
            .map(|r| r.summary)
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    let metadata =
        load_service_metadata(&root.join("specs/service_metadata.yaml")).map_err(|e| {
            HttpError::internal_error(format!("Failed to load service_metadata.yaml: {}", e))
        })?;

    let template_version =
        metadata.template_version.clone().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let service_info = ServiceInfo::new(
        metadata.service_id.clone(),
        template_version,
        metadata.display_name.clone(),
        metadata.description.clone(),
        metadata.links.clone(),
        metadata.tags.clone(),
    );

    let config = config_summary(&state);

    Ok(Json(PlatformStatusResponse {
        service: service_info,
        governance: GovernanceStatus::new(
            ledger_counts,
            devex_counts,
            doc_counts,
            task_counts,
            question_counts,
            friction_counts,
            fork_counts,
            PolicyStatus::new(policy_status),
            ac_coverage,
        ),
        config,
        errors: ErrorSummary::new(false, None, ErrorStats::new(0, 0, 0)),
    }))
}

// ============================================================================
// Internal Types and Helpers
// ============================================================================

/// Get config summary from state.
fn config_summary<S>(state: &S) -> Option<ConfigSummary>
where
    S: PlatformState,
{
    let config = state.config()?;
    let auth = state.platform_auth();
    Some(ConfigSummary::new(
        config.env.clone(),
        config.http_port,
        settings_as_json(&config.settings),
        redacted_secrets(&config.secrets),
        AuthSummary::new(auth.mode_label().to_string(), auth.token_present()),
    ))
}

fn settings_as_json(
    source: &HashMap<String, serde_yaml::Value>,
) -> HashMap<String, serde_json::Value> {
    let mut out = HashMap::new();

    for (k, v) in source {
        if let Ok(json_val) = serde_json::to_value(v) {
            out.insert(k.clone(), json_val);
        }
    }

    out
}

fn redacted_secrets(secrets: &HashMap<String, String>) -> HashMap<String, String> {
    secrets.keys().map(|k| (k.clone(), "[REDACTED]".to_string())).collect()
}

/// Validate doc_type contract for a single document.
fn validate_doc_type_contract(doc: &spec_runtime::DocEntry) -> (bool, Option<String>) {
    let doc_type = doc.doc_type.replace('-', "_");

    match doc_type.as_str() {
        "how_to" => {
            if doc.requirements.is_empty() && doc.acs.is_empty() {
                return (
                    false,
                    Some("how_to should reference at least one requirement or AC".into()),
                );
            }
        }
        "explanation" => {
            if doc.stories.is_empty() && doc.requirements.is_empty() {
                return (
                    false,
                    Some("explanation should reference at least one story or requirement".into()),
                );
            }
        }
        "design_doc" => {
            if doc.requirements.is_empty() {
                return (
                    false,
                    Some("design_doc should reference at least one requirement".into()),
                );
            }
        }
        "reference" => {
            if doc.requirements.is_empty() && doc.acs.is_empty() {
                return (
                    false,
                    Some("reference should reference at least one requirement or AC".into()),
                );
            }
        }
        "status" => {
            if doc.requirements.is_empty() || doc.acs.is_empty() {
                return (false, Some("status should reference both requirements and ACs".into()));
            }
        }
        "adr" => {
            if doc.requirements.is_empty() {
                return (false, Some("adr should reference at least one requirement".into()));
            }
        }
        "guide" => {
            if doc.requirements.is_empty() && doc.acs.is_empty() {
                return (
                    false,
                    Some("guide should reference at least one requirement or AC".into()),
                );
            }
        }
        "impl_plan" => {
            if doc.requirements.is_empty() || doc.acs.is_empty() {
                return (
                    false,
                    Some("impl_plan should reference both requirements and ACs".into()),
                );
            }
        }
        "requirements_doc" => {
            if doc.requirements.is_empty() {
                return (
                    false,
                    Some("requirements_doc should reference at least one requirement".into()),
                );
            }
        }
        "ci_workflow" => {}
        _ => {
            return (false, Some(format!("Unknown doc_type '{}'", doc.doc_type)));
        }
    }
    (true, None)
}

/// Calculate task counts broken down by status.
fn calculate_task_breakdown(tasks_spec: &spec_runtime::TasksSpec) -> TaskStatusBreakdown {
    let mut breakdown = TaskStatusBreakdown::new(0, 0, 0, 0);
    for task in &tasks_spec.tasks {
        let status = &task.status;
        match status.to_lowercase().as_str() {
            "open" | "todo" => breakdown.todo += 1,
            "inprogress" | "in_progress" | "in-progress" => breakdown.in_progress += 1,
            "review" => breakdown.review += 1,
            "done" | "closed" => breakdown.done += 1,
            _ => breakdown.todo += 1,
        }
    }
    breakdown
}

/// Load question counts from questions/ directory.
fn load_question_counts(root: &std::path::Path) -> QuestionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Question {
        pub id: String,
        #[serde(default)]
        pub summary: String,
        #[serde(default)]
        pub status: String,
        pub context: QuestionContext,
    }

    #[derive(Deserialize)]
    struct QuestionContext {
        pub flow: String,
    }

    let questions_dir = root.join("questions");
    if !questions_dir.exists() {
        return QuestionCounts::new(0, 0, 0, 0, vec![]);
    }

    let mut open = 0;
    let mut answered = 0;
    let mut resolved = 0;
    let mut total = 0;
    let mut open_questions = Vec::new();

    if let Ok(entries) = fs::read_dir(&questions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(question) = serde_yaml::from_str::<Question>(&content)
            {
                total += 1;
                match question.status.as_str() {
                    "open" => {
                        open += 1;
                        open_questions.push(QuestionBrief::new(
                            question.id,
                            question.summary,
                            question.context.flow,
                        ));
                    }
                    "answered" => answered += 1,
                    "resolved" => resolved += 1,
                    _ => {}
                }
            }
        }
    }

    open_questions.truncate(3);

    QuestionCounts::new(open, answered, resolved, total, open_questions)
}

/// Load friction counts from friction/ directory.
fn load_friction_counts(root: &std::path::Path) -> FrictionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct FrictionEntry {
        pub id: String,
        pub date: String,
        #[serde(default)]
        pub severity: String,
        #[serde(default)]
        pub summary: String,
        #[serde(default)]
        pub category: String,
        #[serde(default)]
        pub status: String,
    }

    let friction_dir = root.join("friction");
    if !friction_dir.exists() {
        return FrictionCounts::new(0, 0, SeverityCounts::new(0, 0, 0, 0), vec![]);
    }

    let mut total = 0;
    let mut open = 0;
    let mut by_severity = SeverityCounts::new(0, 0, 0, 0);
    let mut all_entries = Vec::new();

    if let Ok(entries) = fs::read_dir(&friction_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(friction) = serde_yaml::from_str::<FrictionEntry>(&content)
            {
                total += 1;

                if friction.status == "open" || friction.status.is_empty() {
                    open += 1;
                }

                match friction.severity.as_str() {
                    "low" => by_severity.low += 1,
                    "medium" => by_severity.medium += 1,
                    "high" => by_severity.high += 1,
                    "critical" => by_severity.critical += 1,
                    _ => {}
                }

                all_entries.push(friction);
            }
        }
    }

    all_entries.sort_by(|a, b| b.date.cmp(&a.date));
    let recent: Vec<FrictionSummary> = all_entries
        .into_iter()
        .take(5)
        .map(|e| FrictionSummary::new(e.id, e.date, e.severity, e.summary, e.category))
        .collect();

    FrictionCounts::new(total, open, by_severity, recent)
}

/// Load fork counts from forks/fork_registry.yaml.
fn load_fork_counts(root: &std::path::Path) -> ForkCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ForkRegistry {
        #[serde(default)]
        pub forks: Vec<ForkEntry>,
    }

    #[derive(Deserialize)]
    struct ForkEntry {
        pub id: String,
    }

    let registry_path = root.join("forks/fork_registry.yaml");
    if !registry_path.exists() {
        return ForkCounts::new(0, vec![]);
    }

    if let Ok(content) = fs::read_to_string(&registry_path)
        && let Ok(registry) = serde_yaml::from_str::<ForkRegistry>(&content)
    {
        let ids: Vec<String> = registry.forks.iter().map(|f| f.id.clone()).collect();
        let total = ids.len();
        ForkCounts::new(total, ids)
    } else {
        ForkCounts::new(0, vec![])
    }
}

#[derive(Debug, Deserialize)]
struct PolicyStatusReport {
    pub summary: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_task_breakdown() {
        let tasks_spec = spec_runtime::TasksSpec {
            schema_version: "1.0".to_string(),
            template_version: "0.0.0".to_string(),
            tasks: vec![
                spec_runtime::Task {
                    id: "task-1".to_string(),
                    title: "Task 1".to_string(),
                    status: "todo".to_string(),
                    requirement: "REQ-001".to_string(),
                    acs: vec![],
                    labels: vec![],
                    owner: None,
                    docs: None,
                    summary: String::new(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
                spec_runtime::Task {
                    id: "task-2".to_string(),
                    title: "Task 2".to_string(),
                    status: "in_progress".to_string(),
                    requirement: "REQ-002".to_string(),
                    acs: vec![],
                    labels: vec![],
                    owner: None,
                    docs: None,
                    summary: String::new(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
            ],
        };

        let breakdown = calculate_task_breakdown(&tasks_spec);
        assert_eq!(breakdown.todo, 1);
        assert_eq!(breakdown.in_progress, 1);
        assert_eq!(breakdown.review, 0);
        assert_eq!(breakdown.done, 0);
    }

    #[test]
    fn test_validate_doc_type_how_to() {
        let doc = spec_runtime::DocEntry {
            id: "DOC-001".to_string(),
            file: "docs/how_to.md".to_string(),
            doc_type: "how_to".to_string(),
            stories: vec![],
            requirements: vec!["REQ-001".to_string()],
            acs: vec![],
            adrs: vec![],
        };
        assert!(validate_doc_type_contract(&doc).0);

        let doc_no_refs = spec_runtime::DocEntry {
            id: "DOC-002".to_string(),
            file: "docs/how_to.md".to_string(),
            doc_type: "how_to".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
            adrs: vec![],
        };
        assert!(!validate_doc_type_contract(&doc_no_refs).0);
    }
}
