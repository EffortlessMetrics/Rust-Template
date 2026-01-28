//! Friction log endpoints for tracking development friction.
//!
//! Friction entries capture process, tooling, and DevEx issues discovered
//! during development workflows.
//!
//! # Endpoints
//!
//! - `GET /friction` - List all friction entries
//! - `GET /friction/{id}` - Get a specific friction entry

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use gov_http_core::{PlatformError, PlatformState};
use serde::Serialize;
use std::path::PathBuf;

// Re-export types from gov-http-types for backwards compatibility
pub use gov_http_types::{FrictionContext, FrictionEntry, RelatedItems, Resolution};

#[derive(Debug, Serialize)]
pub struct FrictionListResponse {
    pub entries: Vec<FrictionEntry>,
    pub total: usize,
}

/// Router for friction endpoints.
///
/// Returns a router that handles:
/// - `GET /friction` - List all friction entries
/// - `GET /friction/{id}` - Get a specific friction entry
pub fn router<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/friction", get(get_all_friction::<S>))
        .route("/friction/{id}", get(get_friction_by_id::<S>))
}

/// Load all friction entries from friction/ directory.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_all_friction_entries(root: &std::path::Path) -> Result<Vec<FrictionEntry>, PlatformError> {
    let friction_dir = root.join("friction");

    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let dir_entries = std::fs::read_dir(&friction_dir).map_err(|e| {
        PlatformError::internal(format!("Failed to read friction directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
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

/// Load a single friction entry from a YAML file.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_friction_entry(path: &std::path::Path) -> Result<FrictionEntry, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read friction file {}: {}", path.display(), e))
    })?;

    let entry: FrictionEntry = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse friction YAML {}: {}", path.display(), e))
    })?;

    Ok(entry)
}

/// GET /friction - Get all friction entries
async fn get_all_friction<S>(
    State(state): State<S>,
) -> Result<Json<FrictionListResponse>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    // Offload blocking filesystem I/O to spawn_blocking to avoid starving
    // the Tokio async runtime under concurrent load.
    let entries = tokio::task::spawn_blocking(move || load_all_friction_entries(&root))
        .await
        .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    let total = entries.len();

    Ok(Json(FrictionListResponse { entries, total }))
}

/// Load a friction entry by ID.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_friction_by_id(
    friction_dir: &std::path::Path,
    id: &str,
) -> Result<FrictionEntry, PlatformError> {
    // Construct the expected file path
    let file_path = friction_dir.join(format!("{}.yaml", id));

    // Check if file exists
    if !file_path.exists() {
        return Err(PlatformError::not_found(format!("Friction entry '{}' not found", id)));
    }

    // Load and return the entry
    let entry = load_friction_entry(&file_path)?;

    // Verify the ID matches (sanity check)
    if entry.id != id {
        return Err(PlatformError::internal(format!(
            "Friction entry ID mismatch: expected '{}', found '{}'",
            id, entry.id
        )));
    }

    Ok(entry)
}

/// GET /friction/{id} - Get a specific friction entry by ID
async fn get_friction_by_id<S>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<FrictionEntry>, PlatformError>
where
    S: PlatformState,
{
    let friction_dir: PathBuf = state.context().root().join("friction");

    // Offload blocking filesystem I/O to spawn_blocking to avoid starving
    // the Tokio async runtime under concurrent load.
    let entry = tokio::task::spawn_blocking(move || load_friction_by_id(&friction_dir, &id))
        .await
        .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

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

        let json = serde_json::to_string(&entry).expect("entry should serialize to JSON");
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

        let entry: FrictionEntry =
            serde_yaml::from_str(yaml).expect("YAML should deserialize to FrictionEntry");
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

        let entry: FrictionEntry =
            serde_yaml::from_str(yaml).expect("YAML should deserialize to FrictionEntry");
        assert_eq!(entry.status, "open");
    }
}
