//! Platform contract types for `/platform/*` endpoints.
//!
//! This crate defines stable, dependency-light interfaces for the platform API.
//! It contains:
//! - Error envelope types for consistent error responses
//! - Response/request DTOs for platform endpoints
//! - Version fields and schema IDs
//!
//! # Design Philosophy
//!
//! - **Minimal dependencies**: Only serde, serde_json, chrono, uuid, thiserror
//! - **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
//! - **Internal-only**: `publish = false` - this is a contract crate for internal use
//!
//! # Example Error Response
//!
//! ```json
//! {
//!   "error": "INVALID_REQUEST",
//!   "message": "Invalid input",
//!   "requestId": "uuid-v4-here"
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Error Envelope Types
// ============================================================================

/// Machine-readable error codes for platform responses.
///
/// These codes allow clients to programmatically handle different error scenarios
/// without parsing error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ErrorCode {
    // Validation errors (4xx)
    #[serde(rename = "INVALID_REQUEST")]
    InvalidRequest,
    #[serde(rename = "INVALID_AMOUNT")]
    InvalidAmount,
    #[serde(rename = "MISSING_FIELD")]
    MissingField,
    #[serde(rename = "INVALID_FORMAT")]
    InvalidFormat,
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,

    // Business logic errors (4xx)
    #[serde(rename = "RESOURCE_NOT_FOUND")]
    ResourceNotFound,
    #[serde(rename = "INVALID_STATE")]
    InvalidState,
    #[serde(rename = "INVALID_TRANSITION")]
    InvalidTransition,
    #[serde(rename = "CONFLICT")]
    Conflict,
    #[serde(rename = "DUPLICATE_REQUEST")]
    DuplicateRequest,

    // System errors (5xx)
    #[serde(rename = "INTERNAL_ERROR")]
    InternalError,
    #[serde(rename = "SERVICE_UNAVAILABLE")]
    ServiceUnavailable,
    #[serde(rename = "DATABASE_ERROR")]
    DatabaseError,
    #[serde(rename = "EXTERNAL_SERVICE_ERROR")]
    ExternalServiceError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InvalidRequest => write!(f, "INVALID_REQUEST"),
            ErrorCode::InvalidAmount => write!(f, "INVALID_AMOUNT"),
            ErrorCode::MissingField => write!(f, "MISSING_FIELD"),
            ErrorCode::InvalidFormat => write!(f, "INVALID_FORMAT"),
            ErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            ErrorCode::ResourceNotFound => write!(f, "RESOURCE_NOT_FOUND"),
            ErrorCode::InvalidState => write!(f, "INVALID_STATE"),
            ErrorCode::InvalidTransition => write!(f, "INVALID_TRANSITION"),
            ErrorCode::Conflict => write!(f, "CONFLICT"),
            ErrorCode::DuplicateRequest => write!(f, "DUPLICATE_REQUEST"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
            ErrorCode::ServiceUnavailable => write!(f, "SERVICE_UNAVAILABLE"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ExternalServiceError => write!(f, "EXTERNAL_SERVICE_ERROR"),
        }
    }
}

/// JSON error response body for platform endpoints.
///
/// This format is required for consistent error handling across all platform endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ErrorResponse {
    /// Machine-readable error code
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Request ID for correlation
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// Optional AC ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac_id: Option<String>,
    /// Optional Feature ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
}

/// Summary of the last error that occurred.
///
/// This is surfaced via `/platform/status` for observability by agents and portals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastErrorSummary {
    /// Error category (e.g., "task_not_found", "invalid_transition", "internal")
    pub category: String,
    /// Human-readable error message
    pub message: String,
    /// HTTP status code returned
    pub status_code: u16,
    /// When the error occurred
    pub occurred_at: DateTime<Utc>,
    /// Request ID for correlation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Aggregated error statistics for the service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ErrorStats {
    /// Total number of errors since service start
    pub total_errors: u64,
    /// Number of 4xx client errors
    pub client_errors: u64,
    /// Number of 5xx server errors
    pub server_errors: u64,
}

/// Error summary surfaced via `/platform/status`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ErrorSummary {
    /// Whether any errors have occurred recently (since service start)
    pub has_recent_errors: bool,
    /// The last error that occurred (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<LastErrorSummary>,
    /// Aggregated error statistics
    pub stats: ErrorStats,
}

// ============================================================================
// Platform Status DTOs
// ============================================================================

