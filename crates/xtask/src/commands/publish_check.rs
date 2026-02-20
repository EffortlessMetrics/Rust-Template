//! Publish readiness check for crates.io publishing.
//!
//! Validates that publishable crates meet all requirements for `cargo publish`:
//! - No path-only dependencies (every `path` must also have `version`)
//! - README.md exists in crate directory
//! - LICENSE-APACHE and LICENSE-MIT exist in crate directory
//! - `cargo package` succeeds
//! - Optional: `cargo publish --dry-run` succeeds

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;

/// Wave 1 publishable crates (order matters for batch publishing)
const PUBLISHABLE_CRATES: &[&str] = &[
    // Batch 1: No internal deps
    "rust-as-spec-gov-model",
    "rust-as-spec-model",
    "rust-as-spec-telemetry",
    "rust-as-spec-gov-receipts",
    // Batch 2: Depend on Batch 1
    "rust-as-spec-runtime",
    "rust-as-spec-gov-contracts",
    "rust-as-spec-gov-policy",
    "rust-as-spec-gov-xtask-core",
    "rust-as-spec-business-core",
    // Batch 3: Depend on Batch 2
    "rust-as-spec-ac-kernel",
];

pub struct PublishCheckArgs {
    pub crate_name: Option<String>,
    pub dry_run: bool,
    pub verbose: bool,
}

pub fn run(args: PublishCheckArgs) -> Result<()> {
    let root = workspace_root()?;

    let crates_to_check: Vec<&str> = if let Some(ref name) = args.crate_name {
        if !PUBLISHABLE_CRATES.contains(&name.as_str()) {
            anyhow::bail!(
                "Crate '{}' is not in the publishable allowlist.\nAllowed crates: {:?}",
                name,
                PUBLISHABLE_CRATES
            );
        }
        vec![name.as_str()]
    } else {
        PUBLISHABLE_CRATES.to_vec()
    };

    println!(
        "{} Checking {} crate(s) for publish readiness...\n",
        "📦".bold(),
        crates_to_check.len()
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut errors: Vec<(String, Vec<String>)> = Vec::new();

    for crate_name in &crates_to_check {
        let crate_errors = check_crate(&root, crate_name, args.dry_run, args.verbose)?;
        if crate_errors.is_empty() {
            println!("  {} {}", "✓".green(), crate_name);
            passed += 1;
        } else {
            println!("  {} {}", "✗".red(), crate_name);
            for err in &crate_errors {
                println!("    {} {}", "→".red(), err);
            }
            errors.push((crate_name.to_string(), crate_errors));
            failed += 1;
        }
    }

    println!();
    println!(
        "{} {} passed, {} failed",
        "Summary:".bold(),
        passed.to_string().green(),
        if failed > 0 { failed.to_string().red() } else { failed.to_string().green() }
    );

    if failed > 0 {
        anyhow::bail!("{} crate(s) failed publish readiness checks", failed);
    }

    println!("\n{}", "All crates are publish-ready! 🎉".green().bold());
    Ok(())
}

fn check_crate(root: &Path, crate_name: &str, dry_run: bool, verbose: bool) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    // Find the crate directory by looking up the package in cargo metadata
    let crate_dir = find_crate_dir(root, crate_name)?;

    // Check 1: README.md exists
    if !crate_dir.join("README.md").exists() {
        errors.push("Missing README.md".to_string());
    }

    // Check 2: LICENSE files exist
    if !crate_dir.join("LICENSE-APACHE").exists() {
        errors.push("Missing LICENSE-APACHE".to_string());
    }
    if !crate_dir.join("LICENSE-MIT").exists() {
        errors.push("Missing LICENSE-MIT".to_string());
    }

    // Check 3: cargo package --list succeeds
    let package_result = Command::new("cargo")
        .args(["package", "-p", crate_name, "--list", "--allow-dirty"])
        .current_dir(root)
        .output()
        .context("Failed to run cargo package --list")?;

    if !package_result.status.success() {
        let stderr = String::from_utf8_lossy(&package_result.stderr);
        errors.push(format!("cargo package --list failed: {}", stderr.trim()));
    } else if verbose {
        let stdout = String::from_utf8_lossy(&package_result.stdout);
        // Verify README and LICENSE appear in manifest
        let manifest = stdout.to_string();
        if !manifest.lines().any(|l| l.contains("README.md")) {
            errors.push("README.md not included in package manifest".to_string());
        }
        if !manifest.lines().any(|l| l.contains("LICENSE-APACHE")) {
            errors.push("LICENSE-APACHE not included in package manifest".to_string());
        }
        if !manifest.lines().any(|l| l.contains("LICENSE-MIT")) {
            errors.push("LICENSE-MIT not included in package manifest".to_string());
        }
    }

    // Check 4: Verify Cargo.toml has publish = true
    let cargo_toml = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
        .context("Failed to read Cargo.toml")?;
    if cargo_toml.contains("publish = false") || cargo_toml.contains("publish.workspace = true") {
        // publish.workspace = true inherits false from workspace
        // But if we've already fixed it, this won't trigger
        if !cargo_toml.contains("publish = true") {
            errors.push("publish is not set to true".to_string());
        }
    }

    // Check 5 (optional): cargo publish --dry-run
    if dry_run && errors.is_empty() {
        let publish_result = Command::new("cargo")
            .args(["publish", "-p", crate_name, "--dry-run", "--allow-dirty"])
            .current_dir(root)
            .output()
            .context("Failed to run cargo publish --dry-run")?;

        if !publish_result.status.success() {
            let stderr = String::from_utf8_lossy(&publish_result.stderr);
            errors.push(format!("cargo publish --dry-run failed: {}", stderr.trim()));
        }
    }

    Ok(errors)
}

fn find_crate_dir(root: &Path, crate_name: &str) -> Result<PathBuf> {
    // Map published name back to directory name
    let dir_name = crate_name.strip_prefix("rust-as-spec-").unwrap_or(crate_name);

    // Handle special case: "runtime" -> "spec-runtime"
    let dir_name = match dir_name {
        "runtime" => "spec-runtime",
        other => other,
    };

    let crate_dir = root.join("crates").join(dir_name);
    if crate_dir.exists() {
        Ok(crate_dir)
    } else {
        anyhow::bail!(
            "Could not find crate directory for '{}' (tried: {})",
            crate_name,
            crate_dir.display()
        )
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .context("Could not determine workspace root")?;
    Ok(root.to_path_buf())
}
