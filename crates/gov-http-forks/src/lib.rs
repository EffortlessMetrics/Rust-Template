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
use gov_http_core::{PlatformError, PlatformState, YamlResourceRepo};
use serde::{Deserialize, Serialize};

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

impl gov_model::YamlResource for ForkEntry {
    fn id(&self) -> &str {
        &self.id
    }
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
    pub pagination: gov_http_core::Pagination,
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

/// GET /forks - Get all fork entries
async fn get_all_forks<S>(State(state): State<S>) -> Result<Json<ForksListResponse>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let response = tokio::task::spawn_blocking(move || {
        let repo = YamlResourceRepo::<ForkEntry>::new(&root, "forks");
        repo.list(
            gov_http_core::PaginationParams::default(),
            |f: &ForkEntry| f.id.starts_with("FORK-"), // Only load files matching FORK-*.yaml pattern
            |a: &ForkEntry, b: &ForkEntry| a.id.cmp(&b.id), // Sort by ID
        )
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    let summaries: Vec<ForkSummary> = response
        .data
        .iter()
        .map(|f| ForkSummary {
            id: f.id.clone(),
            name: f.name.clone(),
            domain: f.domain.clone(),
            status: f.status.clone(),
            kernel_version: f.kernel_version.clone(),
        })
        .collect();

    Ok(Json(ForksListResponse {
        forks: summaries,
        total: response.pagination.total_items,
        pagination: response.pagination,
    }))
}

/// GET /forks/{name} - Get a specific fork entry by ID or name
async fn get_fork_by_name<S>(
    State(state): State<S>,
    Path(name): Path<String>,
) -> Result<Json<ForkEntry>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let fork = tokio::task::spawn_blocking(move || {
        // Validate name/ID format first
        spec_runtime::validate_fork_id(&name)
            .or_else(|_| spec_runtime::validate_name("fork_name", &name))
            .map_err(|e| PlatformError::internal(format!("Invalid fork identifier: {}", e)))?;

        let repo = YamlResourceRepo::<ForkEntry>::new(&root, "forks");

        // Try exact ID match first
        if let Ok(fork) = repo.get(&name) {
            return Ok(fork);
        }

        // Then try name match (more expensive)
        let list = repo.list(
            gov_http_core::PaginationParams::default(),
            |_: &ForkEntry| true,
            |_, _| std::cmp::Ordering::Equal,
        )?;
        list.data
            .into_iter()
            .find(|f| f.name.eq_ignore_ascii_case(&name))
            .ok_or_else(|| PlatformError::not_found(format!("Fork '{}' not found", name)))
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(Json(fork))
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
