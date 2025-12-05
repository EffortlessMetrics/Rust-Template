//! Contracts governance commands: fmt and check.
//!
//! These commands ensure that governed facts in documentation stay synchronized
//! with their sources (code and specs).
//!
//! ## Commands
//!
//! - `contracts-check`: Validate that all documented facts match sources (dry-run)
//! - `contracts-fmt`: Update documentation to match sources
//!
//! ## Design
//!
//! This follows the same pattern as the versioning module:
//! 1. Load a manifest (`specs/contracts_manifest.yaml`) defining patterns
//! 2. Compute current values from sources
//! 3. Plan edits by comparing current docs to expected values
//! 4. Apply edits atomically (or report drift in check mode)

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::ContractsSnapshot;

/// Contracts manifest loaded from specs/contracts_manifest.yaml
#[derive(Debug, Deserialize)]
struct ContractsManifest {
    #[allow(dead_code)]
    schema_version: String,
    #[allow(dead_code)]
    description: Option<String>,
    contracts: HashMap<String, ContractDef>,
    #[serde(default)]
    #[allow(dead_code)]
    validation: ValidationSettings,
}

/// Definition of a single governed contract (e.g., selftest_step_count)
#[derive(Debug, Deserialize)]
struct ContractDef {
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    description: Option<String>,
    patterns: Vec<PatternDef>,
}

/// Pattern definition for where a contract value appears in documentation
#[derive(Debug, Deserialize)]
struct PatternDef {
    file: String,
    regex: String,
    template: String,
    #[serde(default = "default_true")]
    required: bool,
}

fn default_true() -> bool {
    true
}

/// Validation settings from manifest
#[derive(Debug, Default, Deserialize)]
struct ValidationSettings {
    #[serde(default)]
    #[allow(dead_code)]
    strict_patterns: bool,
    #[serde(default)]
    #[allow(dead_code)]
    detect_orphans: bool,
}

/// A planned edit for contract synchronization
#[derive(Debug)]
pub struct ContractEdit {
    pub file: String,
    pub line_number: usize,
    pub old_text: String,
    pub new_text: String,
    pub contract_name: String,
}

/// Get the repository root from CARGO_MANIFEST_DIR
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("xtask parent").parent().expect("crates parent").to_path_buf()
}

/// Run contracts-check (dry-run validation).
pub fn check() -> Result<()> {
    fmt_impl(&repo_root(), true)
}

/// Run contracts-fmt (apply changes).
pub fn fmt() -> Result<()> {
    fmt_impl(&repo_root(), false)
}

fn fmt_impl(repo_root: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}", "📋 Checking contract governance...".blue().bold());
    } else {
        println!("{}", "📋 Synchronizing contract facts...".blue().bold());
    }
    println!();

    // 1. Compute the snapshot from sources
    let snapshot = ContractsSnapshot::compute(repo_root)?;

    println!("Computed facts from source:");
    println!("  • Selftest steps: {}", snapshot.selftest_step_count);
    println!(
        "  • AC counts: total={}, kernel={}, template={}, meta={}",
        snapshot.ac_counts.total,
        snapshot.ac_counts.kernel,
        snapshot.ac_counts.template,
        snapshot.ac_counts.meta
    );
    println!("  • Platform endpoints: {}", snapshot.platform_endpoints.len());
    println!("  • Required checks: {}", snapshot.required_checks.len());
    println!();

    // 2. Load the manifest
    let manifest_path = repo_root.join("specs/contracts_manifest.yaml");
    if !manifest_path.exists() {
        if dry_run {
            println!("{}", "No contracts manifest found - skipping".yellow());
            return Ok(());
        }
        anyhow::bail!("contracts_manifest.yaml not found at {}", manifest_path.display());
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: ContractsManifest = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse contracts_manifest.yaml")?;

    // 3. Plan changes
    let edits = plan_contract_edits(repo_root, &snapshot, &manifest)?;

    if edits.is_empty() {
        println!("{}", "✓ All contract facts are synchronized".green());
        return Ok(());
    }

    // 4. Apply or report
    if dry_run {
        println!("{}", "Contract drift detected:".yellow().bold());
        println!();

        for edit in &edits {
            println!("{}:{}", edit.file.cyan(), edit.line_number);
            println!("  Contract: {}", edit.contract_name.dimmed());
            println!("  {} {}", "-".red(), edit.old_text.dimmed());
            println!("  {} {}", "+".green(), edit.new_text.green());
            println!();
        }

        let files: std::collections::HashSet<_> = edits.iter().map(|e| &e.file).collect();
        anyhow::bail!(
            "contracts-check found {} edit(s) across {} file(s). Run `cargo xtask contracts-fmt` to fix.",
            edits.len(),
            files.len()
        );
    }

    // Apply edits
    apply_contract_edits(repo_root, &edits)?;

    println!("{}", format!("✓ Applied {} contract updates", edits.len()).green());
    Ok(())
}

