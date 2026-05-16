use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Public Types
// ============================================================================

/// Complete platform schema information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSchemas {
    pub schemas: Vec<SchemaInfo>,
    pub endpoints: Vec<EndpointSchema>,
}

/// Information about a specific schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source_file: String,
    pub json_schema: Value,
}

/// API endpoint schema information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSchema {
    pub path: String,
    pub method: String,
    pub description: String,
    pub request_type: Option<String>,
    pub response_type: String,
}
