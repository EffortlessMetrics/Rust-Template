use axum::{Json, Router, extract::Query, routing::get};
use serde::{Deserialize, Serialize};
use spec_runtime::load_all_specs;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod ui;

/// Platform API routes (mounted at /platform)
pub fn router() -> Router {
    Router::new()
        // API routes
        .route("/graph", get(get_graph))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
        .route("/coverage", get(get_coverage))
        .route("/tasks", get(get_tasks))
        .route("/tasks/suggest-next", get(get_suggest_next))
}

/// UI routes (mounted at root)
pub fn ui_router() -> Router {
    Router::new()
        .route("/", get(ui::dashboard))
        .route("/ui", get(ui::dashboard))
        .route("/ui/graph", get(ui::graph_view))
        .route("/ui/flows", get(ui::flows_view))
        .route("/ui/coverage", get(ui::coverage_view))
}

#[derive(Deserialize)]
struct SuggestNextQuery {
    task: String,
}

async fn get_suggest_next(
    Query(q): Query<SuggestNextQuery>,
) -> Json<spec_runtime::tasks::SuggestedSequence> {
    let root = workspace_root();
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .expect("Failed to load tasks.yaml");
    let devex_spec = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex_flows.yaml");
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))
        .expect("Failed to load spec_ledger.yaml");

    let suggestion =
        spec_runtime::tasks::suggest_next(&root, &q.task, &tasks_spec, &devex_spec, &ledger)
            .expect("Failed to generate suggestion");

    Json(suggestion)
}

#[derive(Serialize)]
struct PlatformStatus {
    service_id: String,
    template_version: String,
    governance: GovernanceStatus,
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

async fn get_graph() -> Json<spec_runtime::Graph> {
    let root = workspace_root();
    let specs = load_all_specs(&root).expect("Failed to load specs");
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)
        .expect("Failed to build graph");
    Json(graph)
}

async fn get_devex_flows() -> Json<serde_json::Value> {
    let root = workspace_root();
    let devex = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex flows");
    Json(serde_json::to_value(devex).unwrap())
}

async fn get_docs_index() -> Json<serde_json::Value> {
    let root = workspace_root();
    let docs = spec_runtime::load_doc_index(&root.join("specs/doc_index.yaml"))
        .expect("Failed to load doc index");
    Json(serde_json::to_value(docs).unwrap())
}

async fn get_status() -> Json<PlatformStatus> {
    let root = workspace_root();
    let specs = load_all_specs(&root).expect("Failed to load specs");
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

    Json(PlatformStatus {
        service_id: "rust-as-spec-kernel".to_string(),
        template_version: env!("CARGO_PKG_VERSION").to_string(),
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

#[derive(Serialize)]
pub struct TasksResponse {
    pub tasks: Vec<TaskOut>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct TaskDocsOut {
    pub design: Vec<String>,
    pub plan: Vec<String>,
}

async fn get_tasks(Query(filters): Query<TaskFilters>) -> Json<TasksResponse> {
    let root = workspace_root();
    let spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .expect("Failed to load specs/tasks.yaml");

    let tasks = spec
        .tasks
        .into_iter()
        .filter(|t| match &filters.status {
            Some(s) => t.status.eq_ignore_ascii_case(s),
            None => true,
        })
        .filter(|t| match &filters.req {
            Some(r) => t.requirement == *r,
            None => true,
        })
        .map(TaskOut::from)
        .collect();

    Json(TasksResponse { tasks })
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

async fn get_coverage() -> Json<CoverageResponse> {
    let root = workspace_root();

    // Load spec ledger to get all ACs
    let specs = match load_all_specs(&root) {
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

fn workspace_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}
