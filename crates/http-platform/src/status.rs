use crate::PlatformState;
use crate::config::config_summary;
use crate::status_counts::{
    calculate_task_breakdown, load_fork_counts, load_friction_counts, load_question_counts,
};
use axum::{Json, extract::State};
use http_errors::HttpError;
use platform_contract::{
    AcCoverageInfo, ConfigSummary, DevExCounts, DocCounts, ErrorStats, ErrorSummary,
    GovernanceStatus, LedgerCounts, PolicyStatus, ServiceInfo, TaskCounts,
};
use serde::{Deserialize, Serialize};
use spec_runtime::{load_all_specs, load_service_metadata};
use std::fs;
use tracing::instrument;

/// Platform status response.
#[derive(Debug, Clone, Serialize)]
pub(super) struct PlatformStatusResponse {
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
pub(super) async fn get_status<S>(
    State(state): State<S>,
) -> Result<Json<PlatformStatusResponse>, HttpError>
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
    let ac_cov = crate::idp::load_ac_coverage(root);
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

#[derive(Debug, Deserialize)]
struct PolicyStatusReport {
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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
