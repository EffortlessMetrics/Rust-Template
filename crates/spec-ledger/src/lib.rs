//! Governance spec ledger (stories, requirements, acceptance criteria).
//!
//! This crate provides parsing, loading, and indexing of the spec ledger
//! from `spec_ledger.yaml`. It serves as the foundation for all other
//! spec-* crates.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, serde, serde_yaml, thiserror, anyhow
//! - **Foundation layer**: Other spec-* crates depend on this, not vice versa
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//! - **No axum**: HTTP/web dependencies are isolated to higher-level crates
//!
//! # Example
//!
//! ```ignore
//! use spec_ledger::{load_spec_ledger, SpecLedger};
//!
//! let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
//! println!("Loaded {} stories", ledger.stories.len());
//! ```

use serde::{Deserialize, Serialize};
use spec_types::{SpecError, SpecResult};
use std::collections::HashSet;
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// Spec ledger containing all governance stories and requirements.
///
/// The root structure of `specs/spec_ledger.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpecLedger {
    /// Metadata about the spec ledger (version, last updated, description).
    pub metadata: Metadata,
    /// List of all user stories in the ledger.
    pub stories: Vec<Story>,
}

/// Metadata for the spec ledger.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metadata {
    /// Schema version for spec_ledger.yaml.
    pub schema_version: String,
    /// Template version this ledger is compatible with.
    pub template_version: String,
    /// ISO 8601 timestamp of last update.
    pub last_updated: String,
    /// Human-readable description of this ledger.
    pub description: String,
}

/// A user story with associated requirements.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Story {
    /// Unique story ID (e.g., "US-TPL-001").
    pub id: String,
    /// Title of the story.
    pub title: String,
    /// List of requirements for this story.
    pub requirements: Vec<Requirement>,
}

/// A requirement with acceptance criteria.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Requirement {
    /// Unique requirement ID (e.g., "REQ-TPL-CONFIG").
    pub id: String,
    /// Title of the requirement.
    pub title: String,
    /// Optional tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether this requirement must have at least one acceptance criterion (default: true).
    #[serde(default = "default_must_have_ac")]
    pub must_have_ac: bool,
    /// List of acceptance criteria for this requirement.
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

fn default_must_have_ac() -> bool {
    true
}

/// Mapping from an AC to a test.
///
/// Describes how a test relates to an acceptance criterion.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestMapping {
    /// Type of test (e.g., "bdd", "unit", "integration").
    #[serde(rename = "type")]
    pub test_type: String,
    /// Test tag or scenario identifier.
    pub tag: String,
    /// Optional file path where the test is defined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Optional module path for the test.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub module: Option<String>,
}

/// An acceptance criterion with test mappings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AcceptanceCriterion {
    /// Unique AC ID (e.g., "AC-TPL-CONFIG-001").
    pub id: String,
    /// Description of what this AC validates.
    pub text: String,
    /// List of tests that cover this AC.
    #[serde(default)]
    pub tests: Vec<TestMapping>,
}

// ============================================================================
// Loading
// ============================================================================

/// Load the spec ledger from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to `spec_ledger.yaml`
///
/// # Returns
///
/// Returns a parsed [`SpecLedger`] instance.
///
/// # Errors
///
/// Returns an error if the file is missing, unreadable, or malformed YAML.
///
/// # Example
///
/// ```ignore
/// let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
/// println!("Loaded {} stories", ledger.stories.len());
/// ```
pub fn load_spec_ledger(path: &Path) -> SpecResult<SpecLedger> {
    let content = std::fs::read_to_string(path).map_err(SpecError::Io)?;

    serde_yaml::from_str(&content).map_err(|e| SpecError::YamlParse(e.to_string()))
}

// ============================================================================
// Indexing
// ============================================================================

/// Index of all AC IDs in the spec ledger for fast lookup.
/// Used for referential integrity validation in agent hints and bundles.
pub type AcIdIndex = HashSet<String>;

/// Index of all REQ IDs in the spec ledger for fast lookup.
/// Used for referential integrity validation in agent hints and bundles.
pub type ReqIdIndex = HashSet<String>;

/// Build an index of all AC IDs from a SpecLedger.
///
/// # Example
///
/// ```ignore
/// let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
/// let ac_ids = build_ac_id_index(&ledger);
/// assert!(ac_ids.contains("AC-TPL-001"));
/// ```
pub fn build_ac_id_index(ledger: &SpecLedger) -> AcIdIndex {
    let mut index = AcIdIndex::new();
    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                index.insert(ac.id.clone());
            }
        }
    }
    index
}

