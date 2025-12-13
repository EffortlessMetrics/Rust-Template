//! Platform endpoint handlers.

use crate::error::PlatformError;
use crate::state::PlatformState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
    State(state): State<Arc<S>>,
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
    State(state): State<Arc<S>>,
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
    State(state): State<Arc<S>>,
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
    State(state): State<Arc<S>>,
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

/// Get AC coverage from BDD test results.
///
/// Returns a summary and details of acceptance criteria coverage
/// based on Cucumber JSON reports.
pub async fn get_coverage<S>(State(state): State<Arc<S>>) -> Json<CoverageResponse>
where
    S: PlatformState,
{
    let ctx = state.context();
    let root = &ctx.workspace_root;

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
