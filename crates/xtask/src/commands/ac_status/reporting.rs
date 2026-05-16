use ac_kernel::{Ac, AcJson, AcStatus, build_status_json};
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::super::ac_parsing::Scenario;
use super::model::is_meta_ac;
use super::to_relative_path;
use crate::kernel::layout_for_repo;

pub(super) fn print_summary(acs: &HashMap<String, Ac>) -> Result<()> {
    // Compute totals from all ACs
    let total_acs = acs.len();
    let total_passing = acs.values().filter(|ac| ac.status == AcStatus::Pass).count();
    let total_failing = acs.values().filter(|ac| ac.status == AcStatus::Fail).count();
    let total_unknown = acs.values().filter(|ac| ac.status == AcStatus::Unknown).count();
    let coverage_percent =
        if total_acs > 0 { (total_passing as f64 / total_acs as f64) * 100.0 } else { 0.0 };

    // Count by must_have_ac flag (kernel/required vs optional)
    let must_have_count = acs.values().filter(|ac| ac.must_have_ac).count();
    let optional_count = acs.values().filter(|ac| !ac.must_have_ac).count();

    println!("AC Status Summary:");

    // Build breakdown string dynamically (only include non-zero categories)
    let mut breakdown_parts = Vec::new();
    if must_have_count > 0 {
        breakdown_parts.push(format!("{} must-have", must_have_count));
    }
    if optional_count > 0 {
        breakdown_parts.push(format!("{} optional", optional_count));
    }

    if breakdown_parts.is_empty() {
        println!("  Ledger: {} ACs", total_acs);
    } else {
        println!("  Ledger: {} ACs ({})", total_acs, breakdown_parts.join(", "));
    }

    println!("  Coverage: {:.2}% (passing ACs)", coverage_percent);
    println!("  {} {} passing", AcStatus::Pass.icon(), total_passing);
    println!("  {} {} failing", AcStatus::Fail.icon(), total_failing);
    println!("  {} {} unknown (no mapped tests)", AcStatus::Unknown.icon(), total_unknown);

    if total_failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

/// Print detailed information about a single AC.
///
/// Used by `cargo xtask ac-status --ac <ID>` to debug specific AC status.
pub(super) fn print_single_ac(
    acs: &HashMap<String, Ac>,
    filter_id: &str,
    json_output: bool,
) -> Result<()> {
    let ac = acs.get(filter_id).ok_or_else(|| {
        anyhow::anyhow!(
            "AC '{}' not found in ledger\n\n\
             try: cargo xtask ac-status --json\n\
             hint: check specs/spec_ledger.yaml for available AC IDs",
            filter_id
        )
    })?;

    if json_output {
        // Output just this AC as JSON
        let ac_json = AcJson {
            id: ac.id.clone(),
            story_id: ac.story_id.clone(),
            req_id: ac.req_id.clone(),
            text: ac.text.clone(),
            status: ac.status,
            source: ac.source,
            must_have_ac: ac.must_have_ac,
            scenarios: ac.scenarios.clone(),
            tests: ac.tests.clone(),
            tests_total: ac.tests_total,
            tests_executed: ac.tests_executed,
        };
        let json_str =
            serde_json::to_string_pretty(&ac_json).context("Failed to serialize AC to JSON")?;
        println!("{}", json_str);
    } else {
        // Print human-readable output
        println!("AC: {}", ac.id.cyan().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("  Story:       {}", ac.story_id);
        println!("  Requirement: {}", ac.req_id);
        println!("  Text:        {}", ac.text);
        println!();

        // Status with icon
        let status_display = match ac.status {
            AcStatus::Pass => format!("{} {}", "[PASS]".green(), "All tests passed"),
            AcStatus::Fail => format!("{} {}", "[FAIL]".red(), "One or more tests failed"),
            AcStatus::Unknown => format!("{} {}", "[UNKNOWN]".yellow(), "No tests executed"),
        };
        println!("  Status:      {}", status_display);
        println!("  Tests:       {} executed / {} total", ac.tests_executed, ac.tests_total);
        println!();

        // Scenarios
        if ac.scenarios.is_empty() {
            println!("  Scenarios:   {} (no BDD scenarios mapped)", "(none)".yellow());
            println!();
            println!("  💡 Hint: Add a scenario tagged with @{} to a .feature file", ac.id);
            println!("     Or run: cargo xtask ac-suggest-scenarios {}", ac.id);
        } else {
            println!("  Scenarios:");
            for scenario in &ac.scenarios {
                println!("    • {}", scenario);
            }
        }
        println!();

        // Tests
        if ac.tests.is_empty() {
            println!("  Tests:       {} (no tests declared in ledger)", "(none)".yellow());
        } else {
            println!("  Test mappings:");
            for test in &ac.tests {
                let module = test.module.as_deref().unwrap_or("-");
                println!("    • [{}] {} (tag: {})", test.test_type, module, test.tag);
            }
        }
        println!();

        // Tags
        if !ac.tags.is_empty() {
            println!("  Tags: {}", ac.tags.join(", "));
        }
        if ac.must_have_ac {
            println!("  Flags: must_have_ac=true");
        }
    }

    // Exit with error if AC failed
    if ac.status == AcStatus::Fail {
        anyhow::bail!("AC {} failed", filter_id);
    }

    Ok(())
}

pub(super) fn print_json_output(acs: &HashMap<String, Ac>) -> Result<()> {
    // Use ac-kernel's build_status_json for consistency with the kernel
    let output = build_status_json(acs);

    let json_output =
        serde_json::to_string_pretty(&output).context("Failed to serialize AC status to JSON")?;

    println!("{}", json_output);

    // Check for failures (still need to fail the command if tests failed)
    if output.must_have_acs.failing > 0 || output.optional_acs.failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

/// Extract template_version from spec_ledger.yaml for feature_status metadata
fn get_template_version() -> Option<String> {
    let ledger_path = layout_for_repo().ledger;
    if let Ok(content) = fs::read_to_string(&ledger_path) {
        for line in content.lines() {
            if line.trim().starts_with("template_version:")
                && let Some(version) = line.split(':').nth(1)
            {
                return Some(version.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Extract last_updated from spec_ledger.yaml for feature_status metadata
fn get_last_updated_date() -> Option<String> {
    let ledger_path = layout_for_repo().ledger;
    if let Ok(content) = fs::read_to_string(&ledger_path) {
        for line in content.lines() {
            if line.trim().starts_with("last_updated:")
                && let Some(date) = line.split(':').nth(1)
            {
                return Some(date.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Generate the markdown content for feature_status.md (shared between write and check modes)
pub(super) fn generate_status_md_content(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
) -> String {
    let mut output = String::new();
    output.push_str("<!--\n");
    output.push_str("  AUTO-GENERATED FILE: DO NOT EDIT BY HAND.\n\n");
    output.push_str("  Source of truth:\n");
    output.push_str("    - Specs: specs/spec_ledger.yaml\n");
    output.push_str("    - Tests: specs/features/**/*.feature + unit test results (cargo test)\n");
    output.push_str("  Generated by:\n");
    output.push_str("    - cargo run -p xtask -- ac-status\n");
    output.push_str("  Regenerated by:\n");
    output.push_str("    - cargo run -p xtask -- selftest (AC/ADR mapping step)\n");

    // Add template version and last updated metadata for docs-check validation (AC-PLT-010 invariants)
    if let Some(version) = get_template_version() {
        output.push_str(&format!("  Template Version: {}\n", version));
    }
    if let Some(last_updated) = get_last_updated_date() {
        output.push_str(&format!("  Last Updated: {}\n", last_updated));
    }

    output.push_str("    \n");
    output.push_str("  To update this file, modify specs or BDD scenarios, then run:\n");
    output.push_str("    cargo xtask ac-status\n");
    output.push_str("-->\n\n");
    output.push_str("# Feature Status\n\n");
    output.push_str("Auto-generated AC status from acceptance (BDD) and unit tests.\n\n");

    output.push_str("## AC Status Summary\n\n");
    output.push_str("> **How to read this**\n");
    output.push_str("> - `[PASS]` = at least one test (BDD or unit) ran and passed.\n");
    output.push_str("> - `[FAIL]` = at least one test ran and failed.\n");
    output.push_str("> - `[UNKNOWN]` = no local test ran for this AC. Common causes:\n");
    output.push_str(">   - No BDD scenario is tagged with this AC ID (e.g., `@AC-TPL-001`)\n");
    output.push_str(">   - No unit tests are mapped to this AC in `spec_ledger.yaml`\n");
    output.push_str(">   - Coverage file is missing or stale (run `cargo xtask bdd` first)\n");
    output.push_str(">   - The AC is intentionally meta/CI-only (see sections at the end)\n");
    output.push_str(">\n");
    output
        .push_str("> For formal definitions of `must_have_ac` and AC governance semantics, see\n");
    output.push_str(
        "> [`crates/ac-kernel/README.md`](../crates/ac-kernel/README.md#ac-governance-semantics).\n\n",
    );
    output.push_str("| AC ID | Story | Requirement | Status | Tests |\n");
    output.push_str("|-------|-------|-------------|--------|-------|\n");

    // Sort ACs for deterministic output
    let mut sorted_acs: Vec<_> = acs.values().collect();
    sorted_acs.sort_by_key(|ac| &ac.id);

    for ac in sorted_acs {
        output.push_str(&format!(
            "| {} | {} | {} | {} {} | {} |\n",
            ac.id,
            ac.story_id,
            ac.req_id,
            ac.status.icon(),
            ac.status.name(),
            ac.tests_total
        ));
    }

    // Unmapped ACs: Split into service-level and meta ACs
    // An AC is "unmapped" if it has no tests defined (tests_total == 0)
    // or has Unknown status (no test evidence was captured)
    let unmapped: Vec<_> =
        acs.values().filter(|ac| ac.tests_total == 0 || ac.status == AcStatus::Unknown).collect();

    let service_unmapped: Vec<_> = unmapped.iter().filter(|ac| !is_meta_ac(ac)).copied().collect();
    let meta_unmapped: Vec<_> = unmapped.iter().filter(|ac| is_meta_ac(ac)).copied().collect();

    // Service-level unmapped ACs (should ideally be empty)
    output.push_str("\n## Unmapped ACs (Service Behaviour)\n\n");
    output.push_str(
        "*(This list SHOULD be empty in this repo. If anything appears here, it's a bug.)*\n\n",
    );
    if service_unmapped.is_empty() {
        output.push_str("- *(none)*\n");
    } else {
        // Sort for deterministic output
        let mut sorted_service: Vec<_> = service_unmapped;
        sorted_service.sort_by_key(|ac| &ac.id);
        for ac in sorted_service {
            let text = ac.text.trim();
            output.push_str(&format!("- {}: {}\n", ac.id, text));
        }
    }

    // Meta / CI-only ACs (intentionally not tested locally)
    if !meta_unmapped.is_empty() {
        output.push_str("\n## Meta / CI-only ACs (Not Executed Locally)\n\n");
        output
            .push_str("These ACs describe test harness or example workspace behaviour. They are\n");
        output.push_str("validated in CI, not by local selftest:\n\n");
        let mut sorted_meta: Vec<_> = meta_unmapped;
        sorted_meta.sort_by_key(|ac| &ac.id);
        for ac in sorted_meta {
            let text = ac.text.trim();
            output.push_str(&format!("- {} – {}\n", ac.id, text));
        }
    }

    let mut unmapped_scenarios: Vec<_> =
        scenarios.values().filter(|s| !acs.contains_key(&s.ac_id)).collect();
    unmapped_scenarios.sort_by_key(|s| &s.name);

    if !unmapped_scenarios.is_empty() {
        output.push_str("\n## Unmapped Scenarios\n\n");
        output.push_str("Scenarios referencing non-existent ACs:\n\n");
        for scenario in unmapped_scenarios {
            output.push_str(&format!(
                "- Scenario '{}' references {} (in {})\n",
                scenario.name,
                scenario.ac_id,
                to_relative_path(&scenario.file)
            ));
        }
    }

    output
}

/// Write mode: Generate feature_status.md file
pub(super) fn generate_status_md(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    output_path: &Path,
    should_print_progress: bool,
) -> Result<()> {
    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = generate_status_md_content(acs, scenarios);
    fs::write(output_path, content)?;

    if should_print_progress {
        eprintln!("{} Generated {}", "[OK]".green(), output_path.display());
    }

    Ok(())
}

/// Check mode: Compare computed status against existing file without writing.
/// Returns error if the file content would differ.
pub(super) fn check_status_md(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    output_path: &Path,
    should_print_progress: bool,
) -> Result<()> {
    let expected = generate_status_md_content(acs, scenarios);

    let actual = fs::read_to_string(output_path).with_context(|| {
        format!(
            "AC status file not found: {}\n\n\
             hint: run 'cargo xtask ac-status' to generate it first",
            output_path.display()
        )
    })?;

    if expected != actual {
        // Find first differing line for diagnostics
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        let diff_line = expected_lines
            .iter()
            .zip(actual_lines.iter())
            .enumerate()
            .find(|(_, (e, a))| e != a)
            .map(|(i, _)| i + 1);

        let diff_info = if let Some(line) = diff_line {
            format!(" (first difference at line {})", line)
        } else if expected_lines.len() != actual_lines.len() {
            format!(
                " (line count differs: expected {}, got {})",
                expected_lines.len(),
                actual_lines.len()
            )
        } else {
            String::new()
        };

        anyhow::bail!(
            "AC status file is out of sync: {}{}\n\n\
             hint: run 'cargo xtask ac-status' to regenerate\n\
             then commit the updated file",
            output_path.display(),
            diff_info
        );
    }

    if should_print_progress {
        eprintln!("{} AC status file is up to date", "[OK]".green());
    }

    Ok(())
}