/// Platform status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PlatformStatus {
    /// Service information
    pub service: ServiceInfo,
    /// Governance status
    pub governance: GovernanceStatus,
    /// Optional config summary (redacted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConfigSummary>,
    /// Error summary
    pub errors: ErrorSummary,
}

/// Service information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServiceInfo {
    /// Service ID
    pub service_id: String,
    /// Template version from spec_ledger.yaml
    pub template_version: String,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Links to external resources
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub links: HashMap<String, String>,
    /// Tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

/// Governance status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GovernanceStatus {
    /// Ledger counts
    pub ledger: LedgerCounts,
    /// DevEx counts
    pub devex: DevExCounts,
    /// Documentation counts
    pub docs: DocCounts,
    /// Task counts
    pub tasks: TaskCounts,
    /// Question counts
    pub questions: QuestionCounts,
    /// Friction counts
    pub friction: FrictionCounts,
    /// Fork counts
    pub forks: ForkCounts,
    /// Policy status
    pub policies: PolicyStatus,
    /// AC coverage info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac_coverage: Option<AcCoverageInfo>,
}

/// Ledger counts (stories, requirements, ACs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerCounts {
    /// Number of stories
    pub stories: usize,
    /// Number of requirements
    pub requirements: usize,
    /// Number of acceptance criteria
    pub acs: usize,
}

/// DevEx counts (commands, flows).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DevExCounts {
    /// Number of DevEx commands
    pub commands: usize,
    /// Number of DevEx flows
    pub flows: usize,
}

/// Documentation counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DocCounts {
    /// Total number of docs
    pub total: usize,
    /// Number of design docs
    pub design: usize,
    /// Number of doc type issues
    pub doc_type_issues: usize,
}

/// Task counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskCounts {
    /// Total number of tasks
    pub total: usize,
    /// Task status breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_status: Option<TaskStatusBreakdown>,
}

/// Task status breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskStatusBreakdown {
    /// Number of Todo tasks
    pub todo: usize,
    /// Number of InProgress tasks
    pub in_progress: usize,
    /// Number of Review tasks
    pub review: usize,
    /// Number of Done tasks
    pub done: usize,
}

/// AC coverage info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcCoverageInfo {
    /// Total ACs
    pub total: usize,
    /// Passing ACs
    pub passing: usize,
    /// Failing ACs
    pub failing: usize,
    /// Unknown ACs
    pub unknown: usize,
}

/// Question counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionCounts {
    /// Number of open questions
    pub open: usize,
    /// Number of answered questions
    pub answered: usize,
    /// Number of resolved questions
    pub resolved: usize,
    /// Total number of questions
    pub total: usize,
    /// Top open questions
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub top_open: Vec<QuestionBrief>,
}

/// Question brief for status endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionBrief {
    /// Question ID
    pub id: String,
    /// Question summary
    pub summary: String,
    /// Flow name
    pub flow: String,
}

/// Friction counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionCounts {
    /// Total number of friction entries
    pub total: usize,
    /// Number of open friction entries
    pub open: usize,
    /// Counts by severity
    pub by_severity: SeverityCounts,
    /// Recent friction entries
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub recent: Vec<FrictionSummary>,
}

/// Severity counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeverityCounts {
    /// Low severity count
    pub low: usize,
    /// Medium severity count
    pub medium: usize,
    /// High severity count
    pub high: usize,
    /// Critical severity count
    pub critical: usize,
}

/// Friction summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionSummary {
    /// Friction ID
    pub id: String,
    /// Date
    pub date: String,
    /// Severity
    pub severity: String,
    /// Summary
    pub summary: String,
    /// Category
    pub category: String,
}

/// Fork counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForkCounts {
    /// Total number of forks
    pub total: usize,
    /// Fork IDs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ids: Vec<String>,
}

/// Policy status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PolicyStatus {
    /// Status string
    pub status: String,
}

/// Config summary (redacted).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfigSummary {
    /// Environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// HTTP port
    pub http_port: u16,
    /// Settings
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub settings: HashMap<String, serde_json::Value>,
    /// Redacted secrets
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub secrets_redacted: HashMap<String, String>,
    /// Auth summary
    pub auth: AuthSummary,
}

/// Auth summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuthSummary {
    /// Auth mode
    pub mode: String,
    /// Whether token is present
    pub token_present: bool,
}

