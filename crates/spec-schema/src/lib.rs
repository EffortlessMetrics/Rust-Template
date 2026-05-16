#![recursion_limit = "256"]

//! Schema generation (OpenAPI, JSON schema).
//!
//! This crate provides schema generation functions for platform APIs and
//! configuration. This is where jsonschema lives - don't leak it elsewhere.
//!
//! # Design Principles
//!
//! - **Heavy deps isolated**: This crate contains jsonschema and other heavy dependencies
//! - **Foundation**: Depends on spec-types and spec-ledger
//! - **No axum**: HTTP/web dependencies are isolated to higher-level crates
//!
//! # Example
//!
//! ```ignore
//! use spec_schema::{get_all_schemas, get_schema_by_name};
//!
//! let schemas = get_all_schemas();
//! let ledger_schema = get_schema_by_name("spec_ledger").unwrap();
//! ```

#![allow(missing_docs)]

mod definitions;
mod endpoints;
mod types;

pub use types::{EndpointSchema, PlatformSchemas, SchemaInfo};

// ============================================================================
// Schema Generation
// ============================================================================

/// Get all platform schemas.
///
/// Returns schemas for:
/// - spec_ledger
/// - tasks
/// - questions
/// - devex_flows
/// - config
/// - doc_index
/// - service_metadata
pub fn get_all_schemas() -> PlatformSchemas {
    PlatformSchemas {
        schemas: vec![
            definitions::get_spec_ledger_schema(),
            definitions::get_tasks_schema(),
            definitions::get_questions_schema(),
            definitions::get_devex_flows_schema(),
            definitions::get_config_schema(),
            definitions::get_doc_index_schema(),
            definitions::get_service_metadata_schema(),
        ],
        endpoints: endpoints::get_platform_endpoints(),
    }
}

/// Get schema by name.
///
/// Returns the schema with the specified name, or None if not found.
pub fn get_schema_by_name(name: &str) -> Option<SchemaInfo> {
    get_all_schemas().schemas.into_iter().find(|s| s.name == name)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_schemas() {
        let schemas = get_all_schemas();
        assert!(schemas.schemas.len() >= 7, "Should have at least 7 schemas");
    }

    #[test]
    fn test_get_schema_by_name() {
        let ledger_schema = get_schema_by_name("spec_ledger");
        assert!(ledger_schema.is_some());
        assert_eq!(ledger_schema.unwrap().name, "spec_ledger");

        let unknown_schema = get_schema_by_name("nonexistent");
        assert!(unknown_schema.is_none());
    }

    #[test]
    fn test_platform_endpoints_complete() {
        let endpoints = endpoints::get_platform_endpoints();
        let paths: Vec<&str> = endpoints.iter().map(|e| e.path.as_str()).collect();

        assert!(paths.contains(&"/platform/status"));
        assert!(paths.contains(&"/platform/graph"));
        assert!(paths.contains(&"/platform/schema"));
        assert!(paths.contains(&"/platform/tasks"));
        assert!(paths.contains(&"/platform/idp/snapshot"));
    }
}
