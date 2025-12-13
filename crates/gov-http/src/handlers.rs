//! Platform endpoint handlers.

use crate::error::PlatformError;
use crate::state::PlatformState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
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