// ============================================================================
// Convenience constructors
// ============================================================================

#[allow(missing_docs)]
impl LedgerCounts {
    pub fn new(stories: usize, requirements: usize, acs: usize) -> Self {
        Self { stories, requirements, acs }
    }
}

#[allow(missing_docs)]
impl DevExCounts {
    pub fn new(commands: usize, flows: usize) -> Self {
        Self { commands, flows }
    }
}

#[allow(missing_docs)]
impl DocCounts {
    pub fn new(total: usize, design: usize, doc_type_issues: usize) -> Self {
        Self { total, design, doc_type_issues }
    }
}

#[allow(missing_docs)]
impl TaskStatusBreakdown {
    pub fn new(todo: usize, in_progress: usize, review: usize, done: usize) -> Self {
        Self { todo, in_progress, review, done }
    }
}

#[allow(missing_docs)]
impl TaskCounts {
    pub fn new(total: usize, by_status: Option<TaskStatusBreakdown>) -> Self {
        Self { total, by_status }
    }
}

#[allow(missing_docs)]
impl AcCoverageInfo {
    pub fn new(total: usize, passing: usize, failing: usize, unknown: usize) -> Self {
        Self { total, passing, failing, unknown }
    }
}

#[allow(missing_docs)]
impl QuestionBrief {
    pub fn new(id: String, summary: String, flow: String) -> Self {
        Self { id, summary, flow }
    }
}

#[allow(missing_docs)]
impl QuestionCounts {
    pub fn new(
        open: usize,
        answered: usize,
        resolved: usize,
        total: usize,
        top_open: Vec<QuestionBrief>,
    ) -> Self {
        Self { open, answered, resolved, total, top_open }
    }
}

#[allow(missing_docs)]
impl SeverityCounts {
    pub fn new(low: usize, medium: usize, high: usize, critical: usize) -> Self {
        Self { low, medium, high, critical }
    }
}

#[allow(missing_docs)]
impl FrictionSummary {
    pub fn new(
        id: String,
        date: String,
        severity: String,
        summary: String,
        category: String,
    ) -> Self {
        Self { id, date, severity, summary, category }
    }
}

#[allow(missing_docs)]
impl FrictionCounts {
    pub fn new(
        total: usize,
        open: usize,
        by_severity: SeverityCounts,
        recent: Vec<FrictionSummary>,
    ) -> Self {
        Self { total, open, by_severity, recent }
    }
}

#[allow(missing_docs)]
impl ForkCounts {
    pub fn new(total: usize, ids: Vec<String>) -> Self {
        Self { total, ids }
    }
}

#[allow(missing_docs)]
impl PolicyStatus {
    pub fn new(status: String) -> Self {
        Self { status }
    }
}

#[allow(missing_docs)]
impl ErrorStats {
    pub fn new(total_errors: u64, client_errors: u64, server_errors: u64) -> Self {
        Self { total_errors, client_errors, server_errors }
    }
}

#[allow(missing_docs)]
impl ErrorSummary {
    pub fn new(
        has_recent_errors: bool,
        last_error: Option<LastErrorSummary>,
        stats: ErrorStats,
    ) -> Self {
        Self { has_recent_errors, last_error, stats }
    }
}

#[allow(missing_docs)]
impl AuthSummary {
    pub fn new(mode: String, token_present: bool) -> Self {
        Self { mode, token_present }
    }
}

#[allow(missing_docs)]
impl ConfigSummary {
    pub fn new(
        env: Option<String>,
        http_port: u16,
        settings: HashMap<String, serde_json::Value>,
        secrets_redacted: HashMap<String, String>,
        auth: AuthSummary,
    ) -> Self {
        Self { env, http_port, settings, secrets_redacted, auth }
    }
}

