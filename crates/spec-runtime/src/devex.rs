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
