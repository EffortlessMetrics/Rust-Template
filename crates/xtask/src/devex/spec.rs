use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// DevEx specification loaded from specs/devex_flows.yaml
#[derive(Debug, Deserialize, Serialize)]
pub struct DevExSpec {
    pub schema_version: String,
    pub template_version: String,
    pub commands: HashMap<String, CommandSpec>,
    pub flows: HashMap<String, FlowSpec>,
}

/// Specification for a single xtask command
#[derive(Debug, Deserialize, Serialize)]
pub struct CommandSpec {
    pub category: String,
    pub summary: String,
    pub required: bool,
    #[serde(default)]
    pub docs: DocsRequirement,
}

/// Documentation requirements for a command
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DocsRequirement {
    #[serde(default)]
    pub readme_table: bool,
    #[serde(default)]
    pub contributing_flow: bool,
    #[serde(default)]
    pub claude_golden_path: bool,
}

/// Specification for a canonical workflow
#[derive(Debug, Deserialize, Serialize)]
pub struct FlowSpec {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub documented_in: Vec<String>,
    pub steps: Vec<String>,
}

/// Load DevEx specification from YAML file
pub fn load_spec(path: &Path) -> Result<DevExSpec> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read devex spec: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse devex spec: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn devex_spec_parses() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let spec_path = root.join("specs/devex_flows.yaml");

        let spec = load_spec(&spec_path).expect("devex_flows.yaml should parse");

        // Validate schema
        assert_eq!(spec.schema_version, "1.0");
        assert!(!spec.template_version.is_empty());

        // Validate required commands exist
        assert!(spec.commands.contains_key("doctor"));
        assert!(spec.commands.contains_key("check"));
        assert!(spec.commands.contains_key("selftest"));
        assert!(spec.commands.contains_key("audit"));
        assert!(spec.commands.contains_key("help-flows"));

        // Validate required flows exist
        assert!(spec.flows.contains_key("onboarding"));
        assert!(spec.flows.contains_key("ac_first"));
        assert!(spec.flows.contains_key("release"));

        // Validate doctor is marked as required
        let doctor = spec.commands.get("doctor").unwrap();
        assert!(doctor.required);
        assert_eq!(doctor.category, "onboarding");
    }
}
