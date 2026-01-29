//! Service metadata specification.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root structure for `specs/service_metadata.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceMetadata {
    /// Unique service identifier.
    pub service_id: String,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Brief description of the service.
    pub description: Option<String>,
    /// Template version this service is built on.
    pub template_version: Option<String>,
    /// Metadata tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Map of relevant links (e.g., repository, documentation, support).
    #[serde(default)]
    pub links: HashMap<String, String>,
}

/// Load service metadata from a YAML file.
pub fn load_service_metadata(path: &Path) -> Result<ServiceMetadata> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SpecError::io(path.to_path_buf(), e))?;

    serde_yaml::from_str(&content).map_err(SpecError::Yaml)
}
