//! Kernel Status: Aggregated health check for template kernel
//!
//! Provides a single "pane of glass" view of kernel health, including:
//! - Spec ledger metadata (version, schema, last updated)
//! - Git state (tag, working tree cleanliness)
//! - Kernel AC coverage (total, pass, unknown)
//! - Docs-as-Code invariants (version alignment, doc_index, feature_status)
//! - Governance gates (selftest summary)
//! - CI gates status
//! - Example fork wiring

use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Canonical spec ledger metadata (subset)
#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    last_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Ledger {
    metadata: LedgerMetadata,
}

/// Kernel AC counts parsed from feature_status.md
#[derive(Debug, Default)]
struct KernelAcCounts {
    total: usize,
    pass: usize,
    unknown: usize,
    fail: usize,
}

/// Simple status flag for each subsystem
struct SubsystemStatus {
    name: &'static str,
    ok: bool,
    note: Option<String>,
}

impl SubsystemStatus {
    fn ok(name: &'static str) -> Self {
        Self { name, ok: true, note: None }
    }

    fn fail(name: &'static str, note: impl Into<String>) -> Self {
        Self { name, ok: false, note: Some(note.into()) }
    }
}

/// Aggregate kernel health
pub fn run() -> Result<()> {
    let mut subsystems = Vec::new();

    println!("{}", "Kernel Status (Rust-Template)".bold());
    println!("{}", "==============================".bold());
    println!();

    // 1. Specs / version
    let (ledger_status, metadata) = match load_ledger_metadata() {
        Ok(m) => {
            println!("  Template version: {}", format!("v{}", m.template_version).green(),);
            (SubsystemStatus::ok("spec_ledger"), Some(m))
        }
        Err(e) => {
            println!("  Template version: {}", "ERROR".red());
            (SubsystemStatus::fail("spec_ledger", format!("{e:#}")), None)
        }
    };
    subsystems.push(ledger_status);

    // 2. Git state (tag and working tree)
    let kernel_tag = metadata
        .as_ref()
        .map(|m| format!("v{}-kernel", m.template_version))
        .unwrap_or_else(|| "unknown".into());

    let (is_at_tag, tag_display) = check_git_tag(&kernel_tag);
    if is_at_tag {
        println!("  Kernel tag:       {} (HEAD is at tag: {})", kernel_tag.green(), "yes".green());
    } else {
        println!("  Kernel tag:       {} (HEAD is at tag: {})", kernel_tag.yellow(), "no".yellow());
        if let Some(tag_info) = tag_display {
            println!("                    {}", tag_info.dimmed());
        }
    }

    let tree_clean = check_git_clean();
    if tree_clean {
        println!("  Tree clean:       {}", "yes".green());
        subsystems.push(SubsystemStatus::ok("git_tree"));
    } else {
        println!("  Tree clean:       {}", "no".yellow());
        subsystems.push(SubsystemStatus::fail("git_tree", "Working tree has uncommitted changes"));
    }
    println!();

    // 3. Kernel ACs (must_have_ac=true)
    println!("{}", "Kernel ACs (must_have_ac=true)".bold());
    let ac_counts = parse_kernel_ac_counts();
    println!("  Total:    {}", ac_counts.total);
    println!("  PASS:     {}", format!("{}", ac_counts.pass).green());
    if ac_counts.unknown > 0 {
        println!("  UNKNOWN:  {}", format!("{}", ac_counts.unknown).yellow());
        subsystems.push(SubsystemStatus::fail(
            "kernel_ac_coverage",
            format!("{} kernel ACs have unknown status", ac_counts.unknown),
        ));
    } else {
        println!("  UNKNOWN:  {}", "0".green());
        subsystems.push(SubsystemStatus::ok("kernel_ac_coverage"));
    }
    if ac_counts.fail > 0 {
        println!("  FAIL:     {}", format!("{}", ac_counts.fail).red());
    }
    println!();

    // 4. Docs-as-Code (versions + doc_index + feature_status invariants)
    println!("{}", "Docs-as-Code".bold());
    match super::docs_check::check_version_alignment_v2() {
        Ok(()) => {
            println!("  version alignment: {}", "OK".green());
            subsystems.push(SubsystemStatus::ok("docs_version_alignment"));
        }
        Err(e) => {
            println!("  version alignment: {}", "FAILED".red());
            println!("    {}", format!("{e:#}").red());
            subsystems.push(SubsystemStatus::fail(
                "docs_version_alignment",
                "Run `cargo xtask docs-check` and fix mismatches",
            ));
        }
    }

    match super::docs_check::validate_doc_index() {
        Ok(()) => {
            println!("  doc_index <-> front-matter: {}", "OK".green());
            subsystems.push(SubsystemStatus::ok("doc_index_frontmatter"));
        }
        Err(e) => {
            println!("  doc_index <-> front-matter: {}", "FAILED".red());
            println!("    {}", format!("{e:#}").red());
            subsystems.push(SubsystemStatus::fail(
                "doc_index_frontmatter",
                "Run `cargo xtask docs-check` and fix doc_index/frontmatter entries",
            ));
        }
    }

    match super::docs_check::validate_feature_status_invariants() {
        Ok(()) => {
            println!("  feature_status header: {}", "OK".green());
            subsystems.push(SubsystemStatus::ok("feature_status_header"));
        }
        Err(e) => {
            println!("  feature_status header: {}", "FAILED".red());
            println!("    {}", format!("{e:#}").red());
            subsystems.push(SubsystemStatus::fail(
                "feature_status_header",
                "Re-run `cargo xtask ac-status` to regenerate header, then docs-check",
            ));
        }
    }
    println!();

    // 5. Governance / selftest (summary only)
    println!("{}", "Governance Gates".bold());
    match super::selftest::run_with_verbosity(crate::Verbosity::Quiet) {
        Ok(()) => {
            println!("  selftest: {}", "PASS".green());
            subsystems.push(SubsystemStatus::ok("selftest"));
        }
        Err(e) => {
            println!("  selftest: {}", "FAIL".red());
            println!("    {}", format!("{e:#}").red());
            subsystems.push(SubsystemStatus::fail(
                "selftest",
                "Run `cargo xtask selftest` and fix failing gates",
            ));
        }
    }
    println!();

    // 6. CI gates
    println!("{}", "CI Gates".bold());
    let tier1_has_strict_ac =
        check_ci_env_var(".github/workflows/tier1-selftest.yml", "XTASK_STRICT_AC_COVERAGE");
    let tier1_has_strict_precommit =
        check_ci_env_var(".github/workflows/tier1-selftest.yml", "XTASK_STRICT_PRECOMMIT");

    println!("  tier1-selftest.yml:");
    if tier1_has_strict_ac {
        println!("    XTASK_STRICT_AC_COVERAGE = 1 on main {}", "✓".green());
    } else {
        println!("    XTASK_STRICT_AC_COVERAGE = {} {}", "not set".yellow(), "✗".yellow());
    }
    if tier1_has_strict_precommit {
        println!("    XTASK_STRICT_PRECOMMIT   = 1        {}", "✓".green());
    } else {
        println!("    XTASK_STRICT_PRECOMMIT   = {} {}", "not set".yellow(), "✗".yellow());
    }
    println!();

    // 7. Example fork CI (static check that workflow + example exist)
    println!("{}", "Examples".bold());
    let example_ok = Path::new("examples/fork-customization").exists()
        && Path::new(".github/workflows/ci-example-fork.yml").exists();
    if example_ok {
        println!("  example fork wiring: {}", "present".green());
        subsystems.push(SubsystemStatus::ok("example_fork_ci"));
    } else {
        println!("  example fork wiring: {}", "missing/partial".yellow());
        subsystems.push(SubsystemStatus::fail(
            "example_fork_ci",
            "Ensure `examples/fork-customization/` and `ci-example-fork.yml` exist",
        ));
    }
    println!();

    // 8. Summary
    println!("{}", "Summary".bold());
    let failures: Vec<_> = subsystems.iter().filter(|s| !s.ok).collect();

    if failures.is_empty() {
        println!("  {}", "All kernel checks passing".green().bold());
    } else {
        println!("  {} {} failing subsystem(s):", "Kernel issues:".red().bold(), failures.len());
        for s in &failures {
            println!("    - {}", s.name.red());
            if let Some(note) = &s.note {
                println!("      {}", note);
            }
        }
    }

    if let Some(m) = metadata {
        println!();
        println!(
            "  Kernel: v{} (schema {}, updated {})",
            m.template_version,
            m.schema_version.unwrap_or_else(|| "1.0".into()),
            m.last_updated.unwrap_or_else(|| "unknown".into()),
        );
    }

    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("kernel-status found {} failing subsystem(s)", failures.len())
    }
}

