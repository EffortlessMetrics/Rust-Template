//! Fork registry endpoints for tracking template forks.
//!
//! Forks represent known deployments/customizations of the template,
//! tracking their kernel versions, status, and relationships.
//!
//! # Endpoints
//!
//! - `GET /forks` - List all forks
//! - `GET /forks/{name}` - Get a specific fork by ID or name

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use gov_http_core::{PlatformError, PlatformState};
use serde::{Deserialize, Serialize};
use std::fs;

/// Fork registry entry representing a known template fork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEntry {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub kernel_version: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<Maintainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pain_points: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub friction: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ForksListResponse {
    pub forks: Vec<ForkSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ForkSummary {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub status: String,
    pub kernel_version: String,
}

/// Router for fork endpoints.
///
/// Returns a router that handles:
/// - `GET /forks` - List all forks
/// - `GET /forks/{name}` - Get a specific fork by ID or name
pub fn router<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/forks", get(get_all_forks::<S>))
        .route("/forks/{name}", get(get_fork_by_name::<S>))
}

/// Load all fork entries from forks/ directory
fn load_all_forks(root: &std::path::Path) -> Result<Vec<ForkEntry>, PlatformError> {
    let forks_dir = root.join("forks");

    if !forks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut forks = Vec::new();

    let dir_entries = fs::read_dir(&forks_dir)
        .map_err(|e| PlatformError::internal(format!("Failed to read forks directory: {}", e)))?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and special files
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("README.yaml") | Some("fork_registry.yaml") | Some("fork_schema.yaml")
            )
        {
            continue;
        }

        // Only load files matching FORK-*.yaml pattern
        #[allow(clippy::collapsible_if)]
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if !filename.starts_with("FORK-") {
                continue;
            }
        }

        match load_fork_entry(&path) {
            Ok(fork) => forks.push(fork),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load fork entry"
                );
            }
        }
    }

    // Sort by ID for consistent output
    forks.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(forks)
}

/// Load a single fork entry from a YAML file
fn load_fork_entry(path: &std::path::Path) -> Result<ForkEntry, PlatformError> {
    let content = fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read fork file {}: {}", path.display(), e))
    })?;

    let fork: ForkEntry = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse fork YAML {}: {}", path.display(), e))
    })?;

    Ok(fork)
}

/// GET /forks - Get all fork entries
async fn get_all_forks<S>(State(state): State<S>) -> Result<Json<ForksListResponse>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root();
    let forks = load_all_forks(root)?;

    let summaries: Vec<ForkSummary> = forks
        .iter()
        .map(|f| ForkSummary {
            id: f.id.clone(),
            name: f.name.clone(),
            domain: f.domain.clone(),
            status: f.status.clone(),
            kernel_version: f.kernel_version.clone(),
        })
        .collect();

    let total = summaries.len();

    Ok(Json(ForksListResponse { forks: summaries, total }))
}

/// GET /forks/{name} - Get a specific fork entry by ID or name
async fn get_fork_by_name<S>(
    State(state): State<S>,
    Path(name): Path<String>,
) -> Result<Json<ForkEntry>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root();
    let forks_dir = root.join("forks");

    // Try to find the fork file
    // It could be the full ID (FORK-XXX-001) or just the identifier
    let possible_filenames =
        vec![format!("{}.yaml", name), format!("FORK-{}.yaml", name.trim_start_matches("FORK-"))];

    for filename in possible_filenames {
        let file_path = forks_dir.join(&filename);
        if file_path.exists() {
            let fork = load_fork_entry(&file_path)?;

            // Verify the ID or name matches
            if fork.id != name && !fork.name.eq_ignore_ascii_case(&name) {
                tracing::warn!(
                    requested = %name,
                    found_id = %fork.id,
                    found_name = %fork.name,
                    file = %file_path.display(),
                    "Fork identifier mismatch"
                );
            }

            return Ok(Json(fork));
        }
    }

    Err(PlatformError::not_found(format!("Fork '{}' not found", name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_entry_serialization() {
        let fork = ForkEntry {
            id: "FORK-TEST-001".to_string(),
            name: "Test Fork".to_string(),
            domain: "testing".to_string(),
            kernel_version: "v3.3.3".to_string(),
            status: "active".to_string(),
            url: Some("https://github.com/test/fork".to_string()),
            maintainer: Some(Maintainer {
                name: "Test Maintainer".to_string(),
                contact: Some("test@example.com".to_string()),
            }),
            forked_at: Some("2025-11-26".to_string()),
            last_synced: None,
            features: vec!["feature1".to_string()],
            pain_points: vec![],
            notes: None,
            related_items: None,
        };

        let json = serde_json::to_string(&fork).expect("fork should serialize to JSON");
        assert!(json.contains("FORK-TEST-001"));
        assert!(json.contains("testing"));
    }

    #[test]
    fn test_fork_entry_deserialization() {
        let yaml = r#"
id: FORK-TEST-002
name: "Test Fork 2"
domain: rust-sdk
kernel_version: v3.3.3
status: active
"#;

        let fork: ForkEntry =
            serde_yaml::from_str(yaml).expect("YAML should deserialize to ForkEntry");
        assert_eq!(fork.id, "FORK-TEST-002");
        assert_eq!(fork.domain, "rust-sdk");
        assert_eq!(fork.status, "active");
    }
}
