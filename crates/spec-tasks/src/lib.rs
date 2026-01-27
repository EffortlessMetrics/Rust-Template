//! Task and hint management.
//!
//! This crate provides types and functions for managing work items,
//! task resolution, and hint scoring.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
//! - **View layer**: Depends on spec-ledger for validation
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//!
//! # Example
//!
//! ```ignore
//! use spec_tasks::{load_tasks, validate_task_references};
//!
//! let tasks = load_tasks(Path::new("specs/tasks.yaml"))?;
//! let warnings = validate_task_references(&tasks, &ledger)?;
//! ```

#![allow(missing_docs)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use spec_ledger::SpecLedger;
use spec_types::SpecError;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// Tasks specification.
#[derive(Debug, Deserialize, Serialize)]
pub struct TasksSpec {
    pub schema_version: String,
    pub template_version: String,
    pub tasks: Vec<Task>,
}

/// A work item/task.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub requirement: String,
    pub acs: Vec<String>,
    pub status: String,
    pub owner: Option<String>,
    pub labels: Vec<String>,
    pub docs: Option<TaskDocs>,
    pub summary: String,
    pub recommended_flows: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Documentation references for a task.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskDocs {
    pub design: Vec<String>,
    pub plan: Vec<String>,
}

/// Suggested sequence of actions for a task.
#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedSequence {
    pub task: String,
    pub goal: String,
    pub recommended_flows: Vec<String>,
    pub recommended_sequence: Vec<Action>,
}

/// Step status in suggested sequence.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Satisfied,
}

/// An action in a suggested sequence.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Command { cmd: String, description: String, status: StepStatus },
    Edit { file: String, hint: String, status: StepStatus },
}

/// Referential warning for task validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferentialWarning {
    pub invalid_id: String,
    pub ref_type: String,
    pub source: String,
    pub message: String,
}

// ============================================================================
// Loading
// ============================================================================

/// Load tasks specification from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to `tasks.yaml`
///
/// # Returns
///
/// Returns a parsed [`TasksSpec`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_tasks(path: &Path) -> Result<TasksSpec, anyhow::Error> {
    let content = std::fs::read_to_string(path)
        .map_err(SpecError::Io)
        .with_context(|| format!("Failed to read tasks file: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .map_err(|e| SpecError::YamlParse(e.to_string()))
        .with_context(|| format!("Failed to parse tasks file: {}", path.display()))
}

// ============================================================================
// Validation
// ============================================================================

/// Validate task AC/REQ references against ledger.
///
/// Returns warnings about invalid references found during validation.
///
/// # Arguments
///
/// * `tasks` - Tasks specification to validate
/// * `ledger` - Spec ledger for reference validation
///
/// # Returns
///
/// Returns a list of [`ReferentialWarning`] for any invalid references.
pub fn validate_task_references(
    tasks: &TasksSpec,
    ledger: &SpecLedger,
) -> Result<Vec<ReferentialWarning>> {
    let mut warnings = Vec::new();

    // Build indexes from ledger for O(1) lookup
    let valid_acs: HashSet<_> = ledger
        .stories
        .iter()
        .flat_map(|s| s.requirements.iter())
        .flat_map(|r| r.acceptance_criteria.iter())
        .map(|ac| ac.id.as_str())
        .collect();

    let valid_reqs: HashSet<_> =
        ledger.stories.iter().flat_map(|s| s.requirements.iter()).map(|r| r.id.as_str()).collect();

    // Validate each task
    for task in &tasks.tasks {
        // Validate REQ reference
        if !valid_reqs.contains(task.requirement.as_str()) {
            warnings.push(ReferentialWarning {
                invalid_id: task.requirement.clone(),
                ref_type: "requirement".to_string(),
                source: task.id.clone(),
                message: format!(
                    "Task {} references non-existent requirement {}",
                    task.id, task.requirement
                ),
            });
        }

        // Validate AC references
        for ac_id in &task.acs {
            if !valid_acs.contains(ac_id.as_str()) {
                warnings.push(ReferentialWarning {
                    invalid_id: ac_id.clone(),
                    ref_type: "ac".to_string(),
                    source: task.id.clone(),
                    message: format!("Task {} references non-existent AC {}", task.id, ac_id),
                });
            }
        }
    }

    Ok(warnings)
}

// ============================================================================
// Task Graph
// ============================================================================

/// Task dependency graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskGraph {
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
    pub blocking_relationships: Vec<BlockingRelationship>,
}

/// Node in task graph.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    pub status: String,
    pub requirement: String,
    pub owner: Option<String>,
    pub labels: Vec<String>,
}

/// Edge in task graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
}

/// Blocking relationship between tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockingRelationship {
    pub blocked_task: String,
    pub blocking_tasks: Vec<String>,
    pub reason: String,
}