#[allow(missing_docs)]
impl ServiceInfo {
    pub fn new(
        service_id: String,
        template_version: String,
        display_name: Option<String>,
        description: Option<String>,
        links: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Self {
        Self { service_id, template_version, display_name, description, links, tags }
    }
}

#[allow(missing_docs)]
#[allow(clippy::too_many_arguments)]
impl GovernanceStatus {
    pub fn new(
        ledger: LedgerCounts,
        devex: DevExCounts,
        docs: DocCounts,
        tasks: TaskCounts,
        questions: QuestionCounts,
        friction: FrictionCounts,
        forks: ForkCounts,
        policies: PolicyStatus,
        ac_coverage: Option<AcCoverageInfo>,
    ) -> Self {
        Self { ledger, devex, docs, tasks, questions, friction, forks, policies, ac_coverage }
    }
}

// ============================================================================
// IDP Snapshot DTOs
// ============================================================================

/// IDP snapshot output structure (machine-readable contract for IDPs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IdpSnapshot {
    /// ISO 8601 timestamp of snapshot creation
    pub timestamp: String,
    /// Template version from spec_ledger.yaml
    pub template_version: String,
    /// Service ID from service_metadata.yaml (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    /// Governance health metrics
    pub governance_health: GovernanceHealth,
    /// Documentation metrics
    pub documentation: DocumentationMetrics,
    /// Task hints for agents
    pub task_hints: TaskHints,
}

/// Governance health.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GovernanceHealth {
    /// Overall status: "healthy", "degraded", or "failing"
    pub status: String,
    /// AC coverage metrics from BDD tests
    pub ac_coverage: AcCoverage,
    /// Story/requirement/AC counts from spec_ledger
    pub spec_counts: SpecCounts,
}

/// AC coverage metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcCoverage {
    /// Total ACs
    pub total: usize,
    /// Passing ACs
    pub passing: usize,
    /// Failing ACs
    pub failing: usize,
    /// Unknown ACs
    pub unknown: usize,
}

/// Spec counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpecCounts {
    /// Number of stories
    pub stories: usize,
    /// Number of requirements
    pub requirements: usize,
    /// Number of acceptance criteria
    pub acceptance_criteria: usize,
}

/// Documentation metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DocumentationMetrics {
    /// Total docs
    pub total: usize,
    /// Valid docs
    pub valid: usize,
    /// Docs with issues
    pub with_issues: usize,
}

/// Task hints for agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskHints {
    /// Total pending tasks
    pub total_pending: usize,
    /// Total in-progress tasks
    pub total_in_progress: usize,
    /// Friction count
    pub friction_count: usize,
    /// Question count
    pub question_count: usize,
    /// High-priority tasks
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub high_priority: Vec<TaskHint>,
}

/// Task hint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskHint {
    /// Task ID
    pub task_id: String,
    /// Task title
    pub title: String,
    /// Task status
    pub status: String,
    /// Task owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Requirement IDs
    pub requirement_ids: Vec<String>,
    /// AC IDs
    pub ac_ids: Vec<String>,
}

// ============================================================================
// Debug Info DTOs
// ============================================================================

/// Debug info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DebugInfo {
    /// Kernel version
    pub kernel_version: String,
    /// Template version
    pub template_version: String,
}

// ============================================================================
// Coverage DTOs
// ============================================================================

/// Coverage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoverageSummary {
    /// Passing ACs
    pub passing: usize,
    /// Failing ACs
    pub failing: usize,
    /// Unknown ACs
    pub unknown: usize,
    /// Total ACs
    pub total: usize,
}

/// Coverage detail for a single AC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoverageDetail {
    /// AC ID
    pub id: String,
    /// AC title
    pub title: String,
    /// AC status
    pub status: String,
    /// Story ID
    pub story: String,
    /// Requirement ID
    pub requirement: String,
    /// Scenarios
    pub scenarios: Vec<String>,
}

/// Coverage response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoverageResponse {
    /// Summary
    pub summary: CoverageSummary,
    /// Details
    pub details: Vec<CoverageDetail>,
}

// ============================================================================
// Schema ID Types
// ============================================================================

/// Schema ID for platform schemas.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaId(pub String);

impl std::fmt::Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SchemaId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidAmount.to_string(), "INVALID_AMOUNT");
        assert_eq!(ErrorCode::ResourceNotFound.to_string(), "RESOURCE_NOT_FOUND");
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: ErrorCode::InvalidAmount.to_string(),
            message: "Invalid input".to_string(),
            request_id: "test-123".to_string(),
            ac_id: Some("AC-123".to_string()),
            feature_id: Some("FT-456".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("INVALID_AMOUNT"));
        assert!(json.contains("Invalid input"));
        assert!(json.contains("test-123"));
        assert!(json.contains("AC-123"));
        assert!(json.contains("FT-456"));
    }

    #[test]
    fn test_schema_id_display() {
        let id = SchemaId("test-schema".to_string());
        assert_eq!(id.to_string(), "test-schema");
        assert_eq!(id.as_ref(), "test-schema");
    }
}
