//! Public API diff check for contract crates.
//!
//! This command detects breaking changes in the public API of contract crates
//! by comparing the current API against a baseline (main branch).
//!
//! ## Contract Crates Checked
//!
//! - `platform-contract` - HTTP API types
//! - `xtask-contract` - CLI output types
//! - `receipts-core` - Receipt schemas
//! - `spec-types` - Spec file types
//!
//! ## Usage
//!
//! ```bash
//! cargo xtask check-api-diff
//! ```
//!
//! ## Exit Codes
//!
//! - `0`: No breaking changes detected
//! - `1`: Breaking changes detected (requires ADR)

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Contract crates that must remain stable
const CONTRACT_CRATES: &[&str] =
    &["platform-contract", "xtask-contract", "receipts-core", "spec-types"];

/// Dependencies that contract crates must NOT have
const FORBIDDEN_DEPS: &[&str] = &[
    "axum",       // HTTP framework (belongs to app-http)
    "tokio",      // Async runtime (belongs to app-http)
    "clap",       // CLI parser (belongs to xtask)
    "sqlx",       // Database (belongs to adapters-db-sqlx)
    "tonic",      // gRPC framework (belongs to adapters-grpc)
    "jsonschema", // Schema validation (belongs to spec-runtime)
];

/// Get the repository root from CARGO_MANIFEST_DIR
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("xtask parent").parent().expect("crates parent").to_path_buf()
}

/// Check if cargo-public-api is available
fn has_cargo_public_api() -> bool {
    Command::new("cargo").args(["public-api", "--help"]).output().map(|_| true).unwrap_or(false)
}

/// Run public API diff check using cargo-public-api
fn check_with_public_api(crate_name: &str) -> Result<bool> {
    println!("{} Checking {}...", "🔍".blue().bold(), crate_name.cyan());

    let output = Command::new("cargo")
        .args(["public-api", "diff", crate_name, "--color=never"])
        .output()
        .context("Failed to run cargo public-api")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for breaking changes in output
    // cargo-public-api outputs "Breaking:" for breaking changes
    let has_breaking =
        stdout.contains("Breaking:") || stdout.contains("Major:") || stdout.contains("Removed:");

    if !output.status.success() {
        eprintln!("{}", stderr);
        return Err(anyhow::anyhow!("cargo public-api failed for {}", crate_name));
    }

    if has_breaking {
        eprintln!();
        eprintln!("{}", "Breaking changes detected:".red().bold());
        eprintln!();
        eprintln!("{}", stdout);
        eprintln!();
        eprintln!("{}", "To approve this change:".yellow().bold());
        eprintln!("  1. Create an ADR documenting the breaking change");
        eprintln!("  2. Update specs/contracts_manifest.yaml with new contract version");
        eprintln!("  3. Update dependent crates and consumers");
        eprintln!("  4. Run: cargo xtask release-prepare");
        return Ok(true);
    }

    // Check for additions (non-breaking but notable)
    if stdout.contains("Added:") {
        println!("{}", "API additions detected (non-breaking):".yellow());
        println!("{}", stdout);
    }

    println!("{}", "✓ No breaking changes".green());
    Ok(false)
}

/// Fallback: Run basic cargo check without specialized tooling
fn check_basic(crate_name: &str) -> Result<bool> {
    println!("{} Checking {} (basic mode)...", "🔍".blue().bold(), crate_name.cyan());

    // Check if crate exists
    let crate_path = repo_root().join("crates").join(crate_name);
    if !crate_path.exists() {
        eprintln!("{} Crate not found: {}", "⚠".yellow(), crate_name);
        return Ok(false);
    }

    // Run cargo check on the crate
    let output = Command::new("cargo")
        .args(["check", "-p", crate_name])
        .output()
        .context("Failed to run cargo check")?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("cargo check failed for {}", crate_name));
    }

    println!("{}", "✓ No compilation errors".green());
    Ok(false)
}

/// Arguments for check-api-diff command
pub struct CheckApiDiffArgs {
    /// Path to ADR approving the change (optional)
    pub adr: Option<String>,
}

/// Run the check-api-diff command
pub fn run(args: CheckApiDiffArgs) -> Result<()> {
    println!("{}", "Checking contract crates for API breaking changes...".blue().bold());
    println!();

    // If ADR is provided, validate it exists
    if let Some(adr_path) = &args.adr {
        let full_path = repo_root().join(adr_path);
        if !full_path.exists() {
            eprintln!("{} ADR not found: {}", "⚠".yellow(), full_path.display());
            return Err(anyhow::anyhow!("ADR file does not exist"));
        }
        println!("{} Using ADR for approval: {}", "✓".green(), adr_path.cyan());
        println!();
    }

    let mut has_breaking = false;
    let mut checked_crates = HashSet::new();

    // Check each contract crate
    for crate_name in CONTRACT_CRATES {
        checked_crates.insert(crate_name);

        // First, verify layering (no forbidden dependencies)
        let cargo_toml_path = repo_root().join("crates").join(crate_name).join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
            let mut violations = Vec::new();
            for forbidden in FORBIDDEN_DEPS {
                if content.contains(forbidden) {
                    violations.push(forbidden.to_string());
                }
            }

            if !violations.is_empty() {
                eprintln!();
                eprintln!("{} {} has forbidden dependencies:", "⚠".yellow(), crate_name.cyan());
                for violation in &violations {
                    eprintln!("  - {}", violation.red());
                }
                eprintln!();
                eprintln!(
                    "{} Contract crates must not depend on: {}",
                    "Forbidden:".yellow(),
                    FORBIDDEN_DEPS.join(", ").dimmed()
                );
                return Err(anyhow::anyhow!("Layering violation in {}", crate_name));
            }
        }

        // Run API diff check
        let breaking = if has_cargo_public_api() {
            check_with_public_api(crate_name)?
        } else {
            eprintln!("{} cargo-public-api not available, using basic check", "⚠".yellow());
            check_basic(crate_name)?
        };

        if breaking {
            has_breaking = true;
        }
    }

    println!();
    println!("{}", "Summary:".blue().bold());
    println!("  Checked crates: {}", checked_crates.len());
    println!("  Contract crates: {}", CONTRACT_CRATES.join(", ").cyan());

    if has_breaking {
        println!();
        eprintln!("{}", "❌ Breaking changes detected".red().bold());
        eprintln!();
        eprintln!(
            "{}",
            "Please create an ADR and update the contract manifest before proceeding.".yellow()
        );
        std::process::exit(1);
    }

    println!();
    println!("{}", "✓ All contract crates are stable".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_crates_defined() {
        // Verify all contract crates are defined
        assert_eq!(CONTRACT_CRATES.len(), 4);
        assert!(CONTRACT_CRATES.contains(&"platform-contract"));
        assert!(CONTRACT_CRATES.contains(&"xtask-contract"));
        assert!(CONTRACT_CRATES.contains(&"receipts-core"));
        assert!(CONTRACT_CRATES.contains(&"spec-types"));
    }

    #[test]
    fn test_forbidden_deps_defined() {
        // Verify forbidden dependencies are defined
        assert!(FORBIDDEN_DEPS.contains(&"axum"));
        assert!(FORBIDDEN_DEPS.contains(&"tokio"));
        assert!(FORBIDDEN_DEPS.contains(&"clap"));
        assert!(FORBIDDEN_DEPS.contains(&"sqlx"));
        assert!(FORBIDDEN_DEPS.contains(&"tonic"));
        assert!(FORBIDDEN_DEPS.contains(&"jsonschema"));
    }
}
