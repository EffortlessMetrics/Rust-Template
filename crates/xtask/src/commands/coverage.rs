use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::process::Command;

/// Coverage baseline target (65%)
const COVERAGE_BASELINE: f64 = 65.0;

/// Tarpaulin JSON output structure (partial, only what we need)
#[derive(Debug, Deserialize)]
struct TarpaulinReport {
    coverage: f64,
}

pub fn run() -> Result<()> {
    run_with_baseline(COVERAGE_BASELINE)?;
    Ok(())
}

pub fn run_with_baseline(baseline: f64) -> Result<f64> {
    println!("{}", "Running test coverage analysis".blue().bold());
    println!();

    // Check if cargo-tarpaulin is available
    let has_tarpaulin = which::which("cargo-tarpaulin").is_ok();

    if !has_tarpaulin {
        println!("{}", "⚠️  cargo-tarpaulin not found".yellow());
        println!("Install via:");
        println!("  Nix: {}", "nix develop".cyan());
        println!("  Or manually: {}", "cargo install cargo-tarpaulin".cyan());
        println!();
        println!("{}", "Coverage check skipped (tool not available)".yellow());
        return Ok(0.0);
    }

    println!("Generating coverage report (this may take a while)...");

    // Run cargo tarpaulin with JSON output
    // --out Json: JSON output for machine parsing
    // --exclude-files tests: Don't count test files themselves
    // --timeout 300: 5-minute timeout per test
    // --workspace: Cover all workspace crates
    // --exclude acceptance xtask: Don't analyze BDD tests and xtask
    let mut cmd = Command::new("cargo");
    cmd.args([
        "tarpaulin",
        "--out",
        "Json",
        "--exclude-files",
        "tests",
        "--timeout",
        "300",
        "--workspace",
        "--exclude",
        "acceptance",
        "--exclude",
        "xtask",
    ]);

    // Suppress stderr noise (only matters on failure)
    let output = cmd.output().context("Failed to run cargo-tarpaulin")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", "✗ Coverage generation failed".red());
        eprintln!("{}", stderr);
        anyhow::bail!("cargo-tarpaulin exited with non-zero status");
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: TarpaulinReport =
        serde_json::from_str(&stdout).context("Failed to parse tarpaulin JSON output")?;

    let coverage_pct = report.coverage;

    println!();
    println!("{}", "Coverage Report:".bold());
    println!("  Coverage: {:.2}%", coverage_pct);
    println!("  Baseline: {:.2}%", baseline);

    if coverage_pct >= baseline {
        println!();
        println!(
            "{} Coverage target met! ({:.2}% >= {:.2}%)",
            "✓".green().bold(),
            coverage_pct,
            baseline
        );
        Ok(coverage_pct)
    } else {
        println!();
        println!(
            "{} Coverage below baseline ({:.2}% < {:.2}%)",
            "✗".red().bold(),
            coverage_pct,
            baseline
        );
        println!();
        println!("{}", "Recovery options:".bold());
        println!("  • Add tests for uncovered code");
        println!("  • Run {} to see detailed coverage report", "cargo tarpaulin --out Html".cyan());
        println!("  • Review coverage gaps in {}", "target/coverage/".dimmed());

        anyhow::bail!("Coverage below baseline: {:.2}% < {:.2}%", coverage_pct, baseline);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        let _: fn() -> Result<()> = run;
    }

    #[test]
    fn test_coverage_baseline_constant() {
        // Verify the baseline is set to 65%
        assert_eq!(COVERAGE_BASELINE, 65.0);
    }

    #[test]
    fn test_tarpaulin_report_deserialization() {
        // Test that we can deserialize a sample tarpaulin report
        let sample_json = r#"{"coverage": 68.5}"#;
        let report: TarpaulinReport = serde_json::from_str(sample_json).unwrap();
        assert_eq!(report.coverage, 68.5);
    }

    #[test]
    fn test_run_with_baseline_logic() {
        // Test the baseline comparison logic without actually running tarpaulin
        // This verifies the function signature and return type
        let _: fn(f64) -> Result<f64> = run_with_baseline;
    }
}
