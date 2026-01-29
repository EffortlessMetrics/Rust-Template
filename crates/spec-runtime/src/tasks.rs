//! Task management and sequencing.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Root structure for `specs/tasks.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TasksSpec {
    /// Schema version for tasks.yaml.
    pub schema_version: String,
    /// Template version this tasks.yaml is compatible with.
    pub template_version: String,
    /// List of all tasks defined in the spec.
    pub tasks: Vec<Task>,
}

/// A task definition with metadata and requirements.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    /// Unique task ID (e.g., "TASK-001").
    pub id: String,
    /// Human-readable title of the task.
    pub title: String,
    /// Brief summary of the task.
    pub summary: String,
    /// Requirement ID this task satisfies.
    pub requirement: String,
    /// List of AC IDs this task implements.
    pub acs: Vec<String>,
    /// Default status for the task.
    pub status: String,
    /// Optional owner of the task.
    pub owner: Option<String>,
    /// List of labels for the task.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Optional documentation related to the task.
    pub docs: Option<TaskDocs>,
    /// List of recommended DevEx flows for this task.
    #[serde(default)]
    pub recommended_flows: Vec<String>,
    /// List of dependencies (task IDs that must be completed first).
    #[serde(default, rename = "dependencies")]
    pub depends_on: Vec<String>,
}

/// Documentation associated with a task.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskDocs {
    /// List of design documents.
    #[serde(default)]
    pub design: Vec<String>,
    /// List of implementation plans.
    #[serde(default)]
    pub plan: Vec<String>,
}

/// Load the tasks specification from a YAML file.
pub fn load_tasks(path: &Path) -> Result<TasksSpec> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SpecError::io(path.to_path_buf(), e))?;

    serde_yaml::from_str(&content).map_err(SpecError::Yaml)
}

/// Suggested next task sequence.
#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedSequence {
    /// The task ID this suggestion is for.
    pub task: String,
    /// The high-level goal of the suggested sequence.
    pub goal: String,
    /// List of recommended DevEx flows.
    pub recommended_flows: Vec<String>,
    /// The sequence of suggested actions.
    pub recommended_sequence: Vec<String>,
}

/// Task dependency graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskGraph {
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
}

/// Suggest next tasks based on current task and dependencies.
pub fn suggest_next(
    _root: &Path,
    task_id: &str,
    spec: &TasksSpec,
    _devex: &crate::devex::DevExFlows,
    _ledger: &crate::ledger::SpecLedger,
) -> Result<SuggestedSequence> {
    // Basic implementation for now
    let next_tasks = spec
        .tasks
        .iter()
        .filter(|t| t.depends_on.contains(&task_id.to_string()))
        .map(|t| t.id.clone())
        .collect();

    Ok(SuggestedSequence {
        task: task_id.to_string(),
        goal: format!("Complete task {}", task_id),
        recommended_flows: Vec::new(),
        recommended_sequence: next_tasks,
    })
}

/// Build task dependency graph.
pub fn build_task_graph(spec: &TasksSpec) -> TaskGraph {
    let nodes = spec
        .tasks
        .iter()
        .map(|t| TaskNode { id: t.id.clone(), title: t.title.clone(), status: t.status.clone() })
        .collect();

    let mut edges = Vec::new();
    for task in &spec.tasks {
        for dep in &task.depends_on {
            edges.push(TaskEdge { from: dep.clone(), to: task.id.clone() });
        }
    }

    TaskGraph { nodes, edges }
}

/// Generate Mermaid diagram from task graph.
pub fn generate_mermaid_diagram(graph: &TaskGraph) -> String {
    let mut mermaid = String::from("graph TD\n");
    for edge in &graph.edges {
        mermaid.push_str(&format!("    {} --> {}\n", edge.from, edge.to));
    }
    mermaid
}
