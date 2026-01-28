//! XTask contract types for `--json` output and exit codes.
//!
//! This crate defines stable, dependency-light interfaces for xtask JSON output.
//! It contains:
//! - JSON output DTOs for commands like `ac-status --json`, `issues-search --json`
//! - Exit code enums for CLI exit behavior
//! - Status enums for command status
//! - Common JSON response envelope types
//!
//! # Design Philosophy
//!
//! - **Minimal dependencies**: Only serde, serde_json, chrono, uuid, thiserror
//! - **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
//! - **Internal-only**: `publish = false` - this is a contract crate for internal use
//!
//! # Example JSON Output
//!
//! ```json
//! {
//!   "schema_version": "1.0",
//!   "timestamp": "2025-01-26T12:00:00Z",
//!   "acs": [...]
//! }
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};

// ============================================================================
// Exit Codes
// ============================================================================

/// Exit codes for xtask commands.
///
/// These codes allow scripts and CI systems to programmatically
/// determine the outcome of a command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExitCode {
    /// Success
    Success = 0,
    /// General error
    Error = 1,
    /// Invalid arguments
    InvalidArguments = 2,
    /// File not found
    FileNotFound = 3,
    /// Validation error
    ValidationError = 4,
    /// Network error
    NetworkError = 5,
}

// ============================================================================
// Status Enums
// ============================================================================

/// AC status for coverage reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AcStatus {
    /// AC has passing tests
    Pass,
    /// AC has failing tests
    Fail,
    /// AC has no test coverage
    Unknown,
}

/// Question status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum QuestionStatus {
    /// Question is open
    Open,
    /// Question has been answered
    Answered,
    /// Question has been resolved
    Resolved,
    /// Question is obsolete
    Obsolete,
}

/// Friction status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FrictionStatus {
    /// Friction entry is open
    Open,
    /// Friction entry is being investigated
    Investigating,
    /// Friction entry is in progress
    InProgress,
    /// Friction entry is resolved
    Resolved,
    /// Friction entry won't be fixed
    WontFix,
}

// ============================================================================
// AC Coverage JSON DTOs
// ============================================================================

/// AC status JSON output schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcStatusJson {
    /// Schema version
    pub schema_version: String,
    /// Timestamp of report generation
    pub timestamp: String,
    /// AC summary
    pub summary: AcCategoryStats,
    /// AC details
    pub acs: Vec<AcJson>,
}

/// AC category statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcCategoryStats {
    /// Total ACs
    pub total: usize,
    /// Passing ACs
    pub passing: usize,
    /// Failing ACs
    pub failing: usize,
    /// Unknown ACs
    pub unknown: usize,
    /// Kernel ACs
    pub kernel: KernelStats,
    /// Optional ACs
    pub optional: OptionalStats,
}

/// Kernel AC statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KernelStats {
    /// Total kernel ACs
    pub total: usize,
    /// Passing kernel ACs
    pub passing: usize,
    /// Failing kernel ACs
    pub failing: usize,
    /// Unknown kernel ACs
    pub unknown: usize,
}

/// Optional AC statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionalStats {
    /// Total optional ACs
    pub total: usize,
    /// Passing optional ACs
    pub passing: usize,
    /// Failing optional ACs
    pub failing: usize,
    /// Unknown optional ACs
    pub unknown: usize,
}

/// AC JSON representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcJson {
    /// AC ID
    pub id: String,
    /// AC title
    pub title: String,
    /// AC status
    pub status: String,
    /// Story ID
    pub story_id: String,
    /// Requirement ID
    pub req_id: String,
    /// AC text
    pub text: String,
    /// Whether this is a kernel AC
    pub must_have_ac: bool,
    /// Source of status determination
    pub source: String,
    /// Scenarios
    pub scenarios: Vec<String>,
    /// Test mappings
    pub tests: Vec<TestMapping>,
    /// Tests executed
    pub tests_executed: usize,
    /// Tags
    pub tags: Vec<String>,
}

/// Test mapping from spec ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TestMapping {
    /// Test type (e.g., "unit", "bdd", "integration")
    pub test_type: String,
    /// Test tag or identifier
    pub tag: Option<String>,
}

// ============================================================================
// Issues Search JSON DTOs
// ============================================================================

/// Unified search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResult {
    /// Issue type: friction, question, or task
    pub issue_type: String,
    /// Unique ID
    pub id: String,
    /// Summary/title
    pub summary: String,
    /// Current status
    pub status: String,
    /// Related REQ/AC refs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<String>,
    /// Date (created_at or date field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Relevance score (for sorting)
    pub relevance_score: f32,
}

/// JSON output structure for issues search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchOutput {
    /// Search query
    pub query: String,
    /// Total results
    pub total_results: usize,
    /// Search results
    pub results: Vec<SearchResult>,
}

// ============================================================================
// Friction JSON DTOs
// ============================================================================

/// Friction entry for JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionEntry {
    /// Friction ID
    pub id: String,
    /// Date
    pub date: String,
    /// Category
    pub category: String,
    /// Severity
    pub severity: String,
    /// Summary
    pub summary: String,
    /// Description
    pub description: String,
    /// Expected behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,
    /// Workaround
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workaround: Option<String>,
    /// Impact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    /// Context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FrictionContext>,
    /// Status
    #[serde(default = "default_friction_status")]
    pub status: String,
    /// Resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    /// Related REQ/AC refs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<String>,
    /// Related items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

