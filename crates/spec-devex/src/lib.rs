//! Developer experience flows and xtask command specifications.
//!
//! This crate provides types and functions for managing DevEx flows,
//! command resolution, and flow execution helpers.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, serde, serde_yaml, thiserror, anyhow
//! - **Workflow layer**: Provides flow types and command resolution
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//!
//! # Example
//!
//! ```ignore
//! use spec_devex::{load_devex_flows, resolve_command};
//!
//! let devex = load_devex_flows(Path::new("specs/devex_flows.yaml"))?;
//! let cmd = resolve_command(&devex, "check")?;
//! ```

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use spec_types::{SpecError, SpecResult};
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// Developer experience flows specification.
///
/// Root structure for `specs/devex_flows.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevExFlows {
    /// Schema version for devex_flows.yaml.
    pub schema_version: String,
    /// Template version this spec is compatible with.
    pub template_version: String,
    /// Map of command IDs to command specifications.
    pub commands: HashMap<String, CommandSpec>,
    /// Map of flow IDs to flow specifications.
    pub flows: HashMap<String, FlowSpec>,
}

/// Specification for an individual xtask command.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandSpec {
    /// Command category (e.g., "onboarding", "design_ac", "security").
    pub category: String,
    /// Brief summary of what command does.
    pub summary: String,
    /// Whether this command is required for template to function.
    pub required: bool,
    /// Documentation requirements for this command.
    #[serde(default)]
    pub docs: DocsRequirement,
}

/// Documentation requirements for a command.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DocsRequirement {
    /// Should appear in README command table.
    #[serde(default)]
    pub readme_table: bool,
    /// Should appear in CONTRIBUTING flow section.
    #[serde(default)]
    pub contributing_flow: bool,
    /// Should appear in CLAUDE.md golden path.
    #[serde(default)]
    pub claude_golden_path: bool,
}

/// Specification for a developer workflow.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlowSpec {
    /// Human-readable name of flow.
    pub name: String,
    /// Description of what this flow accomplishes.
    pub description: String,
    /// Whether this flow is required for template.
    pub required: bool,
    /// List of documents where this flow is described.
    pub documented_in: Vec<String>,
    /// Ordered list of command IDs that comprise this flow.
    pub steps: Vec<String>,
}

/// Flow execution result.
#[derive(Debug, Clone)]
pub struct FlowExecution {
    pub flow_id: String,
    pub steps: Vec<FlowStep>,
    pub total_steps: usize,
    pub completed_steps: usize,
}

/// A step in flow execution.
#[derive(Debug, Clone)]
pub struct FlowStep {
    pub command_id: String,
    pub command_spec: CommandSpec,
    pub status: StepStatus,
    pub output: Option<String>,
}

/// Status of a flow step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

// ============================================================================
// Loading
// ============================================================================

/// Load developer experience flows from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to `devex_flows.yaml`
///
/// # Returns
///
/// Returns a parsed [`DevExFlows`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_devex_flows(path: &Path) -> SpecResult<DevExFlows> {
    let content = std::fs::read_to_string(path).map_err(SpecError::Io)?;

    serde_yaml::from_str(&content).map_err(|e| SpecError::YamlParse(e.to_string()))
}

// ============================================================================
// Command Resolution
// ============================================================================

