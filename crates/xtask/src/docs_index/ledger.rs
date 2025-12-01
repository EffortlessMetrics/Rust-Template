use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Minimal Spec Ledger structure for policy validation
#[derive(Debug, Deserialize)]
pub struct Ledger {
    pub stories: Vec<Story>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub must_have_ac: bool,
}

pub fn load_ledger(path: &Path) -> Result<Ledger> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read ledger: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger: {}", path.display()))
}
