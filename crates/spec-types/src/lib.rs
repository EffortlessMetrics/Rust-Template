//! Spec types for ledger/graph/hints/tasks.
//!
//! This crate defines stable, dependency-light types for spec-related operations.
//! It contains:
//! - ID newtypes for Story, Requirement, AC
//! - Shared structs (Story, Requirement, AcceptanceCriterion)
//! - Path types for spec resolution
//! - Common error types for spec operations
//!
//! # Design Philosophy
//!
//! - **Minimal dependencies**: Only serde, serde_yaml, thiserror
//! - **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
//! - **Internal-only**: `publish = false` - this is a contract crate for internal use
//!
//! # Example Usage
//!
//! ```rust
//! use spec_types::{StoryId, RequirementId, AcId};
//!
//! let story_id = StoryId::new("US-PLT-001");
//! let req_id = RequirementId::new("REQ-PLT-001");
//! let ac_id = AcId::new("AC-PLT-001");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ID Newtypes
// ============================================================================

/// Story ID newtype.
///
/// Provides type safety for story identifiers throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoryId(pub String);

impl StoryId {
    /// Create a new StoryId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for StoryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Requirement ID newtype.
///
/// Provides type safety for requirement identifiers throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RequirementId(pub String);

impl RequirementId {
    /// Create a new RequirementId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RequirementId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// AC (Acceptance Criterion) ID newtype.
///
/// Provides type safety for AC identifiers throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcId(pub String);

impl AcId {
    /// Create a new AcId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for AcId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Task ID newtype.
///
/// Provides type safety for task identifiers throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskId(pub String);

impl TaskId {
    /// Create a new TaskId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TaskId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Shared Structs
// ============================================================================

/// Story from spec ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Story {
    /// Story ID
    pub id: String,
    /// Story title
    pub title: String,
    /// Requirements
    pub requirements: Vec<Requirement>,
}

/// Requirement from spec ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Requirement {
    /// Requirement ID
    pub id: String,
    /// Requirement title
    pub title: String,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Whether this requirement has must_have_ac enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_have_ac: Option<bool>,
}

/// Acceptance criterion from spec ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AcceptanceCriterion {
    /// AC ID
    pub id: String,
    /// AC text
    pub text: String,
    /// Whether this AC has must_have_ac enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_have_ac: Option<bool>,
    /// Test mappings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tests: Vec<TestMapping>,
    /// Tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

/// Test mapping from spec ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TestMapping {
    /// Test type (e.g., "unit", "bdd", "integration")
    pub test_type: String,
    /// Test tag or identifier
    pub tag: Option<String>,
}

/// Task from spec ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Task summary
    pub summary: String,
    /// Task status
    pub status: String,
    /// Requirement ID
    pub requirement: String,
    /// AC IDs
    pub acs: Vec<String>,
    /// Task owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Labels
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels: Vec<String>,
}

// ============================================================================
// Path Types
// ============================================================================

/// Path type for spec resolution.
///
/// Represents a file path within the specs directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpecPath(pub String);

impl SpecPath {
    /// Create a new SpecPath.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpecPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SpecPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Error type for spec operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SpecError {
    /// IO error during spec operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error.
    #[error("Failed to parse YAML: {0}")]
    YamlParse(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Spec not found.
    #[error("Spec not found: {0}")]
    NotFound(String),

    /// Invalid spec format.
    #[error("Invalid spec format: {0}")]
    InvalidFormat(String),
}

/// Result type for spec operations.
pub type SpecResult<T> = Result<T, SpecError>;

// ============================================================================
// Ledger Metadata
// ============================================================================

/// Ledger metadata from spec_ledger.yaml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerMetadata {
    /// Template version
    pub template_version: String,
}

/// Spec ledger structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpecLedger {
    /// Ledger metadata
    pub metadata: LedgerMetadata,
    /// Stories
    pub stories: Vec<Story>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_id_display() {
        let id = StoryId::new("US-PLT-001");
        assert_eq!(id.to_string(), "US-PLT-001");
        assert_eq!(id.as_str(), "US-PLT-001");
    }

    #[test]
    fn test_requirement_id_display() {
        let id = RequirementId::new("REQ-PLT-001");
        assert_eq!(id.to_string(), "REQ-PLT-001");
        assert_eq!(id.as_str(), "REQ-PLT-001");
    }

    #[test]
    fn test_ac_id_display() {
        let id = AcId::new("AC-PLT-001");
        assert_eq!(id.to_string(), "AC-PLT-001");
        assert_eq!(id.as_str(), "AC-PLT-001");
    }

    #[test]
    fn test_spec_path_display() {
        let path = SpecPath::new("specs/spec_ledger.yaml");
        assert_eq!(path.to_string(), "specs/spec_ledger.yaml");
        assert_eq!(path.as_str(), "specs/spec_ledger.yaml");
    }

    #[test]
    fn test_story_serialization() {
        let story = Story {
            id: "US-001".to_string(),
            title: "Test Story".to_string(),
            requirements: vec![],
        };

        let yaml = serde_yaml::to_string(&story).unwrap();
        assert!(yaml.contains("US-001"));
        assert!(yaml.contains("Test Story"));
    }

    #[test]
    fn test_spec_ledger_serialization() {
        let ledger = SpecLedger {
            metadata: LedgerMetadata { template_version: "1.0".to_string() },
            stories: vec![],
        };

        let yaml = serde_yaml::to_string(&ledger).unwrap();
        assert!(yaml.contains("template_version"));
        assert!(yaml.contains("1.0"));
    }

    #[test]
    fn test_acceptance_criterion_serialization() {
        let ac = AcceptanceCriterion {
            id: "AC-001".to_string(),
            text: "Test AC".to_string(),
            must_have_ac: Some(true),
            tests: vec![],
            tags: vec![],
        };

        let yaml = serde_yaml::to_string(&ac).unwrap();
        assert!(yaml.contains("AC-001"));
        assert!(yaml.contains("Test AC"));
    }

    #[test]
    fn test_spec_error_display() {
        let err = SpecError::NotFound("test.yaml".to_string());
        assert!(err.to_string().contains("test.yaml"));
    }
}
