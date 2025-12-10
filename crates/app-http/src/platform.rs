use crate::{AppError, AppState, get_error_summary};
use adapters_spec_fs::tasks_state;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use business_core::governance::{TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use spec_runtime::{ValidatedConfig, load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;

mod forks;
mod friction;
mod idp;
mod questions;
mod ui;

/// Platform API routes (mounted at /platform)
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // API routes
        .route("/debug/info", get(debug_info))
        .route("/graph", get(get_graph))
        .route("/schema", get(get_schema))
        .route("/schema/{name}", get(get_schema_by_name_handler))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
        .route("/coverage", get(get_coverage))
        .route("/tasks", get(get_tasks))
        .route("/tasks/suggest-next", get(get_suggest_next))
        .route("/tasks/graph", get(get_task_graph))
        .route("/ui/contract", get(get_ui_contract))
        .merge(friction::router())
        .merge(questions::router())
        .merge(forks::router())
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

#[derive(Deserialize)]
struct SuggestNextQuery {
    task: String,
}

async fn get_suggest_next(
    State(state): State<AppState>,
    Query(q): Query<SuggestNextQuery>,
) -> Result<Json<spec_runtime::tasks::SuggestedSequence>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .map_err(|e| AppError::spec_load_error("load tasks.yaml", e))?;
    let devex_spec = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .map_err(|e| AppError::spec_load_error("load devex_flows.yaml", e))?;
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))
        .map_err(|e| AppError::spec_load_error("load spec_ledger.yaml", e))?;

    let suggestion =
        spec_runtime::tasks::suggest_next(root, &q.task, &tasks_spec, &devex_spec, &ledger)
            .map_err(|e| {
                AppError::internal_error(format!("Failed to generate suggestion: {}", e))
            })?;

    Ok(Json(suggestion))
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
    top_open: Vec<QuestionSummary>,
}

#[derive(Serialize)]
struct QuestionSummary {
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

async fn get_graph(State(state): State<AppState>) -> Result<Json<spec_runtime::Graph>, AppError> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).map_err(|e| AppError::spec_load_error("load specs", e))?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)
        .map_err(|e| AppError::internal_error(format!("Failed to build graph: {}", e)))?;
    Ok(Json(graph))
}

async fn get_devex_flows(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = &state.workspace_root;
    let devex = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .map_err(|e| AppError::spec_load_error("load devex flows", e))?;
    let value = serde_json::to_value(devex)
        .map_err(|e| AppError::internal_error(format!("Failed to serialize devex flows: {}", e)))?;
    Ok(Json(value))
}

/// Response for /platform/docs/index with health info
#[derive(Serialize)]
struct DocsIndexResponse {
    schema_version: String,
    template_version: String,
    docs: Vec<DocInfoWithHealth>,
    summary: DocHealthSummary,
}

#[derive(Serialize)]
struct DocInfoWithHealth {
    id: String,
    file: String,
    doc_type: String,
    #[serde(default)]
    stories: Vec<String>,
    #[serde(default)]
    requirements: Vec<String>,
    #[serde(default)]
    acs: Vec<String>,
    #[serde(default)]
    adrs: Vec<String>,
    /// Doc type contract validation result
    doc_type_valid: bool,
    /// Issue description if doc_type_valid is false
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_type_issue: Option<String>,
}

#[derive(Serialize)]
struct DocHealthSummary {
    total: usize,
    valid: usize,
    with_issues: usize,
}

async fn get_docs_index(
    State(state): State<AppState>,
) -> Result<Json<DocsIndexResponse>, AppError> {
    let root = &state.workspace_root;
    let docs = spec_runtime::load_doc_index(&root.join("specs/doc_index.yaml"))
        .map_err(|e| AppError::spec_load_error("load doc index", e))?;

    let mut docs_with_health = Vec::new();
    let mut valid_count = 0;
    let mut issue_count = 0;

    for doc in docs.docs {
        let (doc_type_valid, doc_type_issue) = validate_doc_type_contract(&doc);
        if doc_type_valid {
            valid_count += 1;
        } else {
            issue_count += 1;
        }

        docs_with_health.push(DocInfoWithHealth {
            id: doc.id,
            file: doc.file,
            doc_type: doc.doc_type,
            stories: doc.stories,
            requirements: doc.requirements,
            acs: doc.acs,
            adrs: doc.adrs,
            doc_type_valid,
            doc_type_issue,
        });
    }

    Ok(Json(DocsIndexResponse {
        schema_version: docs.schema_version,
        template_version: docs.template_version,
        docs: docs_with_health,
        summary: DocHealthSummary {
            total: valid_count + issue_count,
            valid: valid_count,
            with_issues: issue_count,
        },
    }))
}

