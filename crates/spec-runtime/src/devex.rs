//! DevEx flows and commands specification.
//!
//! This module defines the structure of `devex_flows.yaml`, which describes
//! the developer experience flows and the CLI commands that compose them.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root structure for `specs/devex_flows.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevExFlows {
    /// Schema version for devex_flows.yaml.
    pub schema_version: String,
    /// Template version this spec is compatible with.
    pub template_version: String,
    /// Registry of available CLI commands.
    pub commands: HashMap<String, CommandSpec>,
    /// Registry of developer experience flows.
    pub flows: HashMap<String, FlowSpec>,
}

/// A CLI command that can be executed as part of a flow.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandSpec {
    /// Command category (e.g., "onboarding", "design_ac", "security").
    pub category: String,
    /// Brief summary of what the command does.
    pub summary: String,
    /// Whether this command is mandatory.
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

/// A developer experience flow consisting of multiple steps.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FlowSpec {
    /// Human-readable name of the flow.
    pub name: String,
    /// Brief description of the flow's purpose.
    pub description: String,
    /// Whether this flow is required for the template.
    pub required: bool,
    /// List of documents where this flow is described.
    pub documented_in: Vec<String>,
    /// Sequential list of command keys to execute.
    pub steps: Vec<String>,
}

/// Load DevEx flows from a YAML file.
pub fn load_devex_flows(path: &Path) -> Result<DevExFlows> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SpecError::io(path.to_path_buf(), e))?;

    serde_yaml::from_str(&content).map_err(SpecError::Yaml)
}

/// Load DevEx flows using a repository context.
pub fn load_devex_flows_with_context(ctx: &gov_model::RepoContext) -> Result<DevExFlows> {
    load_devex_flows(&ctx.devex_flows_path())
}
