//! Lint the spec_ledger.yaml for structural integrity and invariant violations.
//!
//! This command validates invariants that the spec ledger should maintain:
//!
//! - Every AC has a valid req_id and story link
//! - If `must_have_ac: true` (on both REQ and AC), the AC must have at least one test mapping
//! - Every test mapping has a known type (unit, bdd, integration, docs, manual)
//! - Test files referenced actually exist
//! - No dangling ACs (ACs not referenced by any REQ)
//! - No dangling REQs (REQs that don't have any ACs)
//!
//! Per ADR-0024, this provides a validation layer to prevent the spec ledger from
//! silently rotting as people add new content.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::kernel::layout_for_repo;

/// Known automated test types
pub const AUTOMATED_TEST_TYPES: &[&str] = &["unit", "bdd", "integration"];

/// Known test types (including non-automated)
pub const KNOWN_TEST_TYPES: &[&str] = &["unit", "bdd", "integration", "docs", "manual", "ci"];

/// Arguments for the ac-lint command
#[derive(Debug, Clone, Default)]
pub struct AcLintArgs {
    /// Show verbose output
    pub verbose: bool,
    /// Return non-zero exit code on any warning (default: only errors fail)
    pub strict: bool,
    /// Check that test files exist on disk
    pub check_files: bool,
}

/// A lint finding
#[derive(Debug, Clone)]
pub struct LintFinding {
    pub level: LintLevel,
    pub category: &'static str,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintLevel {
    Error,
    Warning,
}

impl std::fmt::Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevel::Error => write!(f, "error"),
            LintLevel::Warning => write!(f, "warning"),
        }
    }
}

/// Result of linting the spec ledger
#[derive(Debug, Default)]
pub struct LintResult {
    pub findings: Vec<LintFinding>,
    pub stories_checked: usize,
    pub requirements_checked: usize,
    pub acs_checked: usize,
    pub tests_checked: usize,
}

