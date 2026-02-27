//! IDP snapshot endpoint for platform introspection.
//!
//! Provides machine-readable contract for IDPs including:
//! - Governance health metrics
//! - Documentation metrics
//! - Task hints for agents

#![allow(dead_code)]

use axum::{Json, Router, extract::State, routing::get};
use http_errors::HttpError;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::instrument;

// ============================================================================
// IDP Snapshot DTOs
// ============================================================================

/// IDP snapshot output structure (machine-readable contract for IDPs).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdpSnapshot {
    /// ISO 8601 timestamp of snapshot creation
    pub timestamp: String,
    /// Template version from spec_ledger.yaml
    pub template_version: String,
    /// Service ID from service_metadata.yaml (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    /// Governance health metrics
    pub governance_health: GovernanceHealth,
    /// Documentation metrics
    pub documentation: DocumentationMetrics,
    /// Task hints for agents
    pub task_hints: TaskHints,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GovernanceHealth {
    /// Overall status: "healthy", "degraded", or "failing"
    pub status: String,
    /// AC coverage metrics from BDD tests
    pub ac_coverage: AcCoverage,
    /// Story/requirement/AC counts from spec_ledger
    pub spec_counts: SpecCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AcCoverage {
    pub total: usize,
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpecCounts {
    pub stories: usize,
    pub requirements: usize,
    pub acceptance_criteria: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DocumentationMetrics {
    pub total: usize,
    pub valid: usize,
    pub with_issues: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskHints {
    pub total_pending: usize,
    pub total_in_progress: usize,
    pub friction_count: usize,
    pub question_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub high_priority: Vec<TaskHint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskHint {
    pub task_id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
}

// ============================================================================
// Internal Structures for Parsing
// ============================================================================

#[derive(Debug, Deserialize)]
struct Ledger {
    metadata: LedgerMetadata,
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
}

#[derive(Debug, Deserialize)]
struct Story {
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    #[expect(
        dead_code,
        reason = "deserialized for schema completeness; count derived from Vec length"
    )]
    id: String,
}

#[derive(Debug, Deserialize)]
struct DocIndex {
    docs: Vec<DocEntry>,
}

#[derive(Debug, Deserialize)]
struct DocEntry {
    #[expect(
        dead_code,
        reason = "deserialized for schema completeness; validation uses other fields"
    )]
    id: String,
    #[serde(default)]
    doc_type: String,
    #[serde(default)]
    stories: Vec<String>,
    #[serde(default)]
    requirements: Vec<String>,
    #[serde(default)]
    acs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TasksFile {
    tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Deserialize)]
struct TaskDefinition {
    id: String,
    #[serde(default)]
    title: Option<String>,
    status: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    requirement: Option<String>,
    #[serde(default)]
    acs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ServiceMetadata {
    #[serde(default)]
    service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FrictionEntry {
    #[expect(
        dead_code,
        reason = "deserialized for schema completeness; filtering uses status field"
    )]
    id: String,
    #[serde(default)]
    status: String,
}

#[derive(Debug, Deserialize)]
struct Question {
    #[expect(
        dead_code,
        reason = "deserialized for schema completeness; filtering uses status field"
    )]
    id: String,
    #[serde(default)]
    status: String,
}

// ============================================================================
// Router and Handler
// ============================================================================

/// Create the IDP router.
pub fn router<S>() -> Router<S>
where
    S: super::PlatformState + Clone + Send + Sync + 'static,
{
    Router::<S>::new().route("/idp/snapshot", get(get_idp_snapshot::<S>))
}

/// GET /platform/idp/snapshot - Get IDP snapshot with governance health and task hints.
#[allow(clippy::result_large_err)]
#[instrument(skip(state))]
pub async fn get_idp_snapshot<S>(State(state): State<S>) -> Result<Json<IdpSnapshot>, HttpError>
where
    S: super::PlatformState,
{
    let root = state.workspace_root().to_path_buf();

    let snapshot = tokio::task::spawn_blocking(move || generate_snapshot(&root))
        .await
        .map_err(|e| HttpError::internal_error(format!("Task join error: {}", e)))?
        .map_err(|e| {
            HttpError::internal_error(format!("Failed to generate IDP snapshot: {}", e))
        })?;

    Ok(Json(snapshot))
}

// ============================================================================
// Snapshot Generation
// ============================================================================

fn generate_snapshot(root: &std::path::Path) -> anyhow::Result<IdpSnapshot> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Load spec ledger
    let ledger = load_ledger(root)?;
    let template_version = ledger.metadata.template_version.clone();
    let spec_counts = count_specs(&ledger);

    // Load service metadata
    let service_id = load_service_id(root);

    // Load AC coverage
    let ac_coverage = load_ac_coverage(root);

    // Load documentation metrics
    let documentation = load_doc_metrics(root);

    // Load task hints
    let task_hints = load_task_hints(root)?;

    // Determine overall governance status
    let status =
        if ac_coverage.failing > 0 { "degraded".to_string() } else { "healthy".to_string() };

    let governance_health = GovernanceHealth { status, ac_coverage, spec_counts };

    Ok(IdpSnapshot {
        timestamp,
        template_version,
        service_id,
        governance_health,
        documentation,
        task_hints,
    })
}

