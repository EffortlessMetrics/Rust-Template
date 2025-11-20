use axum::{Json, Router, extract::Query, routing::get};
use serde::{Deserialize, Serialize};
use spec_runtime::load_all_specs;
use std::fs;
use std::path::PathBuf;

mod ui;

pub fn router() -> Router {
    Router::new()
        // UI routes
        .route("/", get(ui::dashboard))
        .route("/ui", get(ui::dashboard))
        .route("/ui/graph", get(ui::graph_view))
        .route("/ui/flows", get(ui::flows_view))
        // API routes
        .route("/graph", get(get_graph))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
        .route("/tasks", get(get_tasks))
        .route("/tasks/suggest-next", get(get_suggest_next))
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}