impl LintResult {
    pub fn errors(&self) -> impl Iterator<Item = &LintFinding> {
        self.findings.iter().filter(|f| f.level == LintLevel::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &LintFinding> {
        self.findings.iter().filter(|f| f.level == LintLevel::Warning)
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.level == LintLevel::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|f| f.level == LintLevel::Warning)
    }
}

/// Run the ac-lint check
pub fn run(args: AcLintArgs) -> Result<()> {
    let layout = layout_for_repo();
    let result = lint_ledger(&layout.ledger, &args)?;

    // Print results
    println!();
    println!("{}", "Spec Ledger Lint".bold());
    println!("{}", "═".repeat(40));
    println!();
    println!(
        "  Checked: {} stories, {} requirements, {} ACs, {} test mappings",
        result.stories_checked.to_string().cyan(),
        result.requirements_checked.to_string().cyan(),
        result.acs_checked.to_string().cyan(),
        result.tests_checked.to_string().cyan(),
    );
    println!();

    if !result.has_errors() && !result.has_warnings() {
        println!("{}", "✓ No issues found".green().bold());
        return Ok(());
    }

    let error_count = result.errors().count();
    let warning_count = result.warnings().count();

    // Group findings by category
    let mut by_category: HashMap<&str, Vec<&LintFinding>> = HashMap::new();
    for finding in &result.findings {
        by_category.entry(finding.category).or_default().push(finding);
    }

    for (category, findings) in by_category.iter() {
        println!("{}", format!("{}:", category).bold());
        for finding in findings {
            let icon = match finding.level {
                LintLevel::Error => "✗".red(),
                LintLevel::Warning => "⚠".yellow(),
            };
            let level_str = match finding.level {
                LintLevel::Error => format!("[{}]", finding.level).red(),
                LintLevel::Warning => format!("[{}]", finding.level).yellow(),
            };
            if let Some(ref loc) = finding.location {
                println!("  {} {} {} (at {})", icon, level_str, finding.message, loc.dimmed());
            } else {
                println!("  {} {} {}", icon, level_str, finding.message);
            }
        }
        println!();
    }

    // Summary
    println!(
        "Found {} error(s) and {} warning(s)",
        error_count.to_string().red(),
        warning_count.to_string().yellow()
    );

    if error_count > 0 || (args.strict && warning_count > 0) {
        anyhow::bail!(
            "Lint failed with {} error(s){}",
            error_count,
            if args.strict && warning_count > 0 {
                format!(" and {} warning(s) (strict mode)", warning_count)
            } else {
                String::new()
            }
        );
    }

    Ok(())
}

/// Lint the spec ledger file
pub fn lint_ledger(ledger_path: &Path, args: &AcLintArgs) -> Result<LintResult> {
    #[derive(Debug, Deserialize)]
    struct Ledger {
        #[serde(default)]
        stories: Vec<Story>,
    }

    #[derive(Debug, Deserialize)]
    struct Story {
        id: String,
        #[serde(default)]
        requirements: Vec<Requirement>,
    }

    #[derive(Debug, Deserialize)]
    struct Requirement {
        id: String,
        #[serde(default = "default_true")]
        must_have_ac: bool,
        #[serde(default)]
        acceptance_criteria: Vec<AcceptanceCriteria>,
    }

    #[derive(Debug, Deserialize)]
    struct AcceptanceCriteria {
        id: String,
        #[serde(default)]
        text: String,
        #[serde(default = "default_true")]
        must_have_ac: bool,
        #[serde(default)]
        tests: Vec<TestMapping>,
    }

    #[derive(Debug, Deserialize)]
    struct TestMapping {
        #[serde(rename = "type")]
        test_type: String,
        #[serde(default)]
        tag: String,
        #[serde(default)]
        file: Option<String>,
        #[serde(default)]
        module: Option<String>,
    }

    fn default_true() -> bool {
        true
    }

    let mut result = LintResult::default();

    // Parse ledger
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;
    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger: {}", ledger_path.display()))?;

    // Track IDs for uniqueness checks
    let mut seen_story_ids: HashSet<String> = HashSet::new();
    let mut seen_req_ids: HashSet<String> = HashSet::new();
    let mut seen_ac_ids: HashSet<String> = HashSet::new();

    // Lint each story
    for story in &ledger.stories {
        result.stories_checked += 1;

        // Check for duplicate story ID
        if !seen_story_ids.insert(story.id.clone()) {
            result.findings.push(LintFinding {
                level: LintLevel::Error,
                category: "Duplicate IDs",
                message: format!("Duplicate story ID: {}", story.id),
                location: Some(story.id.clone()),
            });
        }

        // Check story ID format
        if !story.id.starts_with("US-") {
            result.findings.push(LintFinding {
                level: LintLevel::Warning,
                category: "Naming Conventions",
                message: format!("Story ID '{}' should start with 'US-'", story.id),
                location: Some(story.id.clone()),
            });
        }

        // Check for empty requirements
        if story.requirements.is_empty() {
            result.findings.push(LintFinding {
                level: LintLevel::Warning,
                category: "Structure",
                message: format!("Story '{}' has no requirements", story.id),
                location: Some(story.id.clone()),
            });
        }

        // Lint each requirement
        for req in &story.requirements {
            result.requirements_checked += 1;

            // Check for duplicate requirement ID
            if !seen_req_ids.insert(req.id.clone()) {
                result.findings.push(LintFinding {
                    level: LintLevel::Error,
                    category: "Duplicate IDs",
                    message: format!("Duplicate requirement ID: {}", req.id),
                    location: Some(format!("{} > {}", story.id, req.id)),
                });
            }

            // Check requirement ID format
            if !req.id.starts_with("REQ-") {
                result.findings.push(LintFinding {
                    level: LintLevel::Warning,
                    category: "Naming Conventions",
                    message: format!("Requirement ID '{}' should start with 'REQ-'", req.id),
                    location: Some(format!("{} > {}", story.id, req.id)),
                });
            }

            // Check for empty acceptance criteria
            if req.acceptance_criteria.is_empty() {
                result.findings.push(LintFinding {
                    level: LintLevel::Warning,
                    category: "Structure",
                    message: format!("Requirement '{}' has no acceptance criteria", req.id),
                    location: Some(format!("{} > {}", story.id, req.id)),
                });
            }

            // Lint each AC
            for ac in &req.acceptance_criteria {
                result.acs_checked += 1;

                // Check for duplicate AC ID
                if !seen_ac_ids.insert(ac.id.clone()) {
                    result.findings.push(LintFinding {
                        level: LintLevel::Error,
                        category: "Duplicate IDs",
                        message: format!("Duplicate AC ID: {}", ac.id),
                        location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                    });
                }

                // Check AC ID format
                if !ac.id.starts_with("AC-") {
                    result.findings.push(LintFinding {
                        level: LintLevel::Warning,
                        category: "Naming Conventions",
                        message: format!("AC ID '{}' should start with 'AC-'", ac.id),
                        location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                    });
                }

                // Check for empty text
                if ac.text.trim().is_empty() {
                    result.findings.push(LintFinding {
                        level: LintLevel::Warning,
                        category: "Content",
                        message: format!("AC '{}' has empty text", ac.id),
                        location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                    });
                }

                // Check kernel ACs have test mappings (ADR-0024)
                let is_kernel = req.must_have_ac && ac.must_have_ac;
                let has_automated_tests = ac
                    .tests
                    .iter()
                    .any(|t| AUTOMATED_TEST_TYPES.contains(&t.test_type.to_lowercase().as_str()));

                if is_kernel && !has_automated_tests {
                    result.findings.push(LintFinding {
                        level: LintLevel::Error,
                        category: "Kernel AC Coverage",
                        message: format!(
                            "Kernel AC '{}' has no automated test mappings (unit/bdd/integration)",
                            ac.id
                        ),
                        location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                    });
                }

                // Lint each test mapping
                for test in &ac.tests {
                    result.tests_checked += 1;

                    // Check test type is known
                    if !KNOWN_TEST_TYPES.contains(&test.test_type.to_lowercase().as_str()) {
                        result.findings.push(LintFinding {
                            level: LintLevel::Warning,
                            category: "Test Mappings",
                            message: format!(
                                "Unknown test type '{}' in AC '{}' (expected: {})",
                                test.test_type,
                                ac.id,
                                KNOWN_TEST_TYPES.join(", ")
                            ),
                            location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                        });
                    }

                    // Check file exists (if requested and file is specified)
                    if args.check_files
                        && let Some(ref file) = test.file
                    {
                        let file_path = Path::new(file);
                        if !file_path.exists() {
                            result.findings.push(LintFinding {
                                level: LintLevel::Warning,
                                category: "Test File References",
                                message: format!(
                                    "Test file '{}' referenced by AC '{}' does not exist",
                                    file, ac.id
                                ),
                                location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                            });
                        }
                    }

                    // Warn if unit test has no module (verbose only)
                    if test.test_type.to_lowercase() == "unit"
                        && test.module.is_none()
                        && args.verbose
                    {
                        result.findings.push(LintFinding {
                            level: LintLevel::Warning,
                            category: "Test Mappings",
                            message: format!(
                                "Unit test in AC '{}' has no module specified (tag: {})",
                                ac.id, test.tag
                            ),
                            location: Some(format!("{} > {} > {}", story.id, req.id, ac.id)),
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_test_ledger(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    /// Test that known test types constant matches documentation
    #[test]
    fn test_known_test_types_are_documented() {
        // These are the test types mentioned in CLAUDE.md and spec_ledger.yaml
        assert!(KNOWN_TEST_TYPES.contains(&"unit"));
        assert!(KNOWN_TEST_TYPES.contains(&"bdd"));
        assert!(KNOWN_TEST_TYPES.contains(&"integration"));
        assert!(KNOWN_TEST_TYPES.contains(&"docs"));
        assert!(KNOWN_TEST_TYPES.contains(&"manual"));
    }

    /// Test that automated test types are a subset of known types
    #[test]
    fn test_automated_types_are_known() {
        for t in AUTOMATED_TEST_TYPES {
            assert!(
                KNOWN_TEST_TYPES.contains(t),
                "Automated type '{}' should be in KNOWN_TEST_TYPES",
                t
            );
        }
    }

    #[test]
    fn test_valid_ledger_passes() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: true
            tests:
              - { type: unit, tag: "test_foo", module: "foo::tests" }
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(!result.has_errors(), "Valid ledger should have no errors");
    }

    #[test]
    fn test_duplicate_story_id_is_error() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-DUP-001
    requirements: []
  - id: US-DUP-001
    requirements: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_errors());
        assert!(result.findings.iter().any(|f| f.message.contains("Duplicate story ID")));
    }

    #[test]
    fn test_duplicate_req_id_is_error() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-DUP-001
        acceptance_criteria: []
      - id: REQ-DUP-001
        acceptance_criteria: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_errors());
        assert!(result.findings.iter().any(|f| f.message.contains("Duplicate requirement ID")));
    }

    #[test]
    fn test_duplicate_ac_id_is_error() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-DUP-001
            text: "First"
            tests: []
          - id: AC-DUP-001
            text: "Second"
            tests: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_errors());
        assert!(result.findings.iter().any(|f| f.message.contains("Duplicate AC ID")));
    }

    #[test]
    fn test_kernel_ac_without_tests_is_error() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-KERNEL-NOTESTS
            text: "Kernel AC without tests"
            must_have_ac: true
            tests: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_errors());
        assert!(result.findings.iter().any(|f| f.message.contains("no automated test mappings")));
    }