fn load_ledger_metadata() -> Result<LedgerMetadata> {
    let content = fs::read_to_string("specs/spec_ledger.yaml")?;
    let ledger: Ledger = serde_yaml::from_str(&content)?;
    Ok(ledger.metadata)
}

/// Check if HEAD is at the specified kernel tag
fn check_git_tag(expected_tag: &str) -> (bool, Option<String>) {
    // Check if the tag exists and points to HEAD
    let output = Command::new("git").args(["describe", "--tags", "--exact-match", "HEAD"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let current_tag = String::from_utf8_lossy(&o.stdout).trim().to_string();
            (current_tag == expected_tag, None)
        }
        Ok(_) => {
            // HEAD is not at a tag, check if the tag exists
            let tag_exists = Command::new("git")
                .args(["rev-parse", &format!("refs/tags/{}", expected_tag)])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if tag_exists {
                (false, Some(format!("Tag {} exists but HEAD is not at it", expected_tag)))
            } else {
                (false, Some(format!("Tag {} does not exist yet", expected_tag)))
            }
        }
        Err(_) => (false, Some("git not available".into())),
    }
}

/// Check if the git working tree is clean
fn check_git_clean() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| o.status.success() && o.stdout.is_empty())
        .unwrap_or(false)
}

/// Parse kernel AC counts by:
/// 1. Reading spec_ledger.yaml to find ACs with must_have_ac=true
/// 2. Reading feature_status.md to get their status
fn parse_kernel_ac_counts() -> KernelAcCounts {
    let mut counts = KernelAcCounts::default();

    // Get kernel AC IDs from spec_ledger.yaml
    let ledger_content = match fs::read_to_string("specs/spec_ledger.yaml") {
        Ok(c) => c,
        Err(_) => return counts,
    };

    // Find all AC IDs that have must_have_ac: true using line-by-line parsing
    let mut kernel_ac_ids = std::collections::HashSet::new();
    let mut current_ac_id: Option<String> = None;
    for line in ledger_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- id: AC-") || trimmed.starts_with("id: AC-") {
            // Extract AC ID
            if let Some(id_start) = trimmed.find("AC-") {
                let id_part = &trimmed[id_start..];
                let id_end = id_part
                    .find(|c: char| !c.is_alphanumeric() && c != '-')
                    .unwrap_or(id_part.len());
                current_ac_id = Some(id_part[..id_end].to_string());
            }
        } else if trimmed.starts_with("must_have_ac: true") {
            if let Some(ref id) = current_ac_id {
                kernel_ac_ids.insert(id.clone());
            }
        } else if trimmed.starts_with("- id:") || trimmed.starts_with("id:") {
            // Reset if we hit a non-AC id
            if !trimmed.contains("AC-") {
                current_ac_id = None;
            }
        }
    }

    // Now read feature_status.md to get status for each kernel AC
    let status_content = match fs::read_to_string("docs/feature_status.md") {
        Ok(c) => c,
        Err(_) => return counts,
    };

    // Parse the status table: | AC-ID | Story | Requirement | Status | Tests |
    let row_re = Regex::new(r"\|\s*(AC-[A-Z0-9-]+)\s*\|[^|]+\|[^|]+\|\s*\[(PASS|FAIL|UNKNOWN)\]")
        .unwrap_or_else(|_| Regex::new(r"^$").unwrap());

    for cap in row_re.captures_iter(&status_content) {
        if let (Some(ac_id), Some(status)) = (cap.get(1), cap.get(2)) {
            if kernel_ac_ids.contains(ac_id.as_str()) {
                counts.total += 1;
                match status.as_str() {
                    "PASS" => counts.pass += 1,
                    "FAIL" => counts.fail += 1,
                    "UNKNOWN" => counts.unknown += 1,
                    _ => {}
                }
            }
        }
    }

    counts
}

/// Check if a CI workflow file contains a specific environment variable set to "1"
fn check_ci_env_var(workflow_path: &str, env_var: &str) -> bool {
    let content = match fs::read_to_string(workflow_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Look for patterns like:
    // XTASK_STRICT_AC_COVERAGE: ${{ ... '1' ... }}
    // XTASK_STRICT_PRECOMMIT: "1"
    content.contains(env_var)
        && (content.contains(&format!("{}: \"1\"", env_var))
            || content.contains(&format!("{}:", env_var)))
}
