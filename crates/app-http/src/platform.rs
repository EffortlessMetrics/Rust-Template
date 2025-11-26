use crate::{AppError, AppState, ErrorCode};
use adapters_spec_fs::tasks_state;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use business_core::governance::{TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use spec_runtime::{ValidatedConfig, load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;

mod friction;
mod ui;

/// Platform API routes (mounted at /platform)
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // API routes
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
        .merge(friction::router())
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
) -> Json<spec_runtime::tasks::SuggestedSequence> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .expect("Failed to load tasks.yaml");
    let devex_spec = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex_flows.yaml");
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))
        .expect("Failed to load spec_ledger.yaml");

    let suggestion =
        spec_runtime::tasks::suggest_next(root, &q.task, &tasks_spec, &devex_spec, &ledger)
            .expect("Failed to generate suggestion");

    Json(suggestion)
}

#[derive(Serialize)]
struct PlatformStatus {
    service: ServiceInfo,
    governance: GovernanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<ConfigSummary>,
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
    policies: PolicyStatus,
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
}

#[derive(Serialize)]
struct TaskCounts {
    total: usize,
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

async fn get_graph(State(state): State<AppState>) -> Json<spec_runtime::Graph> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).expect("Failed to load specs");
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)
        .expect("Failed to build graph");
    Json(graph)
}

async fn get_devex_flows(State(state): State<AppState>) -> Json<serde_json::Value> {
    let root = &state.workspace_root;
    let devex = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex flows");
    Json(serde_json::to_value(devex).unwrap())
}

async fn get_docs_index(State(state): State<AppState>) -> Json<serde_json::Value> {
    let root = &state.workspace_root;
    let docs = spec_runtime::load_doc_index(&root.join("specs/doc_index.yaml"))
        .expect("Failed to load doc index");
    Json(serde_json::to_value(docs).unwrap())
}

async fn get_status(State(state): State<AppState>) -> Json<PlatformStatus> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).expect("Failed to load specs");
    let tasks_spec =
        spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).expect("Failed to load tasks");

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

    let doc_counts = DocCounts {
        total: specs.docs.docs.len(),
        design: specs.docs.docs.iter().filter(|d| d.doc_type == "design_doc").count(),
    };

    let task_counts = TaskCounts { total: tasks_spec.tasks.len() };

    // Load question counts
    let question_counts = load_question_counts(root);

    // Load friction counts
    let friction_counts = load_friction_counts(root);

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
        .expect("Failed to load service_metadata.yaml");

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

    Json(PlatformStatus {
        service: service_info,
        governance: GovernanceStatus {
            ledger: ledger_counts,
            devex: devex_counts,
            docs: doc_counts,
            tasks: task_counts,
            questions: question_counts,
            friction: friction_counts,
            policies: PolicyStatus { status: policy_status },
        },
        config,
    })
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

fn normalize_status(raw: &str) -> String {
    let key = raw.trim().to_ascii_lowercase().replace([' ', '-'], "_");

    match key.as_str() {
        "todo" | "open" => "Todo".to_string(),
        "inprogress" | "in_progress" => "InProgress".to_string(),
        "review" => "Review".to_string(),
        "done" | "closed" => "Done".to_string(),
        _ => {
            tracing::warn!(
                raw_status = raw,
                normalized_status = "Todo",
                "Unknown task status provided; defaulting to Todo"
            );
            "Todo".to_string()
        }
    }
}

async fn get_tasks(
    State(state): State<AppState>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<TasksResponse>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).map_err(|err| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to load specs/tasks.yaml: {}", err),
        )
    })?;

    let state_map =
        tasks_state::get_all_tasks(&root.join("specs/tasks_state.yaml")).map_err(|err| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                format!("Failed to load task state: {}", err),
            )
        })?;

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
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).map_err(|err| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to load specs/tasks.yaml: {}", err),
        )
    })?;

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
