use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Doc index specification loaded from specs/doc_index.yaml
#[derive(Debug, Deserialize, Serialize)]
pub struct DocIndex {
    pub schema_version: String,
    pub template_version: String,
    pub docs: Vec<DocEntry>,
}

/// Single documentation entry in the index
#[derive(Debug, Deserialize, Serialize)]
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

/// Load doc index from YAML file
pub fn load_doc_index(path: &Path) -> Result<DocIndex> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read doc index: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse doc index: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn doc_index_parses() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let index_path = root.join("specs/doc_index.yaml");

        let index = load_doc_index(&index_path).expect("doc_index.yaml should parse");

        // Validate schema
        assert_eq!(index.schema_version, "1.0");
        assert!(!index.template_version.is_empty());

        // Validate at least one entry exists
        assert!(!index.docs.is_empty());

        // Validate first entry has required fields
        let first = &index.docs[0];
        assert!(!first.id.is_empty());
        assert!(!first.file.is_empty());
        assert!(!first.doc_type.is_empty());
    }
}
