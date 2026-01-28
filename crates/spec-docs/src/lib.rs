//! Documentation index and staleness tracking.
//!
//! This crate provides types and functions for managing documentation inventory,
//! staleness detection, and policy checking.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, serde, serde_yaml, thiserror, anyhow
//! - **View layer**: Depends on spec-ledger for validation
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//!
//! # Example
//!
//! ```ignore
//! use spec_docs::{load_doc_index, check_staleness};
//!
//! let docs = load_doc_index(Path::new("specs/doc_index.yaml"))?;
//! let stale = check_staleness(&docs, &ledger)?;
//! ```

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use spec_types::{SpecError, SpecResult};
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// Documentation index specification.
///
/// Root structure for `specs/doc_index.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocIndex {
    pub schema_version: String,
    pub template_version: String,
    pub docs: Vec<DocEntry>,
}

/// Single documentation entry in the index.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocEntry {
    pub id: String,
    pub file: String,
    pub doc_type: String,
    #[serde(default)]
    pub stories: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub acs: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
}

/// Documentation policy specification.
///
/// Root structure for `specs/doc_policies.yaml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocPolicies {
    pub schema_version: String,
    pub template_version: String,
    pub rules: Vec<PolicyRule>,
}

/// Policy rule for documentation requirements.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub applies_to: AppliesTo,
    pub require_doc_types: Vec<String>,
    pub min_docs: usize,
}

/// What a policy rule applies to.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppliesTo {
    pub requirement_tags: Vec<String>,
}

/// Staleness report for documentation.
#[derive(Debug, Clone)]
pub struct StalenessReport {
    pub stale_docs: Vec<StaleDoc>,
    pub missing_docs: Vec<MissingDoc>,
}

/// A stale document that needs updating.
#[derive(Debug, Clone)]
pub struct StaleDoc {
    pub doc_id: String,
    pub file: String,
    pub reason: String,
}

/// A missing document that should exist.
#[derive(Debug, Clone)]
pub struct MissingDoc {
    pub requirement_id: String,
    pub required_doc_types: Vec<String>,
}

// ============================================================================
// Loading
// ============================================================================

/// Load doc index from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to `doc_index.yaml`
///
/// # Returns
///
/// Returns a parsed [`DocIndex`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_doc_index(path: &Path) -> SpecResult<DocIndex> {
    let content = std::fs::read_to_string(path).map_err(SpecError::Io)?;

    serde_yaml::from_str(&content).map_err(|e| SpecError::YamlParse(e.to_string()))
}

/// Load doc policies from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to `doc_policies.yaml`
///
/// # Returns
///
/// Returns a parsed [`DocPolicies`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_policies(path: &Path) -> SpecResult<DocPolicies> {
    let content = std::fs::read_to_string(path).map_err(SpecError::Io)?;

    serde_yaml::from_str(&content).map_err(|e| SpecError::YamlParse(e.to_string()))
}

// ============================================================================
// Staleness Detection
// ============================================================================

/// Check documentation staleness against ledger.
///
/// Identifies:
/// - Stale docs (docs referencing non-existent requirements)
/// - Missing docs (requirements without required documentation)
///
/// # Arguments
///
/// * `docs` - Documentation index
/// * `ledger` - Spec ledger for validation
///
/// # Returns
///
/// Returns a [`StalenessReport`] with stale and missing docs.
pub fn check_staleness(
    docs: &DocIndex,
    ledger: &spec_ledger::SpecLedger,
) -> Result<StalenessReport, anyhow::Error> {
    let mut stale_docs = Vec::new();
    let mut missing_docs = Vec::new();

    // Build index of valid requirement IDs
    let valid_req_ids: std::collections::HashSet<_> =
        ledger.stories.iter().flat_map(|s| s.requirements.iter()).map(|r| r.id.as_str()).collect();

    // Check for stale docs (referencing non-existent requirements)
    for doc in &docs.docs {
        for req_id in &doc.requirements {
            if !valid_req_ids.contains(req_id.as_str()) {
                stale_docs.push(StaleDoc {
                    doc_id: doc.id.clone(),
                    file: doc.file.clone(),
                    reason: format!("References non-existent requirement {}", req_id),
                });
            }
        }
    }

    // Check for missing docs (requirements without required documentation)
    // This is a simple check - in a full implementation, we'd check
    // against doc policies
    for story in &ledger.stories {
        for req in &story.requirements {
            let has_doc = docs.docs.iter().any(|d| d.requirements.contains(&req.id));
            if !has_doc && !req.tags.is_empty() {
                // For tagged requirements, we might have specific doc requirements
                missing_docs.push(MissingDoc {
                    requirement_id: req.id.clone(),
                    required_doc_types: vec!["design_doc".to_string(), "impl_plan".to_string()],
                });
            }
        }
    }

    Ok(StalenessReport { stale_docs, missing_docs })
}