fn load_ledger(root: &std::path::Path) -> anyhow::Result<Ledger> {
    let ledger_path = root.join("specs/spec_ledger.yaml");
    let content = fs::read_to_string(&ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))
}

fn count_specs(ledger: &Ledger) -> SpecCounts {
    let stories = ledger.stories.len();
    let requirements: usize = ledger.stories.iter().map(|s| s.requirements.len()).sum();
    let acceptance_criteria: usize = ledger
        .stories
        .iter()
        .flat_map(|s| &s.requirements)
        .map(|r| r.acceptance_criteria.len())
        .sum();

    SpecCounts { stories, requirements, acceptance_criteria }
}

fn load_service_id(root: &std::path::Path) -> Option<String> {
    let path = root.join("specs/service_metadata.yaml");
    let content = fs::read_to_string(path).ok()?;
    let metadata: ServiceMetadata = serde_yaml::from_str(&content).ok()?;
    metadata.service_id
}

/// Load AC coverage from feature_status.md or ac_report.json.
pub fn load_ac_coverage(root: &std::path::Path) -> AcCoverage {
    // Try to load from target/ac_report.json (generated by ac-status)
    let report_path = root.join("target/ac_report.json");
    if report_path.exists()
        && let Ok(content) = fs::read_to_string(report_path)
        && let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(&content)
    {
        let mut passing = 0;
        let mut failing = 0;
        let mut unknown = 0;

        for result in results {
            if let Some(elements) = result.get("elements").and_then(|v| v.as_array()) {
                for element in elements {
                    if let Some(tags) = element.get("tags").and_then(|v| v.as_array()) {
                        let has_ac_tag = tags.iter().any(|tag| {
                            tag.get("name")
                                .and_then(|n| n.as_str())
                                .is_some_and(|name| name.starts_with("AC-"))
                        });

                        if has_ac_tag {
                            if let Some(steps) = element.get("steps").and_then(|v| v.as_array()) {
                                let all_passed = steps.iter().all(|step| {
                                    step.get("result")
                                        .and_then(|r| r.get("status"))
                                        .and_then(|s| s.as_str())
                                        == Some("passed")
                                });

                                if all_passed && !steps.is_empty() {
                                    passing += 1;
                                } else if !steps.is_empty() {
                                    failing += 1;
                                } else {
                                    unknown += 1;
                                }
                            } else {
                                unknown += 1;
                            }
                        }
                    }
                }
            }
        }

        return AcCoverage { total: passing + failing + unknown, passing, failing, unknown };
    }

    // Fallback: parse from docs/feature_status.md if ac_report.json is unavailable
    let feature_status_path = root.join("docs/feature_status.md");
    if feature_status_path.exists()
        && let Ok(content) = fs::read_to_string(feature_status_path)
    {
        let mut passing = 0;
        let mut failing = 0;
        let mut unknown = 0;

        for line in content.lines() {
            if line.contains("| AC-") {
                if line.contains("pass") {
                    passing += 1;
                } else if line.contains("fail") {
                    failing += 1;
                } else if line.contains("unknown") {
                    unknown += 1;
                }
            }
        }

        if passing + failing + unknown > 0 {
            return AcCoverage { total: passing + failing + unknown, passing, failing, unknown };
        }
    }

    // No coverage data available
    AcCoverage { total: 0, passing: 0, failing: 0, unknown: 0 }
}

fn load_doc_metrics(root: &std::path::Path) -> DocumentationMetrics {
    let doc_index_path = root.join("specs/doc_index.yaml");
    if doc_index_path.exists()
        && let Ok(content) = fs::read_to_string(doc_index_path)
        && let Ok(index) = serde_yaml::from_str::<DocIndex>(&content)
    {
        let total = index.docs.len();

        // Validate docs against doc_type contract
        let mut valid = 0;
        let mut with_issues = 0;

        for doc in &index.docs {
            if validate_doc_type_contract(doc) {
                valid += 1;
            } else {
                with_issues += 1;
            }
        }

        return DocumentationMetrics { total, valid, with_issues };
    }

    DocumentationMetrics { total: 0, valid: 0, with_issues: 0 }
}

