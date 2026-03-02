use crate::{AppError, AppState, get_error_summary};
use axum::{Json, Router, extract::State, routing::get};
use doc_type_contract::{DocTypeInput, validate};
use serde::{Deserialize, Serialize};
use spec_runtime::{ValidatedConfig, load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;
use tracing::instrument;

mod idp;
mod ui;

// Re-export gov-http types for backwards compatibility with downstream consumers
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

/// Platform API routes (mounted at /platform)
///
/// Uses gov-http's `platform_routes_no_status` for governance-generic endpoints,
/// then adds service-specific endpoints like rich `/status` and debug info.
pub fn router(state: AppState) -> Router<AppState> {
    // Start with gov-http routes (includes friction, questions, forks)
    gov_http::platform_routes_no_status::<AppState>()
        // Service-specific endpoints
        .route("/debug/info", get(debug_info))
        .route("/status", get(get_status))
        // Service-specific sub-routers
        .merge(idp::router())
        .with_state(state)
}

/// UI routes (mounted at root)
pub fn ui_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(ui::dashboard))
        .route("/ui", get(ui::dashboard))
        .route("/ui/graph", get(ui::graph_view))
        .route("/ui/flows", get(ui::flows_view))
        .route("/ui/coverage", get(ui::coverage_view))
        .with_state(state)
}

#[derive(Serialize)]
struct PlatformStatus {
    service: ServiceInfo,
    governance: GovernanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<ConfigSummary>,
    errors: crate::errors::ErrorSummary,
}

#[derive(Serialize)]
struct ServiceInfo {
    service_id: String,
    template_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    links: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<String>,
}

#[derive(Serialize)]
struct GovernanceStatus {
    ledger: LedgerCounts,
    devex: DevExCounts,
    docs: DocCounts,
    tasks: TaskCounts,
    questions: QuestionCounts,
    friction: FrictionCounts,
    forks: ForkCounts,
    policies: PolicyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_coverage: Option<AcCoverageInfo>,
}

#[derive(Serialize)]
struct LedgerCounts {
    stories: usize,
    requirements: usize,
    acs: usize,
}

#[derive(Serialize)]
struct DevExCounts {
    commands: usize,
    flows: usize,
}

#[derive(Serialize)]
struct DocCounts {
    total: usize,
    design: usize,
    doc_type_issues: usize,
}

#[derive(Serialize)]
struct TaskCounts {
    total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_status: Option<TaskStatusBreakdown>,
}

#[derive(Serialize)]
struct TaskStatusBreakdown {
    todo: usize,
    in_progress: usize,
    review: usize,
    done: usize,
}

#[derive(Serialize)]
struct AcCoverageInfo {
    total: usize,
    passing: usize,
    failing: usize,
    unknown: usize,
}

#[derive(Serialize)]
struct QuestionCounts {
    open: usize,
    answered: usize,
    resolved: usize,
    total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    top_open: Vec<StatusQuestionBrief>,
}

/// Question summary for status endpoint (different from gov_http::QuestionSummary)
#[derive(Serialize)]
struct StatusQuestionBrief {
    id: String,
    summary: String,
    flow: String,
}

#[derive(Serialize)]
struct FrictionCounts {
    total: usize,
    open: usize,
    by_severity: SeverityCounts,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    recent: Vec<FrictionSummary>,
}

#[derive(Serialize)]
struct SeverityCounts {
    low: usize,
    medium: usize,
    high: usize,
    critical: usize,
}

#[derive(Serialize)]
struct FrictionSummary {
    id: String,
    date: String,
    severity: String,
    summary: String,
    category: String,
}

#[derive(Serialize)]
struct ForkCounts {
    total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ids: Vec<String>,
}