// ============================================================================
// Policy Checking
// ============================================================================

/// Check documentation against policies.
///
/// Validates that requirements with specific tags have required documentation.
///
/// # Arguments
///
/// * `docs` - Documentation index
/// * `policies` - Documentation policies
///
/// # Returns
///
/// Returns a [`StalenessReport`] with violations.
pub fn check_policies(
    docs: &DocIndex,
    policies: &DocPolicies,
) -> Result<StalenessReport, anyhow::Error> {
    let mut missing_docs = Vec::new();

    for rule in &policies.rules {
        // Find requirements matching the rule
        let matching_reqs: Vec<_> = docs
            .docs
            .iter()
            .filter(|d| {
                d.requirements.iter().any(|_req_id| {
                    // In a full implementation, we'd look up the requirement
                    // and check its tags against rule.applies_to.requirement_tags
                    true // Simplified for now
                })
            })
            .collect();

        // Check if matching requirements have required doc types
        for doc in &matching_reqs {
            for required_type in &rule.require_doc_types {
                if !doc.doc_type.contains(required_type) {
                    missing_docs.push(MissingDoc {
                        requirement_id: doc
                            .requirements
                            .first()
                            .unwrap_or(&"unknown".to_string())
                            .clone(),
                        required_doc_types: vec![required_type.clone()],
                    });
                }
            }
        }
    }

    Ok(StalenessReport { stale_docs: vec![], missing_docs })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_doc_index() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
docs:
  - id: "DOC-TEST-001"
    file: "docs/test.md"
    doc_type: "design_doc"
    requirements: ["REQ-TEST-001"]
    acs: []
    adrs: []
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let docs = load_doc_index(temp.path()).unwrap();
        assert_eq!(docs.docs.len(), 1);
        assert_eq!(docs.docs[0].id, "DOC-TEST-001");
    }

    #[test]
    fn test_load_policies() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
rules:
  - id: "POLICY-001"
    description: "Test policy"
    applies_to:
      requirement_tags: ["security"]
    require_doc_types: ["design_doc", "threat_model"]
    min_docs: 2
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let policies = load_policies(temp.path()).unwrap();
        assert_eq!(policies.rules.len(), 1);
        assert_eq!(policies.rules[0].id, "POLICY-001");
    }

    #[test]
    fn test_check_staleness() {
        let docs = DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            docs: vec![
                DocEntry {
                    id: "DOC-001".to_string(),
                    file: "docs/valid.md".to_string(),
                    doc_type: "design_doc".to_string(),
                    stories: vec![],
                    requirements: vec!["REQ-001".to_string()],
                    acs: vec![],
                    adrs: vec![],
                },
                DocEntry {
                    id: "DOC-002".to_string(),
                    file: "docs/stale.md".to_string(),
                    doc_type: "design_doc".to_string(),
                    stories: vec![],
                    requirements: vec!["REQ-NONEXISTENT".to_string()],
                    acs: vec![],
                    adrs: vec![],
                },
            ],
        };

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
                    title: "Test".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![],
                }],
            }],
        };

        let report = check_staleness(&docs, &ledger).unwrap();
        assert_eq!(report.stale_docs.len(), 1);
        assert_eq!(report.stale_docs[0].doc_id, "DOC-002");
        assert!(report.stale_docs[0].reason.contains("REQ-NONEXISTENT"));
    }
}
