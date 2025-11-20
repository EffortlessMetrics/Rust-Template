use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Documentation policy specification loaded from specs/doc_policies.yaml
#[derive(Debug, Deserialize, Serialize)]
pub struct DocPolicies {
    pub schema_version: String,
    pub template_version: String,
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub applies_to: AppliesTo,
    pub require_doc_types: Vec<String>,
    pub min_docs: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppliesTo {
    pub requirement_tags: Vec<String>,
}

pub fn load_policies(path: &Path) -> Result<DocPolicies> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read doc policies: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse doc policies: {}", path.display()))
}