/// Validate doc_type contract for a single document
/// Returns (is_valid, issue_description)
fn validate_doc_type_contract(doc: &spec_runtime::DocEntry) -> (bool, Option<String>) {
    // Normalize doc_type: treat "how-to" as "how_to"
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
        "ci_workflow" => {
            // CI workflow YAML: no validation
        }
        _ => {
            // Unknown doc_type
            return (false, Some(format!("Unknown doc_type '{}'", doc.doc_type)));
        }
    }

    (true, None)
}

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
                        open_questions.push(QuestionSummary {
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

#[derive(Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub req: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<TaskOut>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskOut {
    pub id: String,
    pub title: String,
    pub requirement: String,
    pub acs: Vec<String>,
    pub status: String,
    pub owner: Option<String>,
    pub labels: Vec<String>,
    pub docs: Option<TaskDocsOut>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskDocsOut {
    pub design: Vec<String>,
    pub plan: Vec<String>,
}

/// Normalize a raw status string to its canonical display form.
/// Uses the canonical `FromStr` implementation from `TaskStatus`.
fn normalize_status(raw: &str) -> String {
    raw.parse::<TaskStatus>().map(|s| s.to_string()).unwrap_or_else(|_| {
        tracing::warn!(
            raw_status = raw,
            normalized_status = "Todo",
            "Unknown task status provided; defaulting to Todo"
        );
        TaskStatus::Todo.to_string()
    })
}

async fn get_tasks(
    State(state): State<AppState>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<TasksResponse>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .map_err(|e| AppError::spec_load_error("load specs/tasks.yaml", e))?;

    let state_map = tasks_state::get_all_tasks(&root.join("specs/tasks_state.yaml"))
        .map_err(|e| AppError::spec_load_error("load task state", e))?;

    let tasks = tasks_spec
        .tasks
        .into_iter()
        .filter_map(|t| {
            let effective_status = state_map
                .get(&TaskId(t.id.clone()))
                .cloned()
                .map(task_status_to_string)
                .unwrap_or_else(|| normalize_status(&t.status));

            if filters.status.as_ref().is_some_and(|s| !effective_status.eq_ignore_ascii_case(s)) {
                return None;
            }

            if filters.req.as_ref().is_some_and(|r| t.requirement != *r) {
                return None;
            }

            let mut task_out: TaskOut = t.into();
            task_out.status = effective_status;

            Some(task_out)
        })
        .collect();

    Ok(Json(TasksResponse { tasks }))
}

impl From<spec_runtime::Task> for TaskOut {
    fn from(t: spec_runtime::Task) -> Self {
        TaskOut {
            id: t.id,
            title: t.title,
            requirement: t.requirement,
            acs: t.acs,
            status: t.status,
            owner: t.owner,
            labels: t.labels,
            docs: t.docs.map(|d| TaskDocsOut { design: d.design, plan: d.plan }),
        }
    }
}

#[derive(Serialize)]
pub struct CoverageSummary {
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CoverageDetail {
    pub id: String,
    pub title: String,
    pub status: String,
    pub story: String,
    pub requirement: String,
    pub scenarios: Vec<String>,
}

#[derive(Serialize)]
pub struct CoverageResponse {
    pub summary: CoverageSummary,
    pub details: Vec<CoverageDetail>,
}

// Cucumber JSON format structures for parsing BDD output
#[derive(Debug, Deserialize)]
struct CucumberReport(Vec<CucumberFeature>);

#[derive(Debug, Deserialize)]
struct CucumberFeature {
    #[allow(dead_code)]
    uri: String,
    elements: Vec<CucumberElement>,
}

#[derive(Debug, Deserialize)]
struct CucumberElement {
    name: String,
    #[serde(rename = "type")]
    element_type: String,
    tags: Vec<CucumberTag>,
    steps: Vec<CucumberStep>,
}

#[derive(Debug, Deserialize)]
struct CucumberTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CucumberStep {
    result: CucumberStepResult,
}

#[derive(Debug, Deserialize)]
struct CucumberStepResult {
    status: String,
}

async fn get_coverage(State(state): State<AppState>) -> Json<CoverageResponse> {
    let root = &state.workspace_root;

    // Load spec ledger to get all ACs
    let specs = match load_all_specs(root) {
        Ok(s) => s,
        Err(_) => {
            // Return empty response if specs can't be loaded
            return Json(CoverageResponse {
                summary: CoverageSummary { passing: 0, failing: 0, unknown: 0, total: 0 },
                details: vec![],
            });
        }
    };

    // Build a map of all ACs from the ledger
    let mut ac_map: HashMap<String, (String, String, String)> = HashMap::new();
    for story in &specs.ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                ac_map.insert(ac.id.clone(), (story.id.clone(), req.id.clone(), ac.text.clone()));
            }
        }
    }

    // Try to parse BDD results from JSON report
    let bdd_json_path = root.join("target/ac_report.json");
    let mut ac_status: HashMap<String, String> = HashMap::new();
    let mut ac_scenarios: HashMap<String, Vec<String>> = HashMap::new();

    if bdd_json_path.exists()
        && let Ok(content) = fs::read_to_string(&bdd_json_path)
        && let Ok(report) = serde_json::from_str::<CucumberReport>(&content)
    {
        for feature in report.0 {
            for element in feature.elements {
                // Only process scenarios
                if element.element_type == "scenario" {
                    // Extract AC IDs from tags
                    let ac_ids: Vec<String> = element
                        .tags
                        .iter()
                        .filter_map(|tag| {
                            // Tags in Cucumber JSON include an @ prefix - normalize before matching
                            let tag_name = tag.name.trim_start_matches('@');
                            if tag_name.starts_with("AC-") {
                                Some(tag_name.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Determine if scenario passed (all steps passed)
                    let passed = element.steps.iter().all(|step| step.result.status == "passed");

                    // Update status and scenarios for each AC
                    for ac_id in ac_ids {
                        // Track scenario name
                        ac_scenarios.entry(ac_id.clone()).or_default().push(element.name.clone());

                        // Update status (if any scenario fails, AC fails)
                        let current_status = ac_status.entry(ac_id.clone()).or_insert_with(|| {
                            if passed { "passing".to_string() } else { "failing".to_string() }
                        });

                        if !passed {
                            *current_status = "failing".to_string();
                        }
                    }
                }
            }
        }
    }

    // Build details and compute summary
    let mut passing = 0;
    let mut failing = 0;
    let mut unknown = 0;
    let mut details = Vec::new();

    for (ac_id, (story_id, req_id, title)) in &ac_map {
        let status = ac_status.get(ac_id).cloned().unwrap_or_else(|| "unknown".to_string());
        let scenarios = ac_scenarios.get(ac_id).cloned().unwrap_or_default();

        match status.as_str() {
            "passing" => passing += 1,
            "failing" => failing += 1,
            _ => unknown += 1,
        }

        details.push(CoverageDetail {
            id: ac_id.clone(),
            title: title.clone(),
            status,
            story: story_id.clone(),
            requirement: req_id.clone(),
            scenarios,
        });
    }

    // Sort details by ID for consistent output
    details.sort_by(|a, b| a.id.cmp(&b.id));

    let total = passing + failing + unknown;

    Json(CoverageResponse {
        summary: CoverageSummary { passing, failing, unknown, total },
        details,
    })
}

async fn get_schema() -> Json<spec_runtime::PlatformSchemas> {
    Json(spec_runtime::get_all_schemas())
}

async fn get_schema_by_name_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<spec_runtime::SchemaInfo>, AppError> {
    spec_runtime::get_schema_by_name(&name)
        .map(Json)
        .ok_or_else(|| AppError::not_found(format!("Schema '{}' not found", name)))
}

fn task_status_to_string(status: TaskStatus) -> String {
    format!("{status:?}")
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

    #[test]
    fn normalizes_common_status_variants() {
        assert_eq!(normalize_status("open"), "Todo");
        assert_eq!(normalize_status("in_progress"), "InProgress");
        assert_eq!(normalize_status("in-progress"), "InProgress");
        assert_eq!(normalize_status("review"), "Review");
        assert_eq!(normalize_status("done"), "Done");
        assert_eq!(normalize_status("InProgress"), "InProgress");
    }

    #[test]
    fn defaults_unknown_statuses_to_todo() {
        assert_eq!(normalize_status("blocked"), "Todo");
        assert_eq!(normalize_status(""), "Todo");
    }
}

#[derive(Deserialize)]
struct TaskGraphQuery {
    format: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum TaskGraphResponse {
    Json(spec_runtime::tasks::TaskGraph),
    Mermaid { mermaid: String },
}

async fn get_task_graph(
    State(state): State<AppState>,
    Query(query): Query<TaskGraphQuery>,
) -> Result<Json<TaskGraphResponse>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .map_err(|e| AppError::spec_load_error("load specs/tasks.yaml", e))?;

    let graph = spec_runtime::tasks::build_task_graph(&tasks_spec);

    let response = match query.format.as_deref() {
        Some("mermaid") => {
            let mermaid = spec_runtime::tasks::generate_mermaid_diagram(&graph);
            TaskGraphResponse::Mermaid { mermaid }
        }
        _ => TaskGraphResponse::Json(graph),
    };

    Ok(Json(response))
}

/// UI Contract endpoint - returns the governed UI surface definitions.
///
/// Returns the UI contract specification which defines screens, regions,
/// and stable `data-uiid` identifiers that agents, tests, and consumers
/// can rely on.
async fn get_ui_contract(
    State(state): State<AppState>,
) -> Result<Json<spec_runtime::UiContract>, AppError> {
    let root = &state.workspace_root;
    spec_runtime::load_ui_contract(&root.join("specs/ui_contract.yaml"))
        .map(Json)
        .map_err(|e| AppError::spec_load_error("load UI contract", e))
}
