//! Platform endpoint handlers.

use crate::error::PlatformError;
use crate::state::PlatformState;
use axum::{
    Json,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use gov_model::TaskStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Platform health endpoint.
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok".to_string() })
}

/// Platform status response (simplified).
#[derive(Serialize)]
pub struct StatusResponse {
    pub governance: GovernanceStatusResponse,
}

#[derive(Serialize)]
pub struct GovernanceStatusResponse {
    pub healthy: bool,
}

/// Platform status endpoint.
pub async fn get_status() -> impl IntoResponse {
    Json(StatusResponse { governance: GovernanceStatusResponse { healthy: true } })
}

// =============================================================================
// Contract Anchor Endpoints
// =============================================================================

/// Get all platform schemas.
///
/// Returns the complete list of platform API schemas.
pub async fn get_schema() -> Json<spec_runtime::PlatformSchemas> {
    Json(spec_runtime::get_all_schemas())
}

/// Get the OpenAPI spec (YAML).
///
/// Returns the OpenAPI document from specs/openapi/openapi.yaml.
pub async fn get_openapi<S>(State(state): State<S>) -> Result<impl IntoResponse, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let openapi_path = ctx.specs_dir().join("openapi/openapi.yaml");
    let content = fs::read_to_string(&openapi_path)
        .map_err(|e| PlatformError::spec_load("OpenAPI spec", e))?;

    Ok(([(header::CONTENT_TYPE, "application/yaml")], content))
}

/// Get a specific schema by name.
///
/// Returns a single schema matching the given name, or 404 if not found.
pub async fn get_schema_by_name(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<spec_runtime::SchemaInfo>, PlatformError> {
    spec_runtime::get_schema_by_name(&name)
        .map(Json)
        .ok_or_else(|| PlatformError::not_found(format!("Schema '{}' not found", name)))
}

/// Get the UI contract.
///
/// Returns the governed UI surface definitions with screens, regions,
/// and stable `data-uiid` identifiers.
pub async fn get_ui_contract<S>(
    State(state): State<S>,
) -> Result<Json<spec_runtime::UiContract>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    spec_runtime::load_ui_contract_with_context(ctx)
        .map(Json)
        .map_err(|e| PlatformError::spec_load("UI contract", e))
}

// =============================================================================
// Docs Index Endpoint
// =============================================================================

/// Response for /platform/docs/index with health info.
#[derive(Serialize)]
pub struct DocsIndexResponse {
    pub schema_version: String,
    pub template_version: String,
    pub docs: Vec<DocInfoWithHealth>,
    pub summary: DocHealthSummary,
}

#[derive(Serialize)]
pub struct DocInfoWithHealth {
    pub id: String,
    pub file: String,
    pub doc_type: String,
    #[serde(default)]
    pub stories: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub acs: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    /// Doc type contract validation result.
    pub doc_type_valid: bool,
    /// Issue description if doc_type_valid is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_type_issue: Option<String>,
}

#[derive(Serialize)]
pub struct DocHealthSummary {
    pub total: usize,
    pub valid: usize,
    pub with_issues: usize,
}

/// Get the docs index with health validation.
///
/// Returns the documentation inventory with doc_type contract validation.
pub async fn get_docs_index<S>(
    State(state): State<S>,
) -> Result<Json<DocsIndexResponse>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let docs = spec_runtime::load_doc_index_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("doc index", e))?;

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

/// Validate doc_type contract for a single document.
///
/// Returns (is_valid, issue_description).
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

// =============================================================================
// Governance Graph Endpoint
// =============================================================================

/// Get the governance graph.
///
/// Returns the full governance graph (stories → REQs → ACs → tests → docs).
pub async fn get_graph<S>(
    State(state): State<S>,
) -> Result<Json<spec_runtime::Graph>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let specs = spec_runtime::load_all_specs_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("specs", e))?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)
        .map_err(|e| PlatformError::internal(format!("Failed to build graph: {}", e)))?;
    Ok(Json(graph))
}

// =============================================================================
// DevEx Flows Endpoint
// =============================================================================

/// Get the DevEx flows specification.
///
/// Returns the developer experience flows and commands specification.
pub async fn get_devex_flows<S>(
    State(state): State<S>,
) -> Result<Json<serde_json::Value>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let devex = spec_runtime::load_devex_flows_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("devex flows", e))?;
    let value = serde_json::to_value(devex)
        .map_err(|e| PlatformError::internal(format!("Failed to serialize devex flows: {}", e)))?;
    Ok(Json(value))
}

// =============================================================================
// AC Coverage Endpoint
// =============================================================================

/// Summary of AC coverage.
#[derive(Serialize)]
pub struct CoverageSummary {
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
    pub total: usize,
}

/// Detail for a single AC's coverage.
#[derive(Serialize)]
pub struct CoverageDetail {
    pub id: String,
    pub title: String,
    pub status: String,
    pub story: String,
    pub requirement: String,
    pub scenarios: Vec<String>,
}

/// Response for /platform/coverage.
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
    /// Feature file URI. Deserialized for schema completeness but not used in coverage logic.
    #[expect(dead_code, reason = "deserialized for schema completeness; coverage uses elements")]
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