/// Build a task dependency graph from tasks spec.
///
/// Creates nodes for each task and edges for dependencies.
/// Also identifies blocking relationships.
pub fn build_task_graph(tasks_spec: &TasksSpec) -> TaskGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut blocking = Vec::new();

    // Build nodes
    for task in &tasks_spec.tasks {
        nodes.push(TaskNode {
            id: task.id.clone(),
            title: task.title.clone(),
            status: task.status.clone(),
            requirement: task.requirement.clone(),
            owner: task.owner.clone(),
            labels: task.labels.clone(),
        });
    }

    // Build edges and identify blocking relationships
    let task_status_map: HashMap<String, String> =
        tasks_spec.tasks.iter().map(|t| (t.id.clone(), t.status.clone())).collect();

    for task in &tasks_spec.tasks {
        let mut incomplete_dependencies = Vec::new();

        for dep_id in &task.depends_on {
            edges.push(TaskEdge {
                from: task.id.clone(),
                to: dep_id.clone(),
                edge_type: "depends_on".to_string(),
            });

            // Check if dependency is blocking (not Done)
            if let Some(dep_status) = task_status_map.get(dep_id) {
                let normalized_status = normalize_status_for_blocking(dep_status);
                if normalized_status != "Done" {
                    incomplete_dependencies.push(dep_id.clone());
                }
            } else {
                // Dependency task doesn't exist
                incomplete_dependencies.push(dep_id.clone());
            }
        }

        // If this task has incomplete dependencies, record blocking relationship
        if !incomplete_dependencies.is_empty() && task.status != "done" && task.status != "Done" {
            blocking.push(BlockingRelationship {
                blocked_task: task.id.clone(),
                blocking_tasks: incomplete_dependencies.clone(),
                reason: format!(
                    "Task '{}' is blocked by {} incomplete dependencies",
                    task.id,
                    incomplete_dependencies.len()
                ),
            });
        }
    }

    TaskGraph { nodes, edges, blocking_relationships: blocking }
}

/// Normalize status for blocking check.
fn normalize_status_for_blocking(status: &str) -> String {
    let key = status.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    match key.as_str() {
        "todo" | "open" => "Todo".to_string(),
        "inprogress" | "in_progress" => "InProgress".to_string(),
        "review" => "Review".to_string(),
        "done" | "closed" => "Done".to_string(),
        _ => "Todo".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tasks() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
tasks:
  - id: "TASK-001"
    title: "Test Task"
    requirement: "REQ-001"
    acs: ["AC-001"]
    status: "Todo"
    owner: "alice"
    labels: ["test"]
    summary: "A test task"
    recommended_flows: []
    depends_on: []
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let tasks = load_tasks(temp.path()).unwrap();
        assert_eq!(tasks.tasks.len(), 1);
        assert_eq!(tasks.tasks[0].id, "TASK-001");
    }

    #[test]
    fn test_validate_task_references() {
        let tasks = TasksSpec {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            tasks: vec![
                Task {
                    id: "TASK-VALID".to_string(),
                    title: "Valid task".to_string(),
                    requirement: "REQ-001".to_string(),
                    acs: vec!["AC-001".to_string()],
                    status: "Todo".to_string(),
                    owner: None,
                    labels: vec![],
                    docs: None,
                    summary: "Valid".to_string(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
                Task {
                    id: "TASK-INVALID".to_string(),
                    title: "Invalid task".to_string(),
                    requirement: "REQ-NONEXISTENT".to_string(),
                    acs: vec!["AC-NONEXISTENT".to_string()],
                    status: "Todo".to_string(),
                    owner: None,
                    labels: vec![],
                    docs: None,
                    summary: "Invalid".to_string(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
            ],
        };

        let ledger = SpecLedger {
            metadata: spec_ledger::Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![spec_ledger::Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![spec_ledger::Requirement {
                    id: "REQ-001".to_string(),
                    title: "Test".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![spec_ledger::AcceptanceCriterion {
                        id: "AC-001".to_string(),
                        text: "Test".to_string(),
                        tests: vec![],
                    }],
                }],
            }],
        };

        let warnings = validate_task_references(&tasks, &ledger).unwrap();
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.invalid_id == "REQ-NONEXISTENT"));
        assert!(warnings.iter().any(|w| w.invalid_id == "AC-NONEXISTENT"));
    }

    #[test]
    fn test_build_task_graph() {
        let tasks = TasksSpec {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            tasks: vec![
                Task {
                    id: "TASK-001".to_string(),
                    title: "First task".to_string(),
                    requirement: "REQ-001".to_string(),
                    acs: vec![],
                    status: "Todo".to_string(),
                    owner: None,
                    labels: vec![],
                    docs: None,
                    summary: "First".to_string(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
                Task {
                    id: "TASK-002".to_string(),
                    title: "Second task".to_string(),
                    requirement: "REQ-001".to_string(),
                    acs: vec![],
                    status: "Todo".to_string(),
                    owner: None,
                    labels: vec![],
                    docs: None,
                    summary: "Second".to_string(),
                    recommended_flows: vec![],
                    depends_on: vec!["TASK-001".to_string()],
                },
            ],
        };

        let graph = build_task_graph(&tasks);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.blocking_relationships.len(), 1);
        assert_eq!(graph.blocking_relationships[0].blocked_task, "TASK-002");
    }
}
