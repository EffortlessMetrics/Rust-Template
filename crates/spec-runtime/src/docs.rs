use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocIndex {
    pub schema_version: String,
    pub template_version: String,
    pub docs: Vec<DocEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocEntry {
    pub id: String,
    pub file: String,
    pub doc_type: String,
    #[serde(default)]
    pub stories: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub acs: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
}

pub fn load_doc_index(path: &Path) -> Result<DocIndex> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read doc index: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse doc index: {}", path.display()))
}
