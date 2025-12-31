use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::{friction, questions};

/// IDP snapshot output structure (machine-readable contract for IDPs)
#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct GovernanceHealth {
    /// Overall status: "healthy", "degraded", or "failing"
    pub status: String,
    /// AC coverage metrics from BDD tests
    pub ac_coverage: AcCoverage,
    /// Story/requirement/AC counts from spec_ledger
    pub spec_counts: SpecCounts,
}

#[derive(Debug, Serialize)]
pub struct AcCoverage {
    pub total: usize,
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
}

#[derive(Debug, Serialize)]
pub struct SpecCounts {
    pub stories: usize,
    pub requirements: usize,
    pub acceptance_criteria: usize,
}

#[derive(Debug, Serialize)]
pub struct DocumentationMetrics {
    pub total: usize,
    pub valid: usize,
    pub with_issues: usize,
}

#[derive(Debug, Serialize)]
pub struct TaskHints {
    pub total_pending: usize,
    pub total_in_progress: usize,
    pub friction_count: usize,
    pub question_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub high_priority: Vec<TaskHint>,
}

#[derive(Debug, Serialize)]
pub struct TaskHint {
    pub task_id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
}

// Internal structures for parsing source files

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
    /// AC ID. Deserialized for schema completeness; count derived from Vec length.
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
    /// Doc ID. Deserialized for schema completeness.
    #[expect(dead_code, reason = "deserialized for schema completeness")]
    id: String,
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

/// Arguments for idp-snapshot command
#[derive(Debug)]
pub struct IdpSnapshotArgs {
    pub output: Option<String>,
    pub pretty: bool,
}

/// Generate IDP snapshot
pub fn run(args: IdpSnapshotArgs) -> Result<()> {
    let snapshot = generate_snapshot()?;

    let json = if args.pretty {
        serde_json::to_string_pretty(&snapshot)
    } else {
        serde_json::to_string(&snapshot)
    }
    .context("Failed to serialize snapshot to JSON")?;

    if let Some(output_path) = args.output {
        fs::write(&output_path, &json)
            .with_context(|| format!("Failed to write snapshot to {}", output_path))?;
        eprintln!("IDP snapshot written to: {}", output_path);
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn generate_snapshot() -> Result<IdpSnapshot> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Load spec ledger
    let ledger = load_ledger()?;
    let template_version = ledger.metadata.template_version.clone();
    let spec_counts = count_specs(&ledger);

    // Load service metadata
    let service_id = load_service_id();

    // Load AC coverage
    let ac_coverage = load_ac_coverage();

    // Load documentation metrics
    let documentation = load_doc_metrics();

    // Load task hints
    let task_hints = load_task_hints()?;

    // Determine overall governance status
    let status = if ac_coverage.failing > 0 {
        "degraded".to_string()
    } else {
        "healthy".to_string() // No failures is still healthy
    };

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

fn load_ledger() -> Result<Ledger> {
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    let content = fs::read_to_string(ledger_path)
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

fn load_service_id() -> Option<String> {
    let path = Path::new("specs/service_metadata.yaml");
    let content = fs::read_to_string(path).ok()?;
    let metadata: ServiceMetadata = serde_yaml::from_str(&content).ok()?;
    metadata.service_id
}

fn load_ac_coverage() -> AcCoverage {
    // Try to load from target/ac_report.json (generated by ac-status)
    let report_path = Path::new("target/ac_report.json");
    if report_path.exists()
        && let Ok(content) = fs::read_to_string(report_path)
    {
        // The ac_report.json is a single-line JSON array of test results
        // We can count passing/failing by parsing the test results
        if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            let mut passing = 0;
            let mut failing = 0;
            let mut unknown = 0;

            for result in results {
                if let Some(elements) = result.get("elements").and_then(|v| v.as_array()) {
                    for element in elements {
                        // Check if this element has @AC-* tags
                        if let Some(tags) = element.get("tags").and_then(|v| v.as_array()) {
                            let has_ac_tag = tags.iter().any(|tag| {
                                tag.get("name")
                                    .and_then(|n| n.as_str())
                                    .is_some_and(|name| name.starts_with("AC-"))
                            });

                            if has_ac_tag {
                                // Check step results to determine pass/fail
                                if let Some(steps) = element.get("steps").and_then(|v| v.as_array())
                                {
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
    }

    // Fallback: parse from docs/feature_status.md if ac_report.json is unavailable
    let feature_status_path = Path::new("docs/feature_status.md");
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

fn load_doc_metrics() -> DocumentationMetrics {
    let doc_index_path = Path::new("specs/doc_index.yaml");
    if doc_index_path.exists()
        && let Ok(content) = fs::read_to_string(doc_index_path)
        && let Ok(index) = serde_yaml::from_str::<DocIndex>(&content)
    {
        let total = index.docs.len();
        // For now, assume all docs are valid unless we have a validation mechanism
        // This could be enhanced later to check file existence or run docs-check
        return DocumentationMetrics { total, valid: total, with_issues: 0 };
    }

    DocumentationMetrics { total: 0, valid: 0, with_issues: 0 }
}

fn load_task_hints() -> Result<TaskHints> {
    let tasks_path = Path::new("specs/tasks.yaml");
    let mut total_pending = 0;
    let mut total_in_progress = 0;
    let mut high_priority = Vec::new();

    if tasks_path.exists() {
        let content = fs::read_to_string(tasks_path)
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
    let friction_entries = friction::load_all_friction_entries().unwrap_or_default();
    let friction_count = friction_entries.iter().filter(|f| f.status == "open").count();

    let questions = questions::load_all_questions().unwrap_or_default();
    let question_count = questions.iter().filter(|q| q.status == "open").count();

    Ok(TaskHints {
        total_pending,
        total_in_progress,
        friction_count,
        question_count,
        high_priority,
    })
}
