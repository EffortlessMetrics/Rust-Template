//! Kernel Status: Aggregated health check for template kernel
//!
//! Provides a single "pane of glass" view of kernel health, including:
//! - Spec ledger metadata (version, schema, last updated)
//! - Docs-as-Code invariants (version alignment, doc_index, feature_status)
//! - Governance gates (selftest summary)
//! - Example fork wiring

use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::Path;

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

    println!("{}", "Kernel Status".bold());
    println!("{}", "==============".bold());
    println!();

    // 1. Specs / version
    let (ledger_status, metadata) = match load_ledger_metadata() {
        Ok(m) => {
            println!(
                "Specs: {} (schema {}, last updated {})",
                format!("v{}", m.template_version).green(),
                m.schema_version.clone().unwrap_or_else(|| "1.0".into()),
                m.last_updated.clone().unwrap_or_else(|| "unknown".into()),
            );
            (SubsystemStatus::ok("spec_ledger"), Some(m))
        }
        Err(e) => {
            println!("Specs: {}", "ERROR".red());
            (SubsystemStatus::fail("spec_ledger", format!("{e:#}")), None)
        }
    };
    subsystems.push(ledger_status);
    println!();

    // 2. Docs-as-Code (versions + doc_index + feature_status invariants)
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

    // 3. Governance / selftest (summary only)
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

    // 4. Example fork CI (static check that workflow + example exist)
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

    // 5. Summary
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
