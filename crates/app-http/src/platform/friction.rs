use crate::{AppError, AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Friction entry representing process/tooling issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionEntry {
    pub id: String,
    pub date: String,
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workaround: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FrictionContext>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub resolved_by: String,
    pub resolved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pr_links: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FrictionListResponse {
    pub entries: Vec<FrictionEntry>,
    pub total: usize,
}

/// Router for friction endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/friction", get(get_all_friction))
        .route("/friction/{id}", get(get_friction_by_id))
}

/// Load all friction entries from friction/ directory
#[allow(clippy::result_large_err)]
fn load_all_friction_entries(
    workspace_root: &std::path::Path,
) -> Result<Vec<FrictionEntry>, AppError> {
    let friction_dir = workspace_root.join("friction");

    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(&friction_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to read friction directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            AppError::internal_error(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_friction_entry(&path) {
            Ok(friction) => entries.push(friction),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load friction entry"
                );
            }
        }
    }

    // Sort by date (most recent first)
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(entries)
}

/// Load a single friction entry from a YAML file
#[allow(clippy::result_large_err)]
fn load_friction_entry(path: &std::path::Path) -> Result<FrictionEntry, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::internal_error(format!("Failed to read friction file {}: {}", path.display(), e))
    })?;

    let entry: FrictionEntry = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse friction YAML {}: {}", path.display(), e))
    })?;

    Ok(entry)
}

/// GET /platform/friction - Get all friction entries
async fn get_all_friction(
    State(state): State<AppState>,
) -> Result<Json<FrictionListResponse>, AppError> {
    let root = &state.workspace_root;
    let entries = load_all_friction_entries(root)?;
    let total = entries.len();

    Ok(Json(FrictionListResponse { entries, total }))
}

/// GET /platform/friction/:id - Get a specific friction entry by ID
async fn get_friction_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FrictionEntry>, AppError> {
    let root = &state.workspace_root;
    let friction_dir = root.join("friction");

    // Construct the expected file path
    let file_path = friction_dir.join(format!("{}.yaml", id));

    // Check if file exists
    if !file_path.exists() {
        return Err(AppError::not_found(format!("Friction entry '{}' not found", id)));
    }

    // Load and return the entry
    let entry = load_friction_entry(&file_path)?;

    // Verify the ID matches (sanity check)
    if entry.id != id {
        return Err(AppError::internal_error(format!(
            "Friction entry ID mismatch: expected '{}', found '{}'",
            id, entry.id
        )));
    }

    Ok(Json(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friction_entry_serialization() {
        let entry = FrictionEntry {
            id: "FRICTION-TEST-001".to_string(),
            date: "2025-11-26".to_string(),
            category: "testing".to_string(),
            severity: "low".to_string(),
            summary: "Test friction entry".to_string(),
            description: "Test description".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "open".to_string(),
            resolution: None,
            refs: Vec::new(),
            related_items: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("FRICTION-TEST-001"));
        assert!(json.contains("testing"));
    }

    #[test]
    fn test_friction_entry_deserialization() {
        let yaml = r#"
id: FRICTION-TEST-002
date: "2025-11-26"
category: devex
severity: medium
summary: "Test friction"
description: "Test description"
status: open
"#;

        let entry: FrictionEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.id, "FRICTION-TEST-002");
        assert_eq!(entry.category, "devex");
        assert_eq!(entry.status, "open");
    }

    #[test]
    fn test_default_status() {
        let yaml = r#"
id: FRICTION-TEST-003
date: "2025-11-26"
category: tooling
severity: high
summary: "Test"
description: "Test"
"#;

        let entry: FrictionEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.status, "open");
    }
}
