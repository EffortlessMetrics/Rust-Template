//! Spec metadata extraction and management.
//!
//! This crate provides types and functions for managing spec metadata,
//! metadata extraction, and metadata validation.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
//! - **Utility layer**: Provides metadata types and extraction helpers
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//!
//! # Example
//!
//! ```ignore
//! use spec_metadata::{load_metadata, extract_tags};
//!
//! let metadata = load_metadata(Path::new("specs/metadata.yaml"))?;
//! let tags = extract_tags(&metadata);
//! ```

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use spec_ledger::SpecLedger;
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// Spec metadata specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpecMetadata {
    pub schema_version: String,
    pub template_version: String,
    pub ledger: LedgerMetadata,
    pub tags: TagsMetadata,
    pub documentation: DocumentationMetadata,
}

/// Ledger metadata.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LedgerMetadata {
    pub last_updated: String,
    pub story_count: usize,
    pub requirement_count: usize,
    pub ac_count: usize,
}

/// Tags metadata.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagsMetadata {
    pub categories: Vec<String>,
    pub tag_counts: HashMap<String, usize>,
}

/// Documentation metadata.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentationMetadata {
    pub doc_count: usize,
    pub doc_types: Vec<String>,
    pub stale_docs: Vec<String>,
}

/// Tag extraction result.
#[derive(Debug, Clone)]
pub struct TagExtraction {
    pub tags: Vec<String>,
    pub by_category: HashMap<String, Vec<String>>,
}

/// Metadata validation result.
#[derive(Debug, Clone)]
pub struct MetadataValidationResult {
    pub valid: bool,
    pub errors: Vec<MetadataValidationError>,
}

/// Metadata validation error.
#[derive(Debug, Clone)]
pub struct MetadataValidationError {
    pub field: String,
    pub error_type: String,
    pub message: String,
}

// ============================================================================
// Loading
// ============================================================================

/// Load spec metadata from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to metadata YAML file
///
/// # Returns
///
/// Returns a parsed [`SpecMetadata`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_metadata(path: &Path) -> Result<SpecMetadata, anyhow::Error> {
    let content = std::fs::read_to_string(path)?;

    serde_yaml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse metadata: {}", e))
}

// ============================================================================
// Extraction
// ============================================================================

/// Extract tags from spec ledger.
///
/// # Arguments
///
/// * `ledger` - Spec ledger to extract tags from
///
/// # Returns
///
/// Returns a [`TagExtraction`] with all tags and category mappings.
pub fn extract_tags(ledger: &SpecLedger) -> TagExtraction {
    let mut tags = Vec::new();
    let mut by_category: HashMap<String, Vec<String>> = HashMap::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            for tag in &req.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
    }

    // Categorize tags (simple heuristic)
    for tag in &tags {
        let category = categorize_tag(tag);
        by_category.entry(category).or_default().push(tag.clone());
    }

    TagExtraction { tags, by_category }
}

/// Categorize a tag (simple heuristic).
fn categorize_tag(tag: &str) -> String {
    match tag.to_lowercase().as_str() {
        tag if tag.starts_with("security") => "security".to_string(),
        tag if tag.starts_with("perf") => "performance".to_string(),
        tag if tag.starts_with("governance") => "governance".to_string(),
        tag if tag.starts_with("infra") => "infrastructure".to_string(),
        _ => "general".to_string(),
    }
}

/// Build ledger metadata from spec ledger.
///
/// # Arguments
///
/// * `ledger` - Spec ledger to build metadata from
///
/// # Returns
///
/// Returns [`LedgerMetadata`] with counts and timestamps.
pub fn build_ledger_metadata(ledger: &SpecLedger) -> LedgerMetadata {
    let story_count = ledger.stories.len();
    let requirement_count = ledger.stories.iter().map(|s| s.requirements.len()).sum();
    let ac_count = ledger
        .stories
        .iter()
        .flat_map(|s| s.requirements.iter())
        .map(|r| r.acceptance_criteria.len())
        .sum();

    let last_updated = ledger.metadata.last_updated.clone();

    LedgerMetadata { last_updated, story_count, requirement_count, ac_count }
}

// ============================================================================
// Validation
// ============================================================================

