use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevExFlows {
    pub schema_version: String,
    pub template_version: String,
    pub commands: HashMap<String, CommandSpec>,
    pub flows: HashMap<String, FlowSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandSpec {
    pub category: String,
    pub summary: String,
    pub required: bool,
    #[serde(default)]
    pub docs: DocsRequirement,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DocsRequirement {
    #[serde(default)]
    pub readme_table: bool,
    #[serde(default)]
    pub contributing_flow: bool,
    #[serde(default)]
    pub claude_golden_path: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlowSpec {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub documented_in: Vec<String>,
    pub steps: Vec<String>,
}

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
