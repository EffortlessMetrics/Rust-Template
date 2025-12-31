//! Ensure all kernel ACs (must_have_ac=true) have at least one test mapping.
//!
//! This command validates that every AC marked as kernel has explicit test
//! coverage declared in spec_ledger.yaml. This is a guardrail that prevents
//! adding kernel ACs without corresponding test mappings.
//!
//! Per ADR-0024, kernel ACs are those where BOTH the requirement AND the AC
//! have `must_have_ac: true`.

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::kernel::layout_for_repo;

/// Arguments for the ac-ensure-kernel-mapped command
#[derive(Debug, Clone, Default)]
pub struct AcEnsureKernelMappedArgs {
    /// Show verbose output
    pub verbose: bool,
    /// Return non-zero exit code if any kernel ACs are unmapped
    pub strict: bool,
}

/// Result of the kernel mapping check
#[derive(Debug)]
pub struct KernelMappingResult {
    /// Total number of kernel ACs
    pub total_kernel: usize,
    /// Kernel ACs with at least one test mapping
    pub mapped: Vec<String>,
    /// Kernel ACs without any test mapping
    pub unmapped: Vec<(String, String)>, // (ac_id, req_id)
}

/// Run the ac-ensure-kernel-mapped check
pub fn run(args: AcEnsureKernelMappedArgs) -> Result<()> {
    let layout = layout_for_repo();
    let result = check_kernel_mappings(&layout.ledger, args.verbose)?;

    // Print results
    println!();
    println!("{}", "Kernel AC Test Mapping Check".bold());
    println!("{}", "═".repeat(40));
    println!();
    println!("  Total kernel ACs:  {}", result.total_kernel.to_string().cyan());
    println!("  {} Mapped:          {}", "✓".green(), result.mapped.len().to_string().green());

    if result.unmapped.is_empty() {
        println!();
        println!("{}", "✓ All kernel ACs have test mappings".green().bold());
        return Ok(());
    }

    println!("  {} Unmapped:        {}", "✗".red(), result.unmapped.len().to_string().red());
    println!();

    // Show unmapped ACs grouped by requirement
    let mut by_req: HashMap<String, Vec<String>> = HashMap::new();
    for (ac_id, req_id) in &result.unmapped {
        by_req.entry(req_id.clone()).or_default().push(ac_id.clone());
    }

    println!("{}", "Unmapped kernel ACs:".bold());
    for (req_id, acs) in by_req.iter() {
        println!("  {} ({})", req_id.yellow(), acs.len());
        for ac_id in acs {
            println!("    • {}", ac_id);
        }
    }

    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Add test mappings to {} for each unmapped AC", "specs/spec_ledger.yaml".cyan());
    println!("  2. Example mapping:");
    println!("     {}", "tests:".dimmed());
    println!("       {}", "- { type: unit, tag: \"test_foo\", module: \"foo::tests\" }".dimmed());
    println!(
        "       {}",
        "- { type: bdd, tag: \"@AC-XXX\", file: \"specs/features/foo.feature\" }".dimmed()
    );
    println!();
    println!("  Or demote the AC by setting {} in spec_ledger.yaml", "must_have_ac: false".cyan());

    if args.strict {
        anyhow::bail!("{} kernel ACs are missing test mappings", result.unmapped.len());
    }

    Ok(())
}

/// Check that all kernel ACs have test mappings in spec_ledger.yaml
pub fn check_kernel_mappings(ledger_path: &Path, verbose: bool) -> Result<KernelMappingResult> {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Ledger {
        stories: Vec<Story>,
    }

    #[derive(Debug, Deserialize)]
    struct Story {
        #[expect(dead_code, reason = "deserialized for schema completeness")]
        id: String,
        requirements: Vec<Requirement>,
    }

    #[derive(Debug, Deserialize)]
    struct Requirement {
        id: String,
        #[serde(default = "default_true")]
        must_have_ac: bool,
        acceptance_criteria: Vec<AcceptanceCriteria>,
    }

    #[derive(Debug, Deserialize)]
    struct AcceptanceCriteria {
        id: String,
        #[serde(default = "default_true")]
        must_have_ac: bool,
        #[serde(default)]
        tests: Vec<TestMapping>,
    }

    #[derive(Debug, Deserialize)]
    struct TestMapping {
        #[serde(rename = "type")]
        test_type: String,
        /// Tag field deserialized for schema completeness; only test_type used in mapping check.
        #[serde(default)]
        #[expect(dead_code, reason = "deserialized for schema completeness; only test_type used")]
        tag: String,
    }

    fn default_true() -> bool {
        true
    }

    // Parse ledger
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;
    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger: {}", ledger_path.display()))?;

    let mut mapped = Vec::new();
    let mut unmapped = Vec::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                // Kernel AC = both requirement AND AC have must_have_ac=true
                let is_kernel = req.must_have_ac && ac.must_have_ac;
                if !is_kernel {
                    continue;
                }

                // Check if AC has any automated test mappings
                let has_automated_tests = ac.tests.iter().any(|t| {
                    let test_type = t.test_type.to_lowercase();
                    matches!(test_type.as_str(), "unit" | "bdd" | "integration")
                });

                if has_automated_tests {
                    mapped.push(ac.id.clone());
                    if verbose {
                        println!(
                            "  {} {} has {} test mapping(s)",
                            "✓".green(),
                            ac.id,
                            ac.tests.len()
                        );
                    }
                } else {
                    unmapped.push((ac.id.clone(), req.id.clone()));
                    if verbose {
                        println!(
                            "  {} {} has no test mappings (req: {})",
                            "✗".red(),
                            ac.id,
                            req.id
                        );
                    }
                }
            }
        }
    }

    Ok(KernelMappingResult { total_kernel: mapped.len() + unmapped.len(), mapped, unmapped })
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

    #[test]
    fn all_kernel_acs_mapped() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-001
    requirements:
      - id: REQ-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-001
            must_have_ac: true
            tests:
              - { type: unit, tag: "test_foo" }
          - id: AC-002
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-002" }
"#,
        );

        let result = check_kernel_mappings(ledger.path(), false).unwrap();
        assert_eq!(result.total_kernel, 2);
        assert_eq!(result.mapped.len(), 2);
        assert!(result.unmapped.is_empty());
    }

    #[test]
    fn unmapped_kernel_ac_detected() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-001
    requirements:
      - id: REQ-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-001
            must_have_ac: true
            tests: []
"#,
        );

        let result = check_kernel_mappings(ledger.path(), false).unwrap();
        assert_eq!(result.total_kernel, 1);
        assert!(result.mapped.is_empty());
        assert_eq!(result.unmapped.len(), 1);
        assert_eq!(result.unmapped[0].0, "AC-001");
    }

    #[test]
    fn non_kernel_ac_ignored() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-001
    requirements:
      - id: REQ-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-001
            must_have_ac: true
            tests: []
"#,
        );

        let result = check_kernel_mappings(ledger.path(), false).unwrap();
        // AC-001 is not kernel because REQ has must_have_ac=false
        assert_eq!(result.total_kernel, 0);
    }

    #[test]
    fn ac_level_must_have_respected() {
        let ledger = write_test_ledger(
            r#"
stories:
  - id: US-001
    requirements:
      - id: REQ-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-001
            must_have_ac: false
            tests: []
"#,
        );

        let result = check_kernel_mappings(ledger.path(), false).unwrap();
        // AC-001 is not kernel because AC itself has must_have_ac=false
        assert_eq!(result.total_kernel, 0);
    }
}