/// Validate spec metadata.
///
/// Checks:
/// - Schema version is valid
/// - Template version is present
/// - Ledger metadata is consistent
///
/// # Arguments
///
/// * `metadata` - Spec metadata to validate
/// * `ledger` - Spec ledger for consistency checks
///
/// # Returns
///
/// Returns a [`MetadataValidationResult`] with validation status and errors.
pub fn validate_metadata(metadata: &SpecMetadata, ledger: &SpecLedger) -> MetadataValidationResult {
    let mut errors = Vec::new();

    // Validate schema version
    if !metadata.schema_version.starts_with('1') {
        errors.push(MetadataValidationError {
            field: "schema_version".to_string(),
            error_type: "invalid_format".to_string(),
            message: "Schema version must start with '1'".to_string(),
        });
    }

    // Validate template version is present
    if metadata.template_version.is_empty() {
        errors.push(MetadataValidationError {
            field: "template_version".to_string(),
            error_type: "missing".to_string(),
            message: "Template version is required".to_string(),
        });
    }

    // Validate ledger metadata consistency
    let calculated_counts = build_ledger_metadata(ledger);
    if metadata.ledger.story_count != calculated_counts.story_count {
        errors.push(MetadataValidationError {
            field: "story_count".to_string(),
            error_type: "mismatch".to_string(),
            message: format!(
                "Story count mismatch: metadata={}, calculated={}",
                metadata.ledger.story_count, calculated_counts.story_count
            ),
        });
    }

    if metadata.ledger.requirement_count != calculated_counts.requirement_count {
        errors.push(MetadataValidationError {
            field: "requirement_count".to_string(),
            error_type: "mismatch".to_string(),
            message: format!(
                "Requirement count mismatch: metadata={}, calculated={}",
                metadata.ledger.requirement_count, calculated_counts.requirement_count
            ),
        });
    }

    if metadata.ledger.ac_count != calculated_counts.ac_count {
        errors.push(MetadataValidationError {
            field: "ac_count".to_string(),
            error_type: "mismatch".to_string(),
            message: format!(
                "AC count mismatch: metadata={}, calculated={}",
                metadata.ledger.ac_count, calculated_counts.ac_count
            ),
        });
    }

    MetadataValidationResult { valid: errors.is_empty(), errors }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_metadata() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
ledger:
  last_updated: "2025-01-01"
  story_count: 10
  requirement_count: 25
  ac_count: 50
tags:
  categories: ["security", "governance", "performance"]
  tag_counts:
    security: 5
    governance: 3
    performance: 2
documentation:
  doc_count: 15
  doc_types: ["design_doc", "impl_plan", "runbook"]
  stale_docs: []
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let metadata = load_metadata(temp.path()).unwrap();
        assert_eq!(metadata.schema_version, "1.0");
        assert_eq!(metadata.ledger.story_count, 10);
        assert_eq!(metadata.tags.categories.len(), 3);
    }

    #[test]
    fn test_extract_tags() {
        let ledger = spec_ledger::SpecLedger {
            metadata: spec_ledger::Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![spec_ledger::Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![spec_ledger::Requirement {
                    id: "REQ-001".to_string(),
                    title: "Test 1".to_string(),
                    tags: vec!["security".to_string(), "performance".to_string()],
                    must_have_ac: true,
                    acceptance_criteria: vec![],
                }],
            }],
        };

        let extraction = extract_tags(&ledger);
        assert_eq!(extraction.tags.len(), 2);
        assert!(extraction.by_category.contains_key("security"));
        assert_eq!(extraction.by_category.get("security").unwrap().len(), 1);
        assert_eq!(extraction.by_category.get("performance").unwrap().len(), 1);
    }

    #[test]
    fn test_build_ledger_metadata() {
        let ledger = spec_ledger::SpecLedger {
            metadata: spec_ledger::Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![spec_ledger::Story {
                id: "US-001".to_string(),
                title: "Test".to_string(),
                requirements: vec![
                    spec_ledger::Requirement {
                        id: "REQ-001".to_string(),
                        title: "Test 1".to_string(),
                        tags: vec![],
                        must_have_ac: true,
                        acceptance_criteria: vec![],
                    },
                    spec_ledger::Requirement {
                        id: "REQ-002".to_string(),
                        title: "Test 2".to_string(),
                        tags: vec![],
                        must_have_ac: true,
                        acceptance_criteria: vec![],
                    },
                ],
            }],
        };

        let metadata = build_ledger_metadata(&ledger);
        assert_eq!(metadata.story_count, 1);
        assert_eq!(metadata.requirement_count, 2);
        assert_eq!(metadata.ac_count, 0);
    }

    #[test]
    fn test_validate_metadata() {
        let ledger = spec_ledger::SpecLedger {
            metadata: spec_ledger::Metadata {
                schema_version: "1.0".to_string(),
                template_version: "1.0".to_string(),
                last_updated: "2025-01-01".to_string(),
                description: "Test".to_string(),
            },
            stories: vec![],
        };

        let metadata = SpecMetadata {
            schema_version: "invalid".to_string(),
            template_version: "1.0".to_string(),
            ledger: build_ledger_metadata(&ledger),
            tags: TagsMetadata { categories: vec![], tag_counts: HashMap::new() },
            documentation: DocumentationMetadata {
                doc_count: 0,
                doc_types: vec![],
                stale_docs: vec![],
            },
        };

        let result = validate_metadata(&metadata, &ledger);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].field, "schema_version");
    }
}