/// Resolve a command by ID.
///
/// Returns the command specification if found.
///
/// # Arguments
///
/// * `devex` - DevEx flows specification
/// * `command_id` - Command ID to resolve
///
/// # Returns
///
/// Returns [`CommandSpec`] if command exists, or error otherwise.
pub fn resolve_command(devex: &DevExFlows, command_id: &str) -> Result<CommandSpec, anyhow::Error> {
    devex
        .commands
        .get(command_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Command '{}' not found in devex_flows.yaml", command_id))
}

/// Get all commands by category.
///
/// Returns all commands matching the specified category.
///
/// # Arguments
///
/// * `devex` - DevEx flows specification
/// * `category` - Category to filter by
///
/// # Returns
///
/// Returns a list of command specifications for the category.
pub fn get_commands_by_category(devex: &DevExFlows, category: &str) -> Vec<CommandSpec> {
    devex.commands.values().filter(|cmd| cmd.category == category).cloned().collect()
}

/// Get all required commands.
///
/// Returns all commands marked as required.
///
/// # Arguments
///
/// * `devex` - DevEx flows specification
///
/// # Returns
///
/// Returns a list of required command specifications.
pub fn get_required_commands(devex: &DevExFlows) -> Vec<CommandSpec> {
    devex.commands.values().filter(|cmd| cmd.required).cloned().collect()
}

// ============================================================================
// Flow Execution
// ============================================================================

/// Execute a flow and return the result.
///
/// This is a simulation function that builds the flow structure
/// without actually running commands.
///
/// # Arguments
///
/// * `devex` - DevEx flows specification
/// * `flow_id` - Flow ID to execute
///
/// # Returns
///
/// Returns a [`FlowExecution`] with steps and status.
pub fn execute_flow(devex: &DevExFlows, flow_id: &str) -> Result<FlowExecution, anyhow::Error> {
    let flow =
        devex.flows.get(flow_id).ok_or_else(|| anyhow::anyhow!("Flow '{}' not found", flow_id))?;

    let steps: Vec<FlowStep> = flow
        .steps
        .iter()
        .map(|cmd_id| {
            let cmd_spec = resolve_command(devex, cmd_id).unwrap_or_else(|_| CommandSpec {
                category: "unknown".to_string(),
                summary: "Unknown command".to_string(),
                required: false,
                docs: DocsRequirement::default(),
            });

            FlowStep {
                command_id: cmd_id.clone(),
                command_spec: cmd_spec,
                status: StepStatus::Pending,
                output: None,
            }
        })
        .collect();

    let total_steps = steps.len();

    Ok(FlowExecution { flow_id: flow_id.to_string(), steps, total_steps, completed_steps: 0 })
}

/// Get all required flows.
///
/// Returns all flows marked as required.
///
/// # Arguments
///
/// * `devex` - DevEx flows specification
///
/// # Returns
///
/// Returns a list of required flow specifications.
pub fn get_required_flows(devex: &DevExFlows) -> Vec<FlowSpec> {
    devex.flows.values().filter(|flow| flow.required).cloned().collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_devex_flows() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
commands:
  check:
    category: "validation"
    summary: "Run checks"
    required: true
flows:
  validate:
    name: "Validate"
    description: "Run validation"
    required: true
    documented_in: ["docs"]
    steps: ["check"]
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let devex = load_devex_flows(temp.path()).unwrap();
        assert_eq!(devex.commands.len(), 1);
        assert_eq!(devex.flows.len(), 1);
        assert!(devex.commands.contains_key("check"));
    }

    #[test]
    fn test_resolve_command() {
        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            commands: {
                let mut map = HashMap::new();
                map.insert(
                    "check".to_string(),
                    CommandSpec {
                        category: "validation".to_string(),
                        summary: "Run checks".to_string(),
                        required: true,
                        docs: DocsRequirement::default(),
                    },
                );
                map
            },
            flows: HashMap::new(),
        };

        let cmd = resolve_command(&devex, "check").unwrap();
        assert_eq!(cmd.category, "validation");
        assert!(cmd.required);
    }

    #[test]
    fn test_get_commands_by_category() {
        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            commands: {
                let mut map = HashMap::new();
                map.insert(
                    "check".to_string(),
                    CommandSpec {
                        category: "validation".to_string(),
                        summary: "Run checks".to_string(),
                        required: true,
                        docs: DocsRequirement::default(),
                    },
                );
                map.insert(
                    "doctor".to_string(),
                    CommandSpec {
                        category: "onboarding".to_string(),
                        summary: "Run diagnostics".to_string(),
                        required: true,
                        docs: DocsRequirement::default(),
                    },
                );
                map
            },
            flows: HashMap::new(),
        };

        let validation_cmds = get_commands_by_category(&devex, "validation");
        assert_eq!(validation_cmds.len(), 1);
        assert_eq!(validation_cmds[0].category, "validation");
    }

    #[test]
    fn test_execute_flow() {
        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            commands: {
                let mut map = HashMap::new();
                map.insert(
                    "check".to_string(),
                    CommandSpec {
                        category: "validation".to_string(),
                        summary: "Run checks".to_string(),
                        required: true,
                        docs: DocsRequirement::default(),
                    },
                );
                map
            },
            flows: {
                let mut map = HashMap::new();
                map.insert(
                    "validate".to_string(),
                    FlowSpec {
                        name: "Validate".to_string(),
                        description: "Run validation".to_string(),
                        required: true,
                        documented_in: vec!["docs".to_string()],
                        steps: vec!["check".to_string()],
                    },
                );
                map
            },
        };

        let execution = execute_flow(&devex, "validate").unwrap();
        assert_eq!(execution.flow_id, "validate");
        assert_eq!(execution.total_steps, 1);
        assert_eq!(execution.steps.len(), 1);
        assert_eq!(execution.steps[0].command_id, "check");
    }
}
