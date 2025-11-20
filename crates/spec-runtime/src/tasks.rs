use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedSequence {
    pub task: String,
    pub goal: String,
    pub recommended_flows: Vec<String>,
    pub recommended_sequence: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Command { cmd: String, description: String },
    Edit { file: String, hint: String },
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
    task_id: &str,
    tasks_spec: &TasksSpec,
    devex_spec: &crate::devex::DevExFlows,
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
                // Map step to action
                match step.as_str() {
                    "ac-new" => sequence.push(Action::Command {
                        cmd: "cargo xtask ac-new AC-XXX-NNN \"Description\" --requirement REQ-..."
                            .to_string(),
                        description: "Scaffold the AC in the ledger and feature file".to_string(),
                    }),
                    "bundle" => sequence.push(Action::Command {
                        cmd: format!("cargo xtask bundle {}", task_id),
                        description: "Generate LLM context bundle".to_string(),
                    }),
                    "bdd" => sequence.push(Action::Command {
                        cmd: "cargo xtask bdd".to_string(),
                        description: "Run BDD acceptance tests".to_string(),
                    }),
                    "selftest" => sequence.push(Action::Command {
                        cmd: "cargo xtask selftest".to_string(),
                        description: "Run full platform verification".to_string(),
                    }),
                    "audit" => sequence.push(Action::Command {
                        cmd: "cargo xtask audit".to_string(),
                        description: "Run security and license audit".to_string(),
                    }),
                    "sbom-local" => sequence.push(Action::Command {
                        cmd: "cargo xtask sbom-local".to_string(),
                        description: "Generate local SPDX SBOM".to_string(),
                    }),
                    "release-prepare" => sequence.push(Action::Command {
                        cmd: "cargo xtask release-prepare X.Y.Z".to_string(),
                        description: "Bump versions and seed changelog".to_string(),
                    }),
                    "release-verify" => sequence.push(Action::Command {
                        cmd: "cargo xtask release-verify".to_string(),
                        description: "Verify release readiness".to_string(),
                    }),
                    _ => {
                        // Generic fallback for commands
                        sequence.push(Action::Command {
                            cmd: format!("cargo xtask {}", step),
                            description: format!("Run {} command", step),
                        });
                    }
                }
            }

            // Add specific edit hints based on flow
            if flow_name == "ac_first" {
                sequence.insert(
                    1,
                    Action::Edit {
                        file: "specs/spec_ledger.yaml".to_string(),
                        hint: "Insert the AC snippet under the requirement".to_string(),
                    },
                );
                sequence.insert(
                    2,
                    Action::Edit {
                        file: "specs/features/*.feature".to_string(),
                        hint: "Add a BDD scenario tagged with the AC ID".to_string(),
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
