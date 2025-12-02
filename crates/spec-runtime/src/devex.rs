//! Developer experience flows and xtask command specifications.
//!
//! This module defines the structure of `devex_flows.yaml`, which describes
//! developer workflows, xtask commands, and their documentation requirements.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Developer experience flows specification.
///
/// Root structure for `specs/devex_flows.yaml`, containing command and flow definitions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevExFlows {
    /// Schema version for devex_flows.yaml.
    pub schema_version: String,
    /// Template version this spec is compatible with.
    pub template_version: String,
    /// Map of command IDs to command specifications (e.g., "check", "selftest").
    pub commands: HashMap<String, CommandSpec>,
    /// Map of flow IDs to flow specifications (e.g., "onboarding", "feature-dev").
    pub flows: HashMap<String, FlowSpec>,
}

/// Specification for an individual xtask command.
///
/// Describes a single command's category, summary, required status, and documentation needs.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandSpec {
    /// Command category (e.g., "onboarding", "design_ac", "security").
    pub category: String,
    /// Brief summary of what the command does.
    pub summary: String,
    /// Whether this command is required for the template to function.
    pub required: bool,
    /// Documentation requirements for this command.
    #[serde(default)]
    pub docs: DocsRequirement,
}

/// Documentation requirements for a command.
///
/// Specifies which documentation artifacts should include this command.
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
///
/// Describes a multi-step workflow composed of commands.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlowSpec {
    /// Human-readable name of the flow.
    pub name: String,
    /// Description of what this flow accomplishes.
    pub description: String,
    /// Whether this flow is required for the template.
    pub required: bool,
    /// List of documents where this flow is described.
    pub documented_in: Vec<String>,
    /// Ordered list of command IDs that comprise this flow.
    pub steps: Vec<String>,
}

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
/// Returns an error if the file is missing, unreadable, or malformed YAML.
///
/// # Example
///
/// ```ignore
/// let flows = load_devex_flows(Path::new("specs/devex_flows.yaml"))?;
/// println!("Loaded {} commands", flows.commands.len());
/// ```
pub fn load_devex_flows(path: &Path) -> Result<DevExFlows> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read devex flows: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse devex flows: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// AC-PLT-014: Validates that specs/devex_flows.yaml exists and contains valid flow definitions
    ///
    /// This test ensures:
    /// 1. The devex_flows.yaml file exists at the expected location
    /// 2. The file can be parsed as valid YAML
    /// 3. All required top-level fields are present (schema_version, template_version, commands, flows)
    /// 4. Each command has required fields (category, summary, required)
    /// 5. Each flow has required fields (name, description, steps, documented_in)
    /// 6. Flow steps reference valid commands (referential integrity)
    /// 7. Expected command categories exist
    #[test]
    fn devex_flows_schema_valid() {
        // 1. File exists: specs/devex_flows.yaml
        let devex_flows_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("specs/devex_flows.yaml");

        assert!(devex_flows_path.exists(), "devex_flows.yaml must exist at specs/devex_flows.yaml");

        // 2. Load and parse the file
        let flows = load_devex_flows(&devex_flows_path).expect("Failed to load devex_flows.yaml");

        // 3. Schema structure validation: schema_version, template_version, commands, flows
        assert!(!flows.schema_version.is_empty(), "schema_version must be present and non-empty");
        assert!(
            !flows.template_version.is_empty(),
            "template_version must be present and non-empty"
        );
        assert!(!flows.commands.is_empty(), "commands section must contain at least one command");
        assert!(!flows.flows.is_empty(), "flows section must contain at least one flow");

        // 4. Command structure validation: category, summary, required, docs
        for (cmd_id, cmd_spec) in &flows.commands {
            assert!(
                !cmd_spec.category.is_empty(),
                "Command '{}' must have a non-empty category",
                cmd_id
            );
            assert!(
                !cmd_spec.summary.is_empty(),
                "Command '{}' must have a non-empty summary",
                cmd_id
            );
            // required is a bool, always valid
            // docs has defaults, so it's always present
        }

        // 5. Flow structure validation: name, description, steps, documented_in
        for (flow_id, flow_spec) in &flows.flows {
            assert!(!flow_spec.name.is_empty(), "Flow '{}' must have a non-empty name", flow_id);
            assert!(
                !flow_spec.description.is_empty(),
                "Flow '{}' must have a non-empty description",
                flow_id
            );
            assert!(!flow_spec.steps.is_empty(), "Flow '{}' must have at least one step", flow_id);
            assert!(
                !flow_spec.documented_in.is_empty(),
                "Flow '{}' must specify at least one document in documented_in",
                flow_id
            );
        }

        // 6. Referential integrity: flow steps reference valid commands
        let valid_commands: HashSet<_> = flows.commands.keys().cloned().collect();

        for (flow_id, flow_spec) in &flows.flows {
            for step in &flow_spec.steps {
                assert!(
                    valid_commands.contains(step),
                    "Flow '{}' references undefined command '{}'. Valid commands: {:?}",
                    flow_id,
                    step,
                    valid_commands
                );
            }
        }

        // 7. Additional validation: Verify expected structure from the spec
        // Check that we have the expected categories
        let categories: HashSet<_> =
            flows.commands.values().map(|cmd| cmd.category.as_str()).collect();

        // At least some core categories should exist
        let expected_categories = ["onboarding", "design_ac", "security", "release"];
        for cat in expected_categories {
            assert!(categories.contains(cat), "Expected category '{}' not found in commands", cat);
        }
    }
}