/// Build an index of all REQ IDs from a SpecLedger.
///
/// # Example
///
/// ```ignore
/// let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
/// let req_ids = build_req_id_index(&ledger);
/// assert!(req_ids.contains("REQ-TPL-HEALTH"));
/// ```
pub fn build_req_id_index(ledger: &SpecLedger) -> ReqIdIndex {
    let mut index = ReqIdIndex::new();
    for story in &ledger.stories {
        for req in &story.requirements {
            index.insert(req.id.clone());
        }
    }
    index
}

// ============================================================================
// Validation
// ============================================================================

/// Validate ledger-specific invariants.
///
/// Checks:
/// - All story IDs are unique
/// - All requirement IDs are unique
/// - All AC IDs are unique
/// - Requirements with must_have_ac=true have at least one AC
pub fn validate_ledger(ledger: &SpecLedger) -> SpecResult<()> {
    // Check unique story IDs
    let mut story_ids = HashSet::new();
    for story in &ledger.stories {
        if !story_ids.insert(&story.id) {
            return Err(SpecError::Validation(format!("Duplicate story ID: {}", story.id)));
        }
    }

    // Check unique requirement IDs
    let mut req_ids = HashSet::new();
    let mut ac_ids = HashSet::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            if !req_ids.insert(&req.id) {
                return Err(SpecError::Validation(format!("Duplicate requirement ID: {}", req.id)));
            }

            // Check must_have_ac invariant
            if req.must_have_ac && req.acceptance_criteria.is_empty() {
                return Err(SpecError::Validation(format!(
                    "Requirement {} has must_have_ac=true but no acceptance criteria",
                    req.id
                )));
            }

            for ac in &req.acceptance_criteria {
                if !ac_ids.insert(&ac.id) {
                    return Err(SpecError::Validation(format!("Duplicate AC ID: {}", ac.id)));
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_spec_ledger() {
        let yaml = r#"
metadata:
  schema_version: "1.0"
  template_version: "3.3.1"
  last_updated: "2025-01-01"
  description: "Test ledger"
stories:
  - id: "US-TEST-001"
    title: "Test Story"
    requirements:
      - id: "REQ-TEST-001"
        title: "Test Requirement"
        must_have_ac: true
        acceptance_criteria:
          - id: "AC-TEST-001"
            text: "Test AC"
            tests:
              - type: "unit"
                tag: "test_tag"
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let ledger = load_spec_ledger(temp.path()).unwrap();
        assert_eq!(ledger.stories.len(), 1);
        assert_eq!(ledger.stories[0].id, "US-TEST-001");
    }

    #[test]
    fn test_build_ac_id_index() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-001".to_string(),
                    title: "Test".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![
                        AcceptanceCriterion {
                            id: "AC-001".to_string(),
                            text: "Test 1".to_string(),
                            tests: vec![],
                        },
                        AcceptanceCriterion {
                            id: "AC-002".to_string(),
                            text: "Test 2".to_string(),
                            tests: vec![],
                        },
                    ],
                }],
            }],
        };

        let index = build_ac_id_index(&ledger);
        assert_eq!(index.len(), 2);
        assert!(index.contains("AC-001"));
        assert!(index.contains("AC-002"));
    }

    #[test]
    fn test_build_req_id_index() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![
                    Requirement {
                        id: "REQ-001".to_string(),
                        title: "Test 1".to_string(),
                        tags: vec![],
                        must_have_ac: true,
                        acceptance_criteria: vec![],
                    },
                    Requirement {
                        id: "REQ-002".to_string(),
                        title: "Test 2".to_string(),
                        tags: vec![],
                        must_have_ac: true,
                        acceptance_criteria: vec![],
                    },
                ],
            }],
        };

        let index = build_req_id_index(&ledger);
        assert_eq!(index.len(), 2);
        assert!(index.contains("REQ-001"));
        assert!(index.contains("REQ-002"));
    }

    #[test]
    fn test_validate_ledger_duplicate_story() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![
                Story {
                    id: "US-001".to_string(),
                    title: "Test 1".to_string(),
                    requirements: vec![],
                },
                Story {
                    id: "US-001".to_string(),
                    title: "Test 2".to_string(),
                    requirements: vec![],
                },
            ],
        };

        let result = validate_ledger(&ledger);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate story ID"));
    }

    #[test]
    fn test_validate_ledger_must_have_ac() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-001".to_string(),
                    title: "Test".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![],
                }],
            }],
        };

        let result = validate_ledger(&ledger);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must_have_ac=true"));
    }
}
