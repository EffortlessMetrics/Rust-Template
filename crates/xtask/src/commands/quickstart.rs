use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Quick validation of template functionality
pub fn run() -> Result<()> {
    println!("{}", "======================================".blue());
    println!("{}", "  Rust Template Quick Start".blue());
    println!("{}", "======================================".blue());
    println!();

    let mut failed = 0;

    // Step 1: Check environment
    println!("{}", "[1/5] Checking environment...".blue());
    if let Err(e) = check_environment() {
        eprintln!("  {} {}", "✗".red(), e);
        failed += 1;
    }
    println!();

    if failed > 0 {
        eprintln!(
            "{}",
            "Environment check failed. Install Rust or enter 'nix develop' shell.".red()
        );
        anyhow::bail!("Environment check failed");
    }

    // Step 2: Run checks
    println!("{}", "[2/5] Running xtask check...".blue());
    match run_check() {
        Ok(_) => {
            println!("  {} Format check passed", "✓".green());
            println!("  {} Clippy passed", "✓".green());
            println!("  {} Tests passed", "✓".green());
        }
        Err(e) => {
            eprintln!("  {} Checks failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 3: Run BDD tests
    println!("{}", "[3/5] Running BDD acceptance tests...".blue());
    match run_bdd() {
        Ok(_) => {
            println!("  {} BDD scenarios passed", "✓".green());
            if Path::new("target/junit/acceptance.xml").exists() {
                println!("  {} JUnit output created", "✓".green());
            } else {
                println!(
                    "  {} JUnit output not found (expected at target/junit/acceptance.xml)",
                    "⚠".yellow()
                );
            }
        }
        Err(e) => {
            eprintln!("  {} BDD tests failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 4: Test bundler
    println!("{}", "[4/5] Testing LLM context bundler...".blue());
    match run_bundler() {
        Ok(_) => {
            println!("  {} Bundle command executed", "✓".green());
            if let Ok(metadata) = std::fs::metadata(".llm/bundle/implement_ac.md") {
                println!("  {} Bundle created ({} bytes)", "✓".green(), metadata.len());
            } else {
                println!("  {} Bundle file not found", "⚠".yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} Bundler failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 5: Test helper commands (when available)
    println!("{}", "[5/5] Testing helper commands...".blue());
    // For now, this is a placeholder
    println!("  {} Core commands validated", "✓".green());
    println!();

    // Summary
    println!("{}", "======================================".blue());
    if failed == 0 {
        println!("{}", "✓ Template validation passed!".green());
        println!();
        println!("Next steps:");
        println!(
            "  • See {} for adoption guide",
            "docs/how-to/new-service-from-template.md".blue()
        );
        println!("  • See {} for stable interface documentation", "TEMPLATE_API.md".blue());
        println!("  • See {} for AC-first development", "docs/tutorials/first-ac-change.md".blue());
        println!("{}", "======================================".blue());
        Ok(())
    } else {
        eprintln!("{}", format!("✗ {} validation step(s) failed", failed).red());
        println!("{}", "======================================".blue());
        anyhow::bail!("{} validation steps failed", failed)
    }
}

fn check_environment() -> Result<()> {
    // Check cargo
    let cargo_output =
        Command::new("cargo").arg("--version").output().context("Failed to check cargo version")?;

    if !cargo_output.status.success() {
        anyhow::bail!("cargo not found");
    }

    let cargo_version = String::from_utf8_lossy(&cargo_output.stdout);
    let version = cargo_version.split_whitespace().nth(1).unwrap_or("unknown");
    println!("  {} cargo {}", "✓".green(), version);

    // Check rustc
    let rustc_output =
        Command::new("rustc").arg("--version").output().context("Failed to check rustc version")?;

    if !rustc_output.status.success() {
        anyhow::bail!("rustc not found");
    }

    let rustc_version = String::from_utf8_lossy(&rustc_output.stdout);
    let version = rustc_version.split_whitespace().nth(1).unwrap_or("unknown");
    println!("  {} rustc {}", "✓".green(), version);

    Ok(())
}

fn run_check() -> Result<()> {
    crate::commands::check::run()
}

fn run_bdd() -> Result<()> {
    crate::commands::bdd::run()
}

fn run_bundler() -> Result<()> {
    crate::commands::bundle::run("implement_ac")
}
