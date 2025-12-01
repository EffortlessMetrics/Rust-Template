//! Task definition loader for filesystem-based governance.
//!
//! This module loads strongly-typed task definitions from a YAML file
//! (typically `specs/tasks.yaml`) and exposes them as `TaskDefinition`
//! values keyed by task id.

use serde::Deserialize;
use std::collections::HashMap;

/// Top-level representation of the tasks definition file.
///
/// This mirrors the YAML structure used for `specs/tasks.yaml` and
/// is intended to be deserialized directly from disk.
#[derive(Debug, Deserialize, Clone)]
pub struct TasksFile {
    /// Schema version of the tasks file format (e.g. `"1.0"`).
    pub schema_version: String,
    /// Template/kernel version this tasks file was authored against.
    ///
    /// This is used for sanity checks when upgrading the kernel or
    /// comparing fork state.
    pub template_version: String,
    /// All task definitions declared in the file.
    pub tasks: Vec<TaskDefinition>,
}

/// Canonical definition of a single task from the spec tree.
///
/// This is the "read-only" view of a task: it describes what the task is,
/// which requirement and ACs it relates to, and how it should be presented.
#[derive(Debug, Deserialize, Clone)]
pub struct TaskDefinition {
    /// Unique task identifier (e.g. `TASK-HINT-001`).
    pub id: String,
    /// Human-readable title for the task.
    pub title: String,
    /// Requirement id this task is primarily associated with (e.g. `REQ-TPL-HEALTH`).
    pub requirement: String,
    /// Acceptance criteria this task helps satisfy.
    #[serde(default)]
    pub acs: Vec<String>,
    /// Optional current status snapshot from the spec (`Todo`, `InProgress`, `Done`, etc.).
    ///
    /// This is descriptive; the authoritative mutable status lives in `tasks_state.yaml`.
    #[serde(default)]
    pub status: Option<String>,
    /// Optional owner identifier (team or person).
    #[serde(default)]
    pub owner: Option<String>,
    /// Free-form labels for filtering and hints (e.g. `["security", "v3"]`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// Short summary or description of the task.
    #[serde(default)]
    pub summary: Option<String>,
    /// Recommended DevEx flows to use for this task (e.g. `["governed-feature-dev"]`).
    #[serde(default)]
    pub recommended_flows: Vec<String>,
    /// Linked documentation for the task (design docs, plans, runbooks).
    #[serde(default)]
    pub docs: TaskDocs,
}

/// Structured document references associated with a task.
///
/// These fields allow the governance UI and agents to surface the
/// right design and plan docs alongside the task.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TaskDocs {
    /// Design documents relevant to this task (paths or URLs).
    #[serde(default)]
    pub design: Vec<String>,
    /// Plan / implementation documents relevant to this task.
    #[serde(default)]
    pub plan: Vec<String>,
}

/// Load task definitions from a YAML file on disk.
///
/// `path` should point to a tasks definition file (usually `specs/tasks.yaml`).
/// On success this returns a map from task id to `TaskDefinition`.
///
/// The error string is suitable for displaying in CLI error messages and
/// propagating up as a `GovernanceError` cause.
pub fn load_tasks_definitions(
    path: &std::path::Path,
) -> Result<HashMap<String, TaskDefinition>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read tasks.yaml: {}", e))?;

    let tasks_file: TasksFile =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse tasks.yaml: {}", e))?;

    let mut map = HashMap::new();
    for task in tasks_file.tasks {
        map.insert(task.id.clone(), task);
    }

    Ok(map)
}
