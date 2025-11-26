use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct TasksSpec {
    pub schema_version: String,
    pub template_version: String,
    pub tasks: Vec<Task>,
}

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
    pub summary: String,                // Added for suggest_next
    pub recommended_flows: Vec<String>, // Added for suggest_next
    #[serde(default)]
    pub depends_on: Vec<String>, // Task IDs this task depends on
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedSequence {
    pub task: String,
    pub goal: String,
    pub recommended_flows: Vec<String>,
    pub recommended_sequence: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Satisfied,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Command { cmd: String, description: String, status: StepStatus },
    Edit { file: String, hint: String, status: StepStatus },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskDocs {
    pub design: Vec<String>,
    pub plan: Vec<String>,
}

pub fn load_tasks(path: &Path) -> Result<TasksSpec> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read tasks file: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse tasks file: {}", path.display()))
}

pub fn suggest_next(
    root: &Path,
    task_id: &str,
    tasks_spec: &TasksSpec,
    devex_spec: &crate::devex::DevExFlows,
    ledger: &crate::SpecLedger,
) -> Result<SuggestedSequence> {
    let task = tasks_spec
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .context(format!("Task not found: {}", task_id))?;

    let mut sequence = Vec::new();

    for flow_name in &task.recommended_flows {
        if let Some(flow) = devex_spec.flows.get(flow_name) {
            for step in &flow.steps {
                // Determine status
                let mut status = StepStatus::Pending;

                match step.as_str() {
                    "ac-new" => {
                        // Check if any AC in the task exists in the ledger
                        let ac_exists = task.acs.iter().any(|ac_id| {
                            ledger.stories.iter().any(|s| {
                                s.requirements
                                    .iter()
                                    .any(|r| r.acceptance_criteria.iter().any(|a| a.id == *ac_id))
                            })
                        });
                        if ac_exists {
                            status = StepStatus::Satisfied;
                        }

                        sequence.push(Action::Command {
                            cmd: "cargo xtask ac-new AC-XXX-NNN \"Description\" --requirement REQ-..."
                                .to_string(),
                            description: "Scaffold the AC in the ledger and feature file".to_string(),
                            status,
                        });
                    }
                    "bundle" => {
                        let bundle_path = root.join(".llm/bundle").join(format!("{}.md", task_id));
                        if bundle_path.exists() {
                            status = StepStatus::Satisfied;
                        }

                        sequence.push(Action::Command {
                            cmd: format!("cargo xtask bundle {}", task_id),
                            description: "Generate LLM context bundle".to_string(),
                            status,
                        });
                    }
                    "bdd" => {
                        // Hard to know if passed without parsing junit, but we can check if junit exists
                        // For now, let's assume Pending unless we want to parse junit
                        // A simple heuristic: if feature file exists, maybe it's partially done?
                        // Let's keep it Pending to encourage running it.
                        sequence.push(Action::Command {
                            cmd: "cargo xtask bdd".to_string(),
                            description: "Run BDD acceptance tests".to_string(),
                            status: StepStatus::Pending,
                        });
                    }
                    "selftest" => {
                        sequence.push(Action::Command {
                            cmd: "cargo xtask selftest".to_string(),
                            description: "Run full platform verification".to_string(),
                            status: StepStatus::Pending,
                        });
                    }
                    "audit" => {
                        sequence.push(Action::Command {
                            cmd: "cargo xtask audit".to_string(),
                            description: "Run security and license audit".to_string(),
                            status: StepStatus::Pending,
                        });
                    }
                    "sbom-local" => {
                        let sbom_path = root.join("target").join("sbom.spdx.json");
                        if sbom_path.exists() {
                            status = StepStatus::Satisfied;
                        }
                        sequence.push(Action::Command {
                            cmd: "cargo xtask sbom-local".to_string(),
                            description: "Generate local SPDX SBOM".to_string(),
                            status,
                        });
                    }
                    "release-prepare" => {
                        sequence.push(Action::Command {
                            cmd: "cargo xtask release-prepare X.Y.Z".to_string(),
                            description: "Bump versions and seed changelog".to_string(),
                            status: StepStatus::Pending,
                        });
                    }
                    "release-verify" => {
                        sequence.push(Action::Command {
                            cmd: "cargo xtask release-verify".to_string(),
                            description: "Verify release readiness".to_string(),
                            status: StepStatus::Pending,
                        });
                    }
                    _ => {
                        // Generic fallback for commands
                        sequence.push(Action::Command {
                            cmd: format!("cargo xtask {}", step),
                            description: format!("Run {} command", step),
                            status: StepStatus::Pending,
                        });
                    }
                }
            }

            // Add specific edit hints based on flow
            if flow_name == "ac_first" {
                // Check if ACs are in ledger (re-use logic)
                let ac_exists = task.acs.iter().any(|ac_id| {
                    ledger.stories.iter().any(|s| {
                        s.requirements
                            .iter()
                            .any(|r| r.acceptance_criteria.iter().any(|a| a.id == *ac_id))
                    })
                });

                sequence.insert(
                    1,
                    Action::Edit {
                        file: "specs/spec_ledger.yaml".to_string(),
                        hint: "Insert the AC snippet under the requirement".to_string(),
                        status: if ac_exists { StepStatus::Satisfied } else { StepStatus::Pending },
                    },
                );

                // Check if feature file exists
                // We don't know the exact feature file name easily without parsing,
                // but we can check if ANY feature file contains the AC ID?
                // For now, let's just assume Pending for feature file edit if we can't easily check.
                // Actually, if AC exists in ledger, likely the user has edited the ledger.
                // But feature file?
                sequence.insert(
                    2,
                    Action::Edit {
                        file: "specs/features/*.feature".to_string(),
                        hint: "Add a BDD scenario tagged with the AC ID".to_string(),
                        status: StepStatus::Pending,
                    },
                );
            }
        }
    }

    Ok(SuggestedSequence {
        task: task.id.clone(),
        goal: task.summary.clone(),
        recommended_flows: task.recommended_flows.clone(),
        recommended_sequence: sequence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devex::{CommandSpec, DevExFlows, DocsRequirement, FlowSpec};
    use crate::ledger::{Metadata, SpecLedger};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn sbom_step_reflects_existing_artifact() {
        let temp_dir = tempdir().expect("temp directory is created");
        let root = temp_dir.path();

        let tasks_spec = TasksSpec {
            schema_version: "1".to_string(),
            template_version: "1".to_string(),
            tasks: vec![Task {
                id: "TASK-SBOM-001".to_string(),
                title: "Generate SBOM".to_string(),
                requirement: "REQ-1".to_string(),
                acs: vec![],
                status: "Todo".to_string(),
                owner: None,
                labels: vec![],
                docs: None,
                summary: "Ensure SBOM exists".to_string(),
                recommended_flows: vec!["release".to_string()],
                depends_on: vec![],
            }],
        };

        let mut commands = HashMap::new();
        commands.insert(
            "sbom-local".to_string(),
            CommandSpec {
                category: "release".to_string(),
                summary: "Generate SBOM".to_string(),
                required: true,
                docs: DocsRequirement::default(),
            },
        );

        let mut flows = HashMap::new();
        flows.insert(
            "release".to_string(),
            FlowSpec {
                name: "release".to_string(),
                description: "Release flow".to_string(),
                required: true,
                documented_in: vec!["docs".to_string()],
                steps: vec!["sbom-local".to_string()],
            },
        );

        let devex_spec = DevExFlows {
            schema_version: "1".to_string(),
            template_version: "1".to_string(),
            commands,
            flows,
        };

        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1".to_string(),
                template_version: "1".to_string(),
                last_updated: "today".to_string(),
                description: "test ledger".to_string(),
            },
            stories: vec![],
        };

        let sbom_path = root.join("target").join("sbom.spdx.json");
        std::fs::create_dir_all(sbom_path.parent().unwrap()).expect("target directory is created");
        std::fs::write(&sbom_path, "{}").expect("sbom artifact is written");

        let sequence =
            suggest_next(root, "TASK-SBOM-001", &tasks_spec, &devex_spec, &ledger).unwrap();

        let sbom_step_status = sequence
            .recommended_sequence
            .iter()
            .find_map(|action| match action {
                Action::Command { cmd, status, .. } if cmd == "cargo xtask sbom-local" => {
                    Some(status)
                }
                _ => None,
            })
            .expect("sbom-local step is present");

        assert_eq!(*sbom_step_status, StepStatus::Satisfied);
    }
}

