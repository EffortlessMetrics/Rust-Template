//! Governance spec ledger (stories, requirements, acceptance criteria).
//!
//! This module defines the structure of `spec_ledger.yaml`, the core governance document
//! that maps user stories → requirements → acceptance criteria → tests.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
pub fn load_spec_ledger(path: &Path) -> Result<SpecLedger> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read spec ledger: {}", path.display()))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse spec ledger: {}", path.display()))
}