#[derive(Serialize)]
struct PolicyStatus {
    status: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct ConfigSummary {
    env: Option<String>,
    http_port: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    settings: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    secrets_redacted: HashMap<String, String>,
    auth: AuthSummary,
}

#[derive(Serialize, Clone)]
struct AuthSummary {
    mode: String,
    token_present: bool,
}

#[derive(Deserialize)]
struct PolicyStatusReport {
    summary: String,
}

pub(crate) fn config_summary(state: &AppState) -> Option<ConfigSummary> {
    let config = state.config.as_ref()?;
    Some(ConfigSummary::from_parts(config, &state.platform_auth))
}

impl ConfigSummary {
    fn from_parts(config: &ValidatedConfig, auth: &crate::security::PlatformAuthConfig) -> Self {
        let settings = settings_as_json(&config.settings);

        ConfigSummary {
            env: config.env.clone(),
            http_port: config.http_port,
            settings,
            secrets_redacted: redacted_secrets(&config.secrets),
            auth: AuthSummary {
                mode: auth.mode_label().to_string(),
                token_present: auth.token_present(),
            },
        }
    }
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

// ============================================================================
// Debug/Info endpoint - matches docs/how-to/add-http-endpoint.md example
// ============================================================================

/// Debug info response DTO
#[derive(Debug, Serialize)]
struct DebugInfo {
    kernel_version: String,
    template_version: String,
}

// Baseline measurement: incremental build test

/// Platform debug info endpoint
///
/// Returns basic kernel and template version information.
/// Documented in docs/how-to/add-http-endpoint.md as a canonical example.
async fn debug_info(State(state): State<AppState>) -> Json<DebugInfo> {
    let root = &state.workspace_root;

    let template_version = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .ok()
        .and_then(|m| m.template_version)
        .unwrap_or_else(|| "unknown".to_string());

    Json(DebugInfo { kernel_version: env!("CARGO_PKG_VERSION").to_string(), template_version })
}

/// Validate doc_type contract for a single document.
///
/// Used by status endpoint to count doc_type issues.
/// Returns (is_valid, issue_description).
fn validate_doc_type_contract(doc: &spec_runtime::DocEntry) -> (bool, Option<String>) {
    let result = validate(DocTypeInput {
        doc_type: &doc.doc_type,
        stories: &doc.stories,
        requirements: &doc.requirements,
        acs: &doc.acs,
    });
    (result.valid, result.issue)
}

#[instrument(skip(state))]
async fn get_status(State(state): State<AppState>) -> Result<Json<PlatformStatus>, AppError> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).map_err(|e| AppError::spec_load_error("load specs", e))?;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .map_err(|e| AppError::spec_load_error("load tasks", e))?;

    let ledger_counts = LedgerCounts {
        stories: specs.ledger.stories.len(),
        requirements: specs.ledger.stories.iter().map(|s| s.requirements.len()).sum(),
        acs: specs
            .ledger
            .stories
            .iter()
            .flat_map(|s| s.requirements.iter())
            .map(|r| r.acceptance_criteria.len())
            .sum(),
    };

    let devex_counts =
        DevExCounts { commands: specs.devex.commands.len(), flows: specs.devex.flows.len() };

    let doc_type_issues =
        specs.docs.docs.iter().filter(|d| !validate_doc_type_contract(d).0).count();
    let doc_counts = DocCounts {
        total: specs.docs.docs.len(),
        design: specs.docs.docs.iter().filter(|d| d.doc_type == "design_doc").count(),
        doc_type_issues,
    };

    // Calculate task status breakdown
    let task_breakdown = calculate_task_breakdown(&tasks_spec);
    let task_counts = TaskCounts { total: tasks_spec.tasks.len(), by_status: Some(task_breakdown) };

    // Load AC coverage from idp module
    let ac_cov = idp::load_ac_coverage(root);
    let ac_coverage = Some(AcCoverageInfo {
        total: ac_cov.total,
        passing: ac_cov.passing,
        failing: ac_cov.failing,
        unknown: ac_cov.unknown,
    });

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

    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .map_err(|e| AppError::spec_load_error("load service_metadata.yaml", e))?;

    let template_version =
        metadata.template_version.clone().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let service_info = ServiceInfo {
        service_id: metadata.service_id.clone(),
        template_version,
        display_name: metadata.display_name.clone(),
        description: metadata.description.clone(),
        links: metadata.links.clone(),
        tags: metadata.tags.clone(),
    };

    let config = config_summary(&state);

    Ok(Json(PlatformStatus {
        service: service_info,
        governance: GovernanceStatus {
            ledger: ledger_counts,
            devex: devex_counts,
            docs: doc_counts,
            tasks: task_counts,
            questions: question_counts,
            friction: friction_counts,
            forks: fork_counts,
            policies: PolicyStatus { status: policy_status },
            ac_coverage,
        },
        config,
        errors: get_error_summary(),
    }))
}

/// Calculate task counts broken down by status
fn calculate_task_breakdown(tasks_spec: &spec_runtime::TasksSpec) -> TaskStatusBreakdown {
    let mut breakdown = TaskStatusBreakdown { todo: 0, in_progress: 0, review: 0, done: 0 };
    for task in &tasks_spec.tasks {
        let status = &task.status;
        match status.to_lowercase().as_str() {
            "open" | "todo" => breakdown.todo += 1,
            "inprogress" | "in_progress" | "in-progress" => breakdown.in_progress += 1,
            "review" => breakdown.review += 1,
            "done" | "closed" => breakdown.done += 1,
            _ => breakdown.todo += 1, // Default unknown to todo
        }
    }
    breakdown
}

