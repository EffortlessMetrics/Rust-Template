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
use gov_http_core::{PlatformError, PlatformState, YamlResourceRepo};
use serde::Serialize;

// Re-export types from gov-http-types for backwards compatibility
pub use gov_http_friction_types::{FrictionContext, FrictionEntry, RelatedItems, Resolution};

#[derive(Debug, Serialize)]
pub struct FrictionListResponse {
    pub entries: Vec<FrictionEntry>,
    pub total: usize,
    pub pagination: gov_http_core::Pagination,
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

/// GET /friction - Get all friction entries
async fn get_all_friction<S>(
    State(state): State<S>,
) -> Result<Json<FrictionListResponse>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let response = tokio::task::spawn_blocking(move || {
        let repo = YamlResourceRepo::<FrictionEntry>::new(&root, "friction");
        repo.list(
            gov_http_core::PaginationParams::default(),
            |_: &FrictionEntry| true,
            |a: &FrictionEntry, b: &FrictionEntry| b.date.cmp(&a.date), // Sort by date desc
        )
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(Json(FrictionListResponse {
        total: response.pagination.total_items,
        entries: response.data,
        pagination: response.pagination,
    }))
}

/// GET /friction/{id} - Get a specific friction entry by ID
async fn get_friction_by_id<S>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<FrictionEntry>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let entry = tokio::task::spawn_blocking(move || {
        spec_runtime::validate_friction_id(&id)
            .map_err(|e| PlatformError::internal(format!("Invalid ID format: {}", e)))?;

        let repo = YamlResourceRepo::<FrictionEntry>::new(&root, "friction");
        repo.get(&id)
    })
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
