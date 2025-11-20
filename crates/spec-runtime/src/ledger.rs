use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpecLedger {
    pub metadata: Metadata,
    pub stories: Vec<Story>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metadata {
    pub schema_version: String,
    pub template_version: String,
    pub last_updated: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub must_have_ac: bool,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub text: String,
}

pub fn load_spec_ledger(path: &Path) -> Result<SpecLedger> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read spec ledger: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse spec ledger: {}", path.display()))
}