    #[test]
    fn test_non_kernel_ac_without_tests_is_ok() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-OPTIONAL-001
            text: "Optional AC without tests"
            must_have_ac: true
            tests: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        // REQ has must_have_ac=false, so AC is not kernel
        assert!(!result.has_errors());
    }

    #[test]
    fn test_unknown_test_type_is_warning() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-TEST-001
            text: "AC with unknown test type"
            must_have_ac: false
            tests:
              - { type: unknown_type, tag: "test" }
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_warnings());
        assert!(result.findings.iter().any(|f| f.message.contains("Unknown test type")));
    }

    #[test]
    fn test_empty_requirements_is_warning() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-EMPTY-001
    requirements: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_warnings());
        assert!(result.findings.iter().any(|f| f.message.contains("has no requirements")));
    }

    #[test]
    fn test_empty_acceptance_criteria_is_warning() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-EMPTY-001
        acceptance_criteria: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_warnings());
        assert!(result.findings.iter().any(|f| f.message.contains("has no acceptance criteria")));
    }

    #[test]
    fn test_naming_convention_warnings() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: STORY-001
    requirements:
      - id: REQUIREMENT-001
        acceptance_criteria:
          - id: CRITERIA-001
            text: "Bad naming"
            tests: []
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert!(result.has_warnings());
        let warnings: Vec<_> = result.warnings().collect();
        assert!(warnings.iter().any(|f| f.message.contains("should start with 'US-'")));
        assert!(warnings.iter().any(|f| f.message.contains("should start with 'REQ-'")));
        assert!(warnings.iter().any(|f| f.message.contains("should start with 'AC-'")));
    }

    #[test]
    fn test_lint_result_counts() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            tests:
              - { type: unit, tag: "test_foo" }
              - { type: bdd, tag: "@AC-TEST-001" }
  - id: US-TEST-002
    requirements:
      - id: REQ-TEST-002
        acceptance_criteria:
          - id: AC-TEST-002
            text: "Another AC"
            tests:
              - { type: integration, tag: "@AC-TEST-002" }
"#,
        );

        let result = lint_ledger(ledger.path(), &AcLintArgs::default()).unwrap();
        assert_eq!(result.stories_checked, 2);
        assert_eq!(result.requirements_checked, 2);
        assert_eq!(result.acs_checked, 2);
        assert_eq!(result.tests_checked, 3);
    }
}
