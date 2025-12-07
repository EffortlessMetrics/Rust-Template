//! Ledger parsing: Extract AC metadata from spec_ledger.yaml.
//!
//! This module provides functions to read and interpret the spec ledger,
//! extracting AC metadata including the `must_have_ac` flag.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata for an acceptance criterion.
#[derive(Debug, Clone)]
pub struct AcMetadata {
    /// The parent requirement ID
    pub req_id: String,
    /// Whether this AC must have BDD coverage (true = kernel, false = non-kernel)
    pub must_have_ac: bool,
}

/// Full details for an acceptance criterion (including text).
#[derive(Debug, Clone)]
pub struct AcDetails {
    /// The AC ID (e.g., "AC-TPL-001")
    pub id: String,
    /// Human-readable AC description text
    pub text: String,
    /// The parent story ID
    pub story_id: String,
    /// The parent requirement ID
    pub req_id: String,
    /// The requirement title
    pub req_title: String,
    /// Whether this AC must have BDD coverage (true = kernel, false = non-kernel)
    pub must_have_ac: bool,
}

// ============================================================================
// Internal YAML structures for parsing spec_ledger.yaml
// ============================================================================

#[derive(Debug, Deserialize)]
struct Ledger {
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    id: String,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
    /// Whether all ACs under this requirement must have BDD coverage.
    /// Defaults to true (kernel). Set to false for non-kernel/exploratory ACs.
    #[serde(default = "default_must_have_ac")]
    must_have_ac: bool,
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

/// Default for `must_have_ac` field: true (kernel AC).
///
/// This matches the AC classification semantics in ADR-0023.
fn default_must_have_ac() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    id: String,
    text: String,
    /// Whether this specific AC must have BDD coverage.
    /// Inherits from requirement if not specified, defaults to true.
    #[serde(default = "default_must_have_ac")]
    must_have_ac: bool,
}

// ============================================================================
// Public API
// ============================================================================

/// Parse the spec_ledger.yaml file and return all ACs with full metadata.
///
/// Returns `HashMap<AC_ID, AcMetadata>` containing:
/// - `req_id`: parent requirement ID
/// - `must_have_ac`: whether this AC must have BDD coverage (kernel AC)
///
/// # must_have_ac Semantics
///
/// The effective `must_have_ac` for an AC is computed using AND semantics:
/// - `effective = req.must_have_ac && ac.must_have_ac`
///
/// This means an AC is only considered a "must-have" kernel AC if both
/// the requirement AND the AC itself have `must_have_ac=true`.
pub fn parse_ledger_with_metadata(ledger_path: &Path) -> Result<HashMap<String, AcMetadata>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    let mut ac_metadata: HashMap<String, AcMetadata> = HashMap::new();

    for story in ledger.stories {
        for req in story.requirements {
            for ac in req.acceptance_criteria {
                // AC's must_have_ac is effective if both REQ and AC have it true
                let effective_must_have = req.must_have_ac && ac.must_have_ac;
                ac_metadata.insert(
                    ac.id.clone(),
                    AcMetadata { req_id: req.id.clone(), must_have_ac: effective_must_have },
                );
            }
        }
    }

    Ok(ac_metadata)
}

/// Look up a single AC by ID and return its full details.
///
/// Returns `None` if the AC ID is not found in the ledger.
pub fn get_ac_details(ledger_path: &Path, ac_id: &str) -> Result<Option<AcDetails>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    for story in ledger.stories {
        for req in story.requirements {
            for ac in req.acceptance_criteria {
                if ac.id == ac_id {
                    let effective_must_have = req.must_have_ac && ac.must_have_ac;
                    return Ok(Some(AcDetails {
                        id: ac.id,
                        text: ac.text,
                        story_id: story.id.clone(),
                        req_id: req.id.clone(),
                        req_title: req.title.clone(),
                        must_have_ac: effective_must_have,
                    }));
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_ledger_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn must_have_ac_defaults_to_true() {
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC without must_have_ac specified"
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(metadata.contains_key("AC-TEST-001"));
        assert!(
            metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should default to true when not specified"
        );
    }

    #[test]
    fn must_have_ac_and_semantics_both_true() {
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: true
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be true when both REQ and AC are true"
        );
    }

    #[test]
    fn must_have_ac_and_semantics_req_false() {
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: true
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            !metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be false when REQ is false (AND semantics)"
        );
    }

    #[test]
    fn must_have_ac_and_semantics_ac_false() {
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: false
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            !metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be false when AC is false (AND semantics)"
        );
    }

    #[test]
    fn get_ac_details_returns_full_info() {
        let content = r#"
stories:
  - id: US-STORY-001
    requirements:
      - id: REQ-PARENT-001
        title: "Parent Requirement"
        must_have_ac: true
        acceptance_criteria:
          - id: AC-CHILD-001
            text: "Child AC description"
            must_have_ac: true
"#;
        let file = write_ledger_file(content);
        let details = get_ac_details(file.path(), "AC-CHILD-001").unwrap().unwrap();

        assert_eq!(details.id, "AC-CHILD-001");
        assert_eq!(details.text, "Child AC description");
        assert_eq!(details.story_id, "US-STORY-001");
        assert_eq!(details.req_id, "REQ-PARENT-001");
        assert_eq!(details.req_title, "Parent Requirement");
        assert!(details.must_have_ac);
    }

    #[test]
    fn get_ac_details_returns_none_for_missing() {
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
"#;
        let file = write_ledger_file(content);
        let details = get_ac_details(file.path(), "AC-NONEXISTENT").unwrap();

        assert!(details.is_none());
    }
}
