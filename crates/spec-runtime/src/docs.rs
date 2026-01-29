//! Documentation inventory and metadata specification.
//!
//! This module defines the structure of `doc_index.yaml`, which tracks
//! all documentation files and their relationships to governance artifacts.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Root structure for `specs/doc_index.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocIndex {
    /// Schema version for doc_index.yaml.
    pub schema_version: String,
    /// Template version this index is compatible with.
    pub template_version: String,
    /// List of all documentation entries.
    pub docs: Vec<DocEntry>,
}

/// An entry for a single documentation file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocEntry {
    /// Unique documentation ID (e.g., "DOC-TPL-CONFIG").
    pub id: String,
    /// Path to the markdown file relative to the workspace root.
    pub file: String,
    /// Type of document (e.g., "how-to", "explanation", "design_doc").
    pub doc_type: String,
    /// User stories this document relates to.
    #[serde(default)]
    pub stories: Vec<String>,
    /// Requirements this document relates to.
    #[serde(default)]
    pub requirements: Vec<String>,
    /// Acceptance criteria this document relates to.
    #[serde(default)]
    pub acs: Vec<String>,
    /// ADRs (Architectural Decision Records) this document relates to.
    #[serde(default)]
    pub adrs: Vec<String>,
}

/// Load the documentation index from a YAML file.
pub fn load_doc_index(path: &Path) -> Result<DocIndex> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SpecError::io(path.to_path_buf(), e))?;

    serde_yaml::from_str(&content).map_err(SpecError::Yaml)
}

/// Load the documentation index using a repository context.
pub fn load_doc_index_with_context(ctx: &gov_model::RepoContext) -> Result<DocIndex> {
    load_doc_index(&ctx.doc_index_path())
}