/// Validate doc_type contract for a single document.
fn validate_doc_type_contract(doc: &DocEntry) -> bool {
    let doc_type = doc.doc_type.replace('-', "_");

    match doc_type.as_str() {
        "how_to" => !doc.requirements.is_empty() || !doc.acs.is_empty(),
        "explanation" => !doc.stories.is_empty() || !doc.requirements.is_empty(),
        "design_doc" => !doc.requirements.is_empty(),
        "reference" => !doc.requirements.is_empty() || !doc.acs.is_empty(),
        "status" => !doc.requirements.is_empty() && !doc.acs.is_empty(),
        "adr" => !doc.requirements.is_empty(),
        "guide" => !doc.requirements.is_empty() || !doc.acs.is_empty(),
        "impl_plan" => !doc.requirements.is_empty() && !doc.acs.is_empty(),
        "requirements_doc" => !doc.requirements.is_empty(),
        "ci_workflow" | "" => true,
        _ => false,
    }
}

fn load_task_hints(root: &std::path::Path) -> anyhow::Result<TaskHints> {
    let tasks_path = root.join("specs/tasks.yaml");
    let mut total_pending = 0;
    let mut total_in_progress = 0;
    let mut high_priority = Vec::new();

    if tasks_path.exists() {
        let content = fs::read_to_string(&tasks_path)
            .with_context(|| format!("Failed to read tasks: {}", tasks_path.display()))?;

        let tasks_file: TasksFile = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse tasks YAML: {}", tasks_path.display()))?;

        for task in tasks_file.tasks {
            let status = task.status.as_deref().unwrap_or("open");

            match status {
                "open" => total_pending += 1,
                "in_progress" => total_in_progress += 1,
                _ => {}
            }

            // Collect high-priority tasks (pending or in-progress)
            if matches!(status, "open" | "in_progress") {
                let requirement_ids =
                    if let Some(req) = task.requirement { vec![req] } else { Vec::new() };

                high_priority.push(TaskHint {
                    task_id: task.id,
                    title: task.title.unwrap_or_default(),
                    status: status.to_string(),
                    owner: task.owner,
                    requirement_ids,
                    ac_ids: task.acs,
                });
            }
        }
    }

    // Sort high_priority: in_progress first, then by task_id
    high_priority.sort_by(|a, b| {
        let status_cmp = match (a.status.as_str(), b.status.as_str()) {
            ("in_progress", "open") => std::cmp::Ordering::Less,
            ("open", "in_progress") => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
        status_cmp.then_with(|| a.task_id.cmp(&b.task_id))
    });

    // Limit to top 5 high-priority tasks
    high_priority.truncate(5);

    // Load friction and question counts
    let friction_count = load_friction_count(root);
    let question_count = load_question_count(root);

    Ok(TaskHints {
        total_pending,
        total_in_progress,
        friction_count,
        question_count,
        high_priority,
    })
}

fn load_friction_count(root: &std::path::Path) -> usize {
    let friction_dir = root.join("friction");
    if !friction_dir.exists() {
        return 0;
    }

    let mut count = 0;

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
                && (friction.status == "open" || friction.status.is_empty())
            {
                count += 1;
            }
        }
    }

    count
}

fn load_question_count(root: &std::path::Path) -> usize {
    let questions_dir = root.join("questions");
    if !questions_dir.exists() {
        return 0;
    }

    let mut count = 0;

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
                && question.status == "open"
            {
                count += 1;
            }
        }
    }

    count
}

// ============================================================================
// Helper Trait
// ============================================================================

trait WithContext<T> {
    fn with_context<F>(self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: std::fmt::Display> WithContext<T> for Result<T, E> {
    fn with_context<F>(self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| anyhow::anyhow!("{}: {}", f(), e))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_doc_type_how_to() {
        let doc = DocEntry {
            id: "DOC-001".to_string(),
            doc_type: "how_to".to_string(),
            stories: vec![],
            requirements: vec!["REQ-001".to_string()],
            acs: vec![],
        };
        assert!(validate_doc_type_contract(&doc));

        let doc_no_refs = DocEntry {
            id: "DOC-002".to_string(),
            doc_type: "how_to".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
        };
        assert!(!validate_doc_type_contract(&doc_no_refs));
    }

    #[test]
    fn test_validate_doc_type_design_doc() {
        let doc = DocEntry {
            id: "DOC-003".to_string(),
            doc_type: "design_doc".to_string(),
            stories: vec![],
            requirements: vec!["REQ-001".to_string()],
            acs: vec![],
        };
        assert!(validate_doc_type_contract(&doc));

        let doc_no_req = DocEntry {
            id: "DOC-004".to_string(),
            doc_type: "design_doc".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
        };
        assert!(!validate_doc_type_contract(&doc_no_req));
    }

    #[test]
    fn test_validate_doc_type_ci_workflow() {
        let doc = DocEntry {
            id: "DOC-005".to_string(),
            doc_type: "ci_workflow".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
        };
        assert!(validate_doc_type_contract(&doc));
    }

    #[test]
    fn test_validate_doc_type_unknown() {
        let doc = DocEntry {
            id: "DOC-006".to_string(),
            doc_type: "unknown_type".to_string(),
            stories: vec![],
            requirements: vec![],
            acs: vec![],
        };
        assert!(!validate_doc_type_contract(&doc));
    }
}
