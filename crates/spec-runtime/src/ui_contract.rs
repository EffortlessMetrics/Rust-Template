//! UI contract for governed platform UI surfaces.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root structure for `specs/ui_contract.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiContract {
    /// Schema version for ui_contract.yaml.
    pub schema_version: String,
    /// Template version this contract is compatible with.
    pub template_version: String,
    /// List of UI screens defined in the contract.
    pub screens: Vec<Screen>,
    /// Map of region kinds to their semantic descriptions.
    pub region_kinds: HashMap<String, String>,
}

/// A UI screen with regions and stable identifiers.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Screen {
    /// Unique screen identifier.
    pub id: String,
    /// Canonical route for the screen.
    pub route: String,
    /// Optional aliases for the route.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Brief description of the screen's purpose.
    pub description: String,
    /// List of governed regions within the screen.
    pub regions: Vec<Region>,
}

/// A region within a screen containing governed UI elements.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Region {
    /// Unique region ID (data-uiid).
    pub id: String,
    /// Semantic kind of the region.
    pub kind: String,
    /// Description of the region's content or purpose.
    pub description: String,
}

/// Load the UI contract from a YAML file.
pub fn load_ui_contract(path: &Path) -> Result<UiContract> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SpecError::io(path.to_path_buf(), e))?;

    serde_yaml::from_str(&content).map_err(SpecError::Yaml)
}

/// Load the UI contract using a repository context.
pub fn load_ui_contract_with_context(ctx: &gov_model::RepoContext) -> Result<UiContract> {
    load_ui_contract(&ctx.ui_contract_path())
}
