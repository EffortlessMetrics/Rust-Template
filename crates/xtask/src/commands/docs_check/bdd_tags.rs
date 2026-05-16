use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct BddTagIssue {
    /// Path to the feature file
    file: String,
    /// Line number where the tag was found
    line: usize,
    /// The tag that has an issue
    tag: String,
    /// Description of the issue
    issue: String,
}

/// Validate BDD feature file tags against spec_ledger.yaml.
///
/// This check validates:
/// 1. `@AC-*` tags in feature files exist as AC IDs in spec_ledger.yaml
/// 2. Reports orphaned tags (tags referencing non-existent ACs)
/// 3. Reports scenarios without AC tags (advisory only)
///
/// Special tags like `@ci-only`, `@smoke`, `@wip` are allowed and not validated.
///
/// Issue #95: BDD feature file tag validation
pub(crate) fn validate_bdd_tags() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let ledger_path = root.join("specs/spec_ledger.yaml");
    let features_dir = root.join("specs/features");

    if !ledger_path.exists() || !features_dir.exists() {
        // Can't validate without spec_ledger or feature files
        return Ok(());
    }

    // Load AC IDs from spec_ledger.yaml
    let ac_ids = extract_ac_ids_from_ledger(&ledger_path)?;

    // Regex to match @AC-* tags: @AC-XXX-NNN or @AC-XXX-NNN-SUFFIX
    let ac_tag_re = Regex::new(r"@(AC-[A-Z]+-[A-Z0-9]+(?:-[A-Z0-9]+)*)")
        .context("Failed to compile AC tag regex")?;

    // Special tags that are allowed and not validated
    let special_tags = [
        "@ci-only",
        "@smoke",
        "@wip",
        "@platform",
        "@issues",
        "@schema",
        "@ordering",
        "@filtering",
        "@pagination",
        "@cursor",
        "@offset",
        "@summary",
        "@error",
        "@devup",
        "@selective_testing",
        "@release_bundle_generation",
        "@release_bundle_structure",
        "@example_fork_ci",
    ];

    let mut orphaned_tags: Vec<BddTagIssue> = Vec::new();
    let mut untagged_scenarios: Vec<BddTagIssue> = Vec::new();

    // Scan all .feature files
    for entry in WalkDir::new(&features_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
        e.path().is_file() && e.path().extension().map(|s| s == "feature").unwrap_or(false)
    }) {
        let file_path = entry.path();
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let relative_file = file_path.strip_prefix(root).unwrap_or(file_path).display().to_string();

        // Track previous scenario info for checking when we encounter a new scenario
        let mut prev_scenario_line = 0;
        let mut prev_scenario_has_ac_tag = false;
        let mut prev_scenario_tags: Vec<String> = Vec::new();
        let mut prev_scenario_name = String::new();

        // Track current (pending) tags collected since last scenario
        let mut current_tags: Vec<String> = Vec::new();
        let mut current_has_ac_tag = false;

        let mut in_scenario = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for tag lines (start with @)
            if trimmed.starts_with('@') {
                // Extract all AC tags from this line
                for cap in ac_tag_re.captures_iter(trimmed) {
                    let tag = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    current_tags.push(format!("@{}", tag));

                    // Check if this AC ID exists in spec_ledger
                    if !ac_ids.contains(tag) {
                        orphaned_tags.push(BddTagIssue {
                            file: relative_file.clone(),
                            line: line_num + 1,
                            tag: format!("@{}", tag),
                            issue: "AC not found in spec_ledger".to_string(),
                        });
                    } else {
                        current_has_ac_tag = true;
                    }
                }
                // Also collect non-AC tags for special tag checking
                for tag in trimmed.split_whitespace() {
                    if tag.starts_with('@') && !tag.starts_with("@AC-") {
                        current_tags.push(tag.to_string());
                    }
                }
            }

            // Check for Scenario or Scenario Outline
            if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
                // If we were in a previous scenario, check if IT had an AC tag
                // (using saved previous scenario info, not current tags)
                if in_scenario && !prev_scenario_has_ac_tag {
                    // Only report if there were no special tags either
                    let has_special_tag = prev_scenario_tags.iter().any(|t| {
                        special_tags.iter().any(|st| t.to_lowercase() == st.to_lowercase())
                    });
                    if !has_special_tag {
                        untagged_scenarios.push(BddTagIssue {
                            file: relative_file.clone(),
                            line: prev_scenario_line,
                            tag: String::new(),
                            issue: format!("Scenario '{}' has no @AC-* tag", prev_scenario_name),
                        });
                    }
                }

                // Current tags belong to THIS scenario - save as previous for next iteration
                prev_scenario_line = line_num + 1;
                prev_scenario_has_ac_tag = current_has_ac_tag;
                prev_scenario_tags = current_tags.clone();
                prev_scenario_name = trimmed
                    .trim_start_matches("Scenario:")
                    .trim_start_matches("Scenario Outline:")
                    .trim()
                    .to_string();

                // Reset current tags for the next scenario
                in_scenario = true;
                current_tags.clear();
                current_has_ac_tag = false;
            }

            // Feature-level tags apply to all scenarios (reset tracking)
            if trimmed.starts_with("Feature:") {
                in_scenario = false;
                prev_scenario_has_ac_tag = false;
                prev_scenario_tags.clear();
                prev_scenario_name.clear();
                current_tags.clear();
                current_has_ac_tag = false;
            }
        }

        // Check final scenario (using saved previous scenario info)
        if in_scenario && !prev_scenario_has_ac_tag {
            let has_special_tag = prev_scenario_tags
                .iter()
                .any(|t| special_tags.iter().any(|st| t.to_lowercase() == st.to_lowercase()));
            if !has_special_tag && !untagged_scenarios.iter().any(|s| s.line == prev_scenario_line)
            {
                untagged_scenarios.push(BddTagIssue {
                    file: relative_file.clone(),
                    line: prev_scenario_line,
                    tag: String::new(),
                    issue: format!("Scenario '{}' has no @AC-* tag", prev_scenario_name),
                });
            }
        }
    }

    // Report results
    if !orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "BDD tag validation issues:".yellow().bold());
        for issue in &orphaned_tags {
            eprintln!(
                "  {}:{} - {} ({})",
                issue.file.cyan(),
                issue.line,
                issue.tag.red(),
                issue.issue
            );
        }
    }

    if !untagged_scenarios.is_empty() && orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "Untagged scenarios (advisory):".yellow().bold());
        // Only show first 5 to avoid noise
        for issue in untagged_scenarios.iter().take(5) {
            eprintln!("  {}:{} - {}", issue.file.cyan(), issue.line, issue.issue.yellow());
        }
        if untagged_scenarios.len() > 5 {
            eprintln!("  ... and {} more", untagged_scenarios.len() - 5);
        }
    }

    if !orphaned_tags.is_empty() {
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Add missing AC IDs to specs/spec_ledger.yaml");
        eprintln!("  • Or update the @AC-* tag to reference an existing AC");
        eprintln!("  • Use 'cargo xtask ac-new' to create new ACs");
        anyhow::bail!("{} orphaned @AC-* tag(s) found", orphaned_tags.len());
    }

    Ok(())
}

/// Extract AC IDs from spec_ledger.yaml.
/// Returns a set of all AC IDs (e.g., "AC-TPL-001", "AC-PLT-018").
pub(crate) fn extract_ac_ids_from_ledger(
    ledger_path: &Path,
) -> Result<std::collections::HashSet<String>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read {}", ledger_path.display()))?;

    let mut ac_ids = std::collections::HashSet::new();

    // Look for lines like "- id: AC-XXX-NNN"
    let ac_id_re =
        Regex::new(r"^\s*-\s*id:\s*(AC-[A-Z]+-[A-Z0-9]+(?:-[A-Z0-9]+)*)").expect("valid regex");

    for line in content.lines() {
        if let Some(cap) = ac_id_re.captures(line)
            && let Some(id) = cap.get(1)
        {
            ac_ids.insert(id.as_str().to_string());
        }
    }

    Ok(ac_ids)
}
