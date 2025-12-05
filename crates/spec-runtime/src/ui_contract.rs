//! UI Contract types for governed platform UI surfaces.
//!
//! This module provides types and loaders for the UI contract specification,
//! which defines the screens, regions, and stable identifiers (`data-uiid`)
//! that agents, tests, and consumers can rely on.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Top-level UI contract specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiContract {
    pub schema_version: String,
    pub template_version: String,
    pub screens: Vec<Screen>,
    #[serde(default)]
    pub region_kinds: HashMap<String, String>,
}

/// A UI screen definition.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Screen {
    /// Unique screen identifier (e.g., "platform_dashboard")
    pub id: String,
    /// Primary route (e.g., "/", "/ui/graph")
    pub route: String,
    /// Alternative routes that serve the same screen
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Human-readable description of the screen's purpose
    pub description: String,
    /// Regions (sections) within the screen
    pub regions: Vec<Region>,
}

/// A region within a UI screen.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Region {
    /// Unique region ID, used as `data-uiid` attribute (e.g., "dashboard.health")
    pub id: String,
    /// Semantic kind: header, navigation, panel, visualization, controls, table
    pub kind: String,
    /// Human-readable description of the region's content
    pub description: String,
}

/// Load UI contract from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to the UI contract YAML file
///
/// # Returns
///
/// Returns [`UiContract`] containing parsed screen and region definitions.
///
/// # Errors
///
/// Returns an error if the file is missing, malformed, or fails validation.
pub fn load_ui_contract(path: &Path) -> Result<UiContract> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read UI contract: {}", path.display()))?;

    let contract: UiContract = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse UI contract: {}", path.display()))?;

    // Validate uniqueness of screen IDs
    let mut seen_screens: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for screen in &contract.screens {
        if !seen_screens.insert(&screen.id) {
            anyhow::bail!("Duplicate screen ID: {}", screen.id);
        }

        // Validate uniqueness of region IDs within each screen
        let mut seen_regions: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for region in &screen.regions {
            if !seen_regions.insert(&region.id) {
                anyhow::bail!("Duplicate region ID '{}' in screen '{}'", region.id, screen.id);
            }
        }
    }

    Ok(contract)
}

/// Validate that all region kinds reference valid kind definitions.
pub fn validate_region_kinds(contract: &UiContract) -> Result<()> {
    for screen in &contract.screens {
        for region in &screen.regions {
            if !contract.region_kinds.is_empty()
                && !contract.region_kinds.contains_key(&region.kind)
            {
                anyhow::bail!(
                    "Region '{}' in screen '{}' has unknown kind '{}'. Valid kinds: {:?}",
                    region.id,
                    screen.id,
                    region.kind,
                    contract.region_kinds.keys().collect::<Vec<_>>()
                );
            }
        }
    }
    Ok(())
}

/// Get all region IDs across all screens (for test assertions).
pub fn all_region_ids(contract: &UiContract) -> Vec<String> {
    contract.screens.iter().flat_map(|s| s.regions.iter().map(|r| r.id.clone())).collect()
}

/// Get region IDs for a specific screen.
pub fn region_ids_for_screen(contract: &UiContract, screen_id: &str) -> Option<Vec<String>> {
    contract
        .screens
        .iter()
        .find(|s| s.id == screen_id)
        .map(|s| s.regions.iter().map(|r| r.id.clone()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_contract() {
        let yaml = r#"
schema_version: "1.0"
template_version: "v3.3.6"
screens:
  - id: test_screen
    route: "/test"
    description: "Test screen"
    regions:
      - id: "test.header"
        kind: "header"
        description: "Header region"
      - id: "test.content"
        kind: "panel"
        description: "Content region"
region_kinds:
  header: "Top-level branding"
  panel: "Grouped content section"
"#;
        let contract: UiContract = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(contract.screens.len(), 1);
        assert_eq!(contract.screens[0].regions.len(), 2);
        assert_eq!(contract.region_kinds.len(), 2);
    }

    #[test]
    fn detects_duplicate_region_ids() {
        let yaml = r#"
schema_version: "1.0"
template_version: "v3.3.6"
screens:
  - id: test_screen
    route: "/test"
    description: "Test screen"
    regions:
      - id: "test.header"
        kind: "header"
        description: "Header region"
      - id: "test.header"
        kind: "panel"
        description: "Duplicate!"
region_kinds: {}
"#;
        // Verify parsing succeeds (YAML is valid, validation happens at load time)
        let _contract: UiContract = serde_yaml::from_str(yaml).unwrap();

        // Create a temporary file for testing
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), yaml).unwrap();
        let result = load_ui_contract(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate region ID"));
    }

    #[test]
    fn all_region_ids_collects_all() {
        let contract = UiContract {
            schema_version: "1.0".into(),
            template_version: "v3.3.6".into(),
            screens: vec![
                Screen {
                    id: "screen1".into(),
                    route: "/s1".into(),
                    aliases: vec![],
                    description: "Screen 1".into(),
                    regions: vec![Region {
                        id: "s1.header".into(),
                        kind: "header".into(),
                        description: "Header".into(),
                    }],
                },
                Screen {
                    id: "screen2".into(),
                    route: "/s2".into(),
                    aliases: vec![],
                    description: "Screen 2".into(),
                    regions: vec![Region {
                        id: "s2.content".into(),
                        kind: "panel".into(),
                        description: "Content".into(),
                    }],
                },
            ],
            region_kinds: HashMap::new(),
        };
        let ids = all_region_ids(&contract);
        assert_eq!(ids, vec!["s1.header", "s2.content"]);
    }
}