/// Load question counts from questions/ directory
fn load_question_counts(root: &std::path::Path) -> QuestionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Question {
        id: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        status: String,
        context: QuestionContext,
    }

    #[derive(Deserialize)]
    struct QuestionContext {
        flow: String,
    }

    let questions_dir = root.join("questions");
    if !questions_dir.exists() {
        return QuestionCounts { open: 0, answered: 0, resolved: 0, total: 0, top_open: vec![] };
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
                        open_questions.push(StatusQuestionBrief {
                            id: question.id,
                            summary: question.summary,
                            flow: question.context.flow,
                        });
                    }
                    "answered" => answered += 1,
                    "resolved" => resolved += 1,
                    _ => {}
                }
            }
        }
    }

    // Take top 3 open questions
    open_questions.truncate(3);

    QuestionCounts { open, answered, resolved, total, top_open: open_questions }
}

/// Load friction counts from friction/ directory
fn load_friction_counts(root: &std::path::Path) -> FrictionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct FrictionEntry {
        id: String,
        date: String,
        #[serde(default)]
        severity: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        category: String,
        #[serde(default)]
        status: String,
    }

    let friction_dir = root.join("friction");
    if !friction_dir.exists() {
        return FrictionCounts {
            total: 0,
            open: 0,
            by_severity: SeverityCounts { low: 0, medium: 0, high: 0, critical: 0 },
            recent: vec![],
        };
    }

    let mut total = 0;
    let mut open = 0;
    let mut by_severity = SeverityCounts { low: 0, medium: 0, high: 0, critical: 0 };
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

                // Count open friction
                if friction.status == "open" || friction.status.is_empty() {
                    open += 1;
                }

                // Count by severity
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

    // Sort by date (most recent first) and take top 5
    all_entries.sort_by(|a, b| b.date.cmp(&a.date));
    let recent: Vec<FrictionSummary> = all_entries
        .into_iter()
        .take(5)
        .map(|e| FrictionSummary {
            id: e.id,
            date: e.date,
            severity: e.severity,
            summary: e.summary,
            category: e.category,
        })
        .collect();

    FrictionCounts { total, open, by_severity, recent }
}

/// Load fork counts from forks/fork_registry.yaml
fn load_fork_counts(root: &std::path::Path) -> ForkCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ForkRegistry {
        #[serde(default)]
        forks: Vec<ForkEntry>,
    }

    #[derive(Deserialize)]
    struct ForkEntry {
        id: String,
    }

    let registry_path = root.join("forks/fork_registry.yaml");
    if !registry_path.exists() {
        return ForkCounts { total: 0, ids: vec![] };
    }

    if let Ok(content) = fs::read_to_string(&registry_path)
        && let Ok(registry) = serde_yaml::from_str::<ForkRegistry>(&content)
    {
        let ids: Vec<String> = registry.forks.iter().map(|f| f.id.clone()).collect();
        let total = ids.len();
        ForkCounts { total, ids }
    } else {
        ForkCounts { total: 0, ids: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{PlatformAuthConfig, PlatformAuthMode};

    #[test]
    fn log_hygiene_redacts_secrets() {
        let mut settings = HashMap::new();
        settings
            .insert("platform.auth_mode".to_string(), serde_yaml::Value::String("basic".into()));

        let mut secrets = HashMap::new();
        secrets.insert("db.url".to_string(), "postgres://user:pass@localhost:5432/app".to_string());
        secrets.insert("platform.auth_token".to_string(), "super-secret-token".to_string());

        let config =
            ValidatedConfig { http_port: 8080, env: Some("dev".to_string()), settings, secrets };

        let auth = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("super-secret-token".into()),
            jwt_secret: None,
        };
        let summary = ConfigSummary::from_parts(&config, &auth);

        let serialized = serde_json::to_string(&summary).expect("summary should serialize");

        assert!(
            !serialized.contains("super-secret-token"),
            "Serialized summary should not leak auth tokens"
        );
        assert_eq!(summary.secrets_redacted.get("db.url"), Some(&"[REDACTED]".to_string()));
        assert_eq!(summary.auth.mode, "basic");
        assert!(summary.auth.token_present);
    }

    // Note: normalize_status tests moved to gov-http::handlers tests
}