fn plan_contract_edits(
    repo_root: &Path,
    snapshot: &ContractsSnapshot,
    manifest: &ContractsManifest,
) -> Result<Vec<ContractEdit>> {
    let mut edits = Vec::new();

    for (contract_name, contract_def) in &manifest.contracts {
        // Get the value for this contract
        let value = match contract_name.as_str() {
            "selftest_step_count" => snapshot.selftest_step_count,
            // Legacy name (deprecated, kept for compatibility)
            "kernel_ac_count" => snapshot.ac_counts.kernel,
            // New AC count contracts
            "ac_total" => snapshot.ac_counts.total,
            "ac_kernel" => snapshot.ac_counts.kernel,
            "ac_template" => snapshot.ac_counts.template,
            "ac_meta" => snapshot.ac_counts.meta,
            _ => continue, // Skip unknown contracts
        };

        for pattern_def in &contract_def.patterns {
            let file_path = repo_root.join(&pattern_def.file);

            if !file_path.exists() {
                if pattern_def.required {
                    eprintln!("  {} File not found: {}", "⚠".yellow(), pattern_def.file);
                }
                continue;
            }

            let content = fs::read_to_string(&file_path)?;
            let re = Regex::new(&pattern_def.regex).with_context(|| {
                format!("Invalid regex for {}: {}", contract_name, pattern_def.regex)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                if let Some(caps) = re.captures(line) {
                    // Extract the current number from the match
                    let current_value =
                        caps.get(1).map(|m| m.as_str().parse::<usize>().ok()).flatten();

                    // If the number differs from expected, plan an edit
                    if current_value != Some(value) {
                        // Build the expected text using the template
                        let expected = pattern_def.template.replace("{n}", &value.to_string());

                        // Replace the match with the expected text
                        let new_line = re.replace(line, expected.as_str()).to_string();

                        if new_line != line {
                            edits.push(ContractEdit {
                                file: pattern_def.file.clone(),
                                line_number: line_num + 1,
                                old_text: line.to_string(),
                                new_text: new_line,
                                contract_name: contract_name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(edits)
}

fn apply_contract_edits(repo_root: &Path, edits: &[ContractEdit]) -> Result<()> {
    // Group edits by file
    let mut by_file: HashMap<&str, Vec<&ContractEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.file).or_default().push(edit);
    }

    for (rel_path, file_edits) in by_file {
        let file_path = repo_root.join(rel_path);
        let content = fs::read_to_string(&file_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Sort edits by line number descending to avoid index shifting
        let mut sorted_edits = file_edits;
        sorted_edits.sort_by(|a, b| b.line_number.cmp(&a.line_number));

        for edit in sorted_edits {
            let idx = edit.line_number - 1;
            if idx < lines.len() && lines[idx] == edit.old_text {
                lines[idx].clone_from(&edit.new_text);
            }
        }

        // Write atomically via temp file
        let temp_path = file_path.with_extension("tmp");
        let new_content = lines.join("\n") + "\n";
        fs::write(&temp_path, &new_content)?;
        fs::rename(&temp_path, &file_path)?;

        println!("  {} {}", "✓".green(), rel_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_without_manifest_is_soft_fail() {
        // When no manifest exists and dry_run=true, should return Ok
        // (graceful degradation)
        let temp_dir = tempfile::tempdir().expect("temp dir");

        // Create minimal selftest.rs and spec_ledger.yaml for snapshot computation
        let xtask_dir = temp_dir.path().join("crates/xtask/src/commands");
        fs::create_dir_all(&xtask_dir).expect("create dir");
        fs::write(
            xtask_dir.join("selftest.rs"),
            r#"
            println!("[1/11] Step 1");
            println!("[2/11] Step 2");
            "#,
        )
        .expect("write");

        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).expect("create dir");
        fs::write(
            specs_dir.join("spec_ledger.yaml"),
            r#"
stories:
  - id: US-TEST
    requirements:
      - id: REQ-TEST
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST
            must_have_ac: true
"#,
        )
        .expect("write");

        // No contracts_manifest.yaml - should be OK in dry run
        let result = fmt_impl(temp_dir.path(), true);
        assert!(result.is_ok(), "Should succeed without manifest in dry run");
    }

    #[test]
    fn test_plan_edits_with_matching_content() {
        use crate::contracts::AcCounts;

        // When content already matches, should produce no edits
        let snapshot = ContractsSnapshot {
            selftest_step_count: 11,
            ac_counts: AcCounts { total: 105, kernel: 61, template: 27, meta: 17 },
            platform_endpoints: vec![],
            required_checks: vec![],
        };

        let manifest_yaml = r#"
schema_version: "1.0"
contracts:
  selftest_step_count:
    source: test
    patterns:
      - file: test.md
        regex: '(\d+)-step gate'
        template: "{n}-step gate"
"#;
        let manifest: ContractsManifest =
            serde_yaml::from_str(manifest_yaml).expect("parse manifest");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        // Content already has correct value
        fs::write(temp_dir.path().join("test.md"), "An 11-step gate\n").expect("write");

        let edits = plan_contract_edits(temp_dir.path(), &snapshot, &manifest).expect("plan edits");
        assert!(edits.is_empty(), "Should have no edits when content matches");
    }

    #[test]
    fn test_plan_edits_with_outdated_content() {
        use crate::contracts::AcCounts;

        // When content is outdated, should produce edit
        let snapshot = ContractsSnapshot {
            selftest_step_count: 11,
            ac_counts: AcCounts { total: 105, kernel: 61, template: 27, meta: 17 },
            platform_endpoints: vec![],
            required_checks: vec![],
        };

        let manifest_yaml = r#"
schema_version: "1.0"
contracts:
  selftest_step_count:
    source: test
    patterns:
      - file: test.md
        regex: '(\d+)-step gate'
        template: "{n}-step gate"
"#;
        let manifest: ContractsManifest =
            serde_yaml::from_str(manifest_yaml).expect("parse manifest");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        // Content has OLD value (10 instead of 11)
        fs::write(temp_dir.path().join("test.md"), "An 10-step gate\n").expect("write");

        let edits = plan_contract_edits(temp_dir.path(), &snapshot, &manifest).expect("plan edits");
        assert_eq!(edits.len(), 1, "Should have one edit");
        assert_eq!(edits[0].contract_name, "selftest_step_count");
        assert!(edits[0].old_text.contains("10-step"));
        assert!(edits[0].new_text.contains("11-step"));
    }
}