fn default_friction_status() -> String {
    "open".to_string()
}

/// Friction context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionContext {
    /// Who discovered the friction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_by: Option<String>,
    /// Flow name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    /// Phase name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Files involved
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub files_involved: Vec<String>,
    /// Commands involved
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub commands_involved: Vec<String>,
}

/// Resolution details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Resolution {
    /// Who resolved it
    pub resolved_by: String,
    /// When it was resolved
    pub resolved_at: String,
    /// Fix description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_description: Option<String>,
    /// PR links
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pr_links: Vec<String>,
    /// Verification notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
}

/// Related items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RelatedItems {
    /// Related issues
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<String>,
    /// Related ADRs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub adrs: Vec<String>,
    /// Related tasks
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tasks: Vec<String>,
}

/// Friction list JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionListJson {
    /// Timestamp
    pub timestamp: String,
    /// Total count
    pub total_count: usize,
    /// Statistics
    pub stats: FrictionStatsJson,
    /// Entries
    pub entries: Vec<FrictionEntry>,
}

/// Friction statistics JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionStatsJson {
    /// Open count
    pub open: usize,
    /// Investigating count
    pub investigating: usize,
    /// In progress count
    pub in_progress: usize,
    /// Resolved count
    pub resolved: usize,
    /// Wont fix count
    pub wont_fix: usize,
    /// Severity breakdown
    pub severity: FrictionSeverityStatsJson,
}

/// Friction severity statistics JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrictionSeverityStatsJson {
    /// Low severity count
    pub low: usize,
    /// Medium severity count
    pub medium: usize,
    /// High severity count
    pub high: usize,
    /// Critical severity count
    pub critical: usize,
}

// ============================================================================
// Question JSON DTOs
// ============================================================================

/// Question entry for JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Question {
    /// Question ID
    pub id: String,
    /// Task ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Requirement IDs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub req_ids: Vec<String>,
    /// AC IDs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ac_ids: Vec<String>,
    /// Related refs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<String>,
    /// Summary
    pub summary: String,
    /// Context
    pub context: QuestionContext,
    /// Options
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<QuestionOption>,
    /// Recommendation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<Recommendation>,
    /// Who created it
    pub created_by: String,
    /// When it was created
    pub created_at: String,
    /// Status
    #[serde(default = "default_question_status")]
    pub status: String,
    /// Resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
}

fn default_question_status() -> String {
    "open".to_string()
}

/// Question context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionContext {
    /// Flow name
    pub flow: String,
    /// Phase name
    pub phase: String,
    /// Description
    pub description: String,
    /// Files involved
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub files_involved: Vec<String>,
}

/// Question option.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionOption {
    /// Option label
    pub label: String,
    /// Option description
    pub description: String,
    /// Risk assessment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    /// Whether option is reversible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reversible: Option<bool>,
}

/// Recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Recommendation {
    /// Chosen option label
    pub option_label: String,
    /// Rationale
    pub rationale: String,
    /// Confidence level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

/// Question list JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionsListJson {
    /// Timestamp
    pub timestamp: String,
    /// Total count
    pub total_count: usize,
    /// Statistics
    pub stats: QuestionStatsJson,
    /// Questions
    pub questions: Vec<Question>,
}

/// Question statistics JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuestionStatsJson {
    /// Open count
    pub open: usize,
    /// Answered count
    pub answered: usize,
    /// Resolved count
    pub resolved: usize,
    /// Obsolete count
    pub obsolete: usize,
}

// ============================================================================
// Common JSON Response Envelope
// ============================================================================

/// Common JSON response envelope with timestamp and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JsonEnvelope<T> {
    /// Schema version
    pub schema_version: String,
    /// Timestamp of response generation
    pub timestamp: String,
    /// Response data
    pub data: T,
}

impl<T> JsonEnvelope<T> {
    /// Create a new JSON envelope.
    pub fn new(data: T, schema_version: impl Into<String>) -> Self {
        Self { schema_version: schema_version.into(), timestamp: Utc::now().to_rfc3339(), data }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::Error as i32, 1);
        assert_eq!(ExitCode::InvalidArguments as i32, 2);
    }

    #[test]
    fn test_ac_status_serialization() {
        let status = AcStatusJson {
            schema_version: "1.0".to_string(),
            timestamp: "2025-01-26T12:00:00Z".to_string(),
            summary: AcCategoryStats {
                total: 10,
                passing: 8,
                failing: 1,
                unknown: 1,
                kernel: KernelStats { total: 5, passing: 4, failing: 1, unknown: 0 },
                optional: OptionalStats { total: 5, passing: 4, failing: 0, unknown: 1 },
            },
            acs: vec![],
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("1.0"));
        assert!(json.contains("total"));
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            issue_type: "friction".to_string(),
            id: "FRICTION-001".to_string(),
            summary: "Test friction".to_string(),
            status: "open".to_string(),
            refs: vec!["REQ-001".to_string()],
            date: Some("2025-01-26".to_string()),
            relevance_score: 10.0,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("FRICTION-001"));
        assert!(json.contains("friction"));
    }

    #[test]
    fn test_json_envelope() {
        let data = vec!["item1", "item2"];
        let envelope = JsonEnvelope::new(data.clone(), "1.0");

        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("1.0"));
        assert!(json.contains("item1"));
        assert!(json.contains("item2"));
    }
}