/// Get AC coverage from BDD test results.
///
/// Returns a summary and details of acceptance criteria coverage
/// based on Cucumber JSON reports.
pub async fn get_coverage<S>(State(state): State<S>) -> Json<CoverageResponse>
where
    S: PlatformState,
{
    let ctx = state.context();
    let root = ctx.root();

    // Load spec ledger to get all ACs
    let specs = match spec_runtime::load_all_specs_with_context(ctx) {
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
        && let Ok(content) = std::fs::read_to_string(&bdd_json_path)
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

// =============================================================================
// Tasks Endpoints
// =============================================================================

/// Query parameters for filtering tasks.
#[derive(Debug, Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub req: Option<String>,
}

/// Response for /platform/tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<TaskOut>,
}

/// Task output DTO with full metadata.
#[derive(Debug, Serialize, Deserialize)]
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

/// Task docs output DTO.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDocsOut {
    pub design: Vec<String>,
    pub plan: Vec<String>,
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

/// Normalize a raw status string to its canonical display form.
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

/// Convert TaskStatus to string for display.
fn task_status_to_string(status: TaskStatus) -> String {
    status.to_string()
}

/// Get all tasks with optional filtering.
///
/// Returns tasks from specs/tasks.yaml with status overlay from the governance repository.
pub async fn get_tasks<S>(
    State(state): State<S>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<TasksResponse>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();
    let repo = state.governance_repo();

    // Load task definitions from spec (has full metadata)
    let tasks_spec = spec_runtime::load_tasks_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("tasks.yaml", e))?;

    // Get status overlay from governance repository
    let all_tasks = repo
        .find_all_tasks()
        .map_err(|e| PlatformError::internal(format!("Failed to load task states: {}", e)))?;

    let status_map: HashMap<String, TaskStatus> =
        all_tasks.into_iter().map(|t| (t.id.0, t.status)).collect();

    // Merge and filter
    let tasks = tasks_spec
        .tasks
        .into_iter()
        .filter_map(|t| {
            let effective_status = status_map
                .get(&t.id)
                .cloned()
                .map(task_status_to_string)
                .unwrap_or_else(|| normalize_status(&t.status));

            // Apply filters
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

/// Query parameters for suggest-next endpoint.
#[derive(Debug, Deserialize)]
pub struct SuggestNextQuery {
    pub task: String,
}

/// Get suggested next task sequence.
///
/// Returns a sequence of suggested tasks based on the current task and workflow dependencies.
pub async fn get_suggest_next<S>(
    State(state): State<S>,
    Query(q): Query<SuggestNextQuery>,
) -> Result<Json<spec_runtime::tasks::SuggestedSequence>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();

    let tasks_spec = spec_runtime::load_tasks_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("tasks.yaml", e))?;
    let devex_spec = spec_runtime::load_devex_flows_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("devex_flows.yaml", e))?;
    let ledger = spec_runtime::load_spec_ledger_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("spec_ledger.yaml", e))?;

    let suggestion =
        spec_runtime::tasks::suggest_next(ctx.root(), &q.task, &tasks_spec, &devex_spec, &ledger)
            .map_err(|e| PlatformError::internal(format!("Failed to generate suggestion: {}", e)))?;

    Ok(Json(suggestion))
}

/// Query parameters for task graph endpoint.
#[derive(Debug, Deserialize)]
pub struct TaskGraphQuery {
    pub format: Option<String>,
}

/// Response for /platform/tasks/graph.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TaskGraphResponse {
    Json(spec_runtime::tasks::TaskGraph),
    Mermaid { mermaid: String },
}

/// Get task dependency graph.
///
/// Returns the task dependency graph in JSON or Mermaid format.
pub async fn get_task_graph<S>(
    State(state): State<S>,
    Query(query): Query<TaskGraphQuery>,
) -> Result<Json<TaskGraphResponse>, PlatformError>
where
    S: PlatformState,
{
    let ctx = state.context();

    let tasks_spec = spec_runtime::load_tasks_with_context(ctx)
        .map_err(|e| PlatformError::spec_load("tasks.yaml", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_status_variants() {
        // Standard canonical forms
        assert_eq!(normalize_status("Todo"), "Todo");
        assert_eq!(normalize_status("InProgress"), "InProgress");
        assert_eq!(normalize_status("Review"), "Review");
        assert_eq!(normalize_status("Done"), "Done");

        // Case-insensitive variants
        assert_eq!(normalize_status("todo"), "Todo");
        assert_eq!(normalize_status("TODO"), "Todo");
        assert_eq!(normalize_status("inprogress"), "InProgress");
        assert_eq!(normalize_status("INPROGRESS"), "InProgress");

        // Alternative spellings
        assert_eq!(normalize_status("in_progress"), "InProgress");
        assert_eq!(normalize_status("in-progress"), "InProgress");
        assert_eq!(normalize_status("In_Progress"), "InProgress");

        // Legacy aliases
        assert_eq!(normalize_status("open"), "Todo");
        assert_eq!(normalize_status("Open"), "Todo");
        assert_eq!(normalize_status("closed"), "Done");
        assert_eq!(normalize_status("Closed"), "Done");
    }

    #[test]
    fn defaults_unknown_statuses_to_todo() {
        // Unknown statuses should default to Todo with a warning log
        assert_eq!(normalize_status("garbage"), "Todo");
        assert_eq!(normalize_status("unknown"), "Todo");
        assert_eq!(normalize_status(""), "Todo");
        assert_eq!(normalize_status("Pending"), "Todo");
    }
}