/// Task graph structures for dependency visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskGraph {
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
    pub blocking_relationships: Vec<BlockingRelationship>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    pub status: String,
    pub requirement: String,
    pub owner: Option<String>,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,      // Task that depends
    pub to: String,        // Task being depended on
    pub edge_type: String, // "depends_on"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockingRelationship {
    pub blocked_task: String,
    pub blocking_tasks: Vec<String>,
    pub reason: String,
}

/// Build a task dependency graph from tasks spec
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
                // Dependency task doesn't exist - that's a problem
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

/// Generate Mermaid diagram for task graph
pub fn generate_mermaid_diagram(graph: &TaskGraph) -> String {
    let mut output = String::new();
    output.push_str("graph TD\n");

    // Add nodes with status coloring
    for node in &graph.nodes {
        let label = format!("{}[{}]", node.id, escape_mermaid_label(&node.title));
        output.push_str(&format!("  {}:::{}\n", label, status_to_class(&node.status)));
    }

    // Add edges
    for edge in &graph.edges {
        output.push_str(&format!("  {} -->|depends on| {}\n", edge.from, edge.to));
    }

    // Add style classes
    output.push_str("\n  classDef done fill:#90EE90,stroke:#333,stroke-width:2px\n");
    output.push_str("  classDef inprogress fill:#FFD700,stroke:#333,stroke-width:2px\n");
    output.push_str("  classDef review fill:#87CEEB,stroke:#333,stroke-width:2px\n");
    output.push_str("  classDef todo fill:#FFB6C1,stroke:#333,stroke-width:2px\n");

    output
}

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

fn status_to_class(status: &str) -> String {
    match normalize_status_for_blocking(status).as_str() {
        "Done" => "done",
        "InProgress" => "inprogress",
        "Review" => "review",
        _ => "todo",
    }
    .to_string()
}

fn escape_mermaid_label(label: &str) -> String {
    label
        .replace('"', "'")
        .replace('[', "(")
        .replace(']', ")")
        .chars()
        .take(50) // Limit label length
        .collect()
}
