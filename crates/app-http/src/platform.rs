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
use spec_runtime::{load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;

mod ui;

/// Platform API routes (mounted at /platform)
pub fn router(state: AppState) -> Router {
    Router::new()
        // API routes
        .route("/graph", get(get_graph))
        .route("/schema", get(get_schema))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
        .route("/coverage", get(get_coverage))
        .route("/tasks", get(get_tasks))
        .route("/tasks/suggest-next", get(get_suggest_next))
        .with_state(state)
}

/// UI routes (mounted at root)
pub fn ui_router(state: AppState) -> Router {
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
struct PolicyStatus {
    status: String,
}

#[derive(Deserialize)]
struct PolicyStatusReport {
    summary: String,
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

    Json(PlatformStatus {
        service: service_info,
        governance: GovernanceStatus {
            ledger: ledger_counts,
            devex: devex_counts,
            docs: doc_counts,
            tasks: task_counts,
            policies: PolicyStatus { status: policy_status },
        },
    })
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
                .unwrap_or_else(|| t.status.clone());

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

#[derive(Serialize)]
struct PlatformEndpointSchema {
    path: String,
    method: String,
    request_type: Option<String>,
    response_type: String,
}

#[derive(Serialize)]
struct PlatformSchema {
    endpoints: Vec<PlatformEndpointSchema>,
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

async fn get_schema() -> Json<PlatformSchema> {
    fn ep(
        path: &str,
        method: &str,
        request: Option<&str>,
        response: &str,
    ) -> PlatformEndpointSchema {
        PlatformEndpointSchema {
            path: path.to_string(),
            method: method.to_string(),
            request_type: request.map(|s| s.to_string()),
            response_type: response.to_string(),
        }
    }

    Json(PlatformSchema {
        endpoints: vec![
            ep("/platform/status", "GET", None, "PlatformStatus"),
            ep("/platform/graph", "GET", None, "PlatformGraph"),
            ep("/platform/devex/flows", "GET", None, "PlatformDevExFlows"),
            ep("/platform/docs/index", "GET", None, "PlatformDocsIndex"),
            ep("/platform/tasks", "GET", None, "TasksResponse"),
            ep("/platform/agent/hints", "GET", None, "AgentHints"),
        ],
    })
}

fn task_status_to_string(status: TaskStatus) -> String {
    format!("{status:?}")
}
