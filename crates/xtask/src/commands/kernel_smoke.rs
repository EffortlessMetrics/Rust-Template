use anyhow::Result;
use colored::Colorize;

/// Run a fast smoke test to verify the kernel is healthy
/// (docs-check + selftest)
pub fn run() -> Result<()> {
    println!("{}", "🚀 Running kernel smoke test...".blue().bold());
    println!();
    println!("{}", "This runs:".dimmed());
    println!("  1. {} - Documentation consistency", "cargo xtask docs-check".cyan());
    println!("  2. {} - Full governance validation", "cargo xtask selftest".cyan());
    println!();

    let mut failures = Vec::new();

    // Step 1: docs-check
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "Step 1: Documentation checks".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    match crate::commands::docs_check::run() {
        Ok(_) => {
            println!();
            println!("{} Documentation checks passed", "✓".green().bold());
        }
        Err(e) => {
            println!();
            println!("{} Documentation checks failed", "✗".red().bold());
            failures.push(format!("docs-check: {}", e));
        }
    }

    println!();

    // Step 2: selftest
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "Step 2: Selftest (8 gates)".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    match crate::commands::selftest::run() {
        Ok(_) => {
            println!();
            println!("{} Selftest passed", "✓".green().bold());
        }
        Err(e) => {
            println!();
            println!("{} Selftest failed", "✗".red().bold());
            failures.push(format!("selftest: {}", e));
        }
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "Kernel Smoke Test Summary".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if failures.is_empty() {
        println!();
        println!("{} All checks passed!", "✓".green().bold());
        println!();
        println!("{}", "The kernel is healthy and ready to fork.".dimmed());
        Ok(())
    } else {
        println!();
        println!("{} {} check(s) failed:", "✗".red().bold(), failures.len());
        for (i, failure) in failures.iter().enumerate() {
            println!("  {}. {}", i + 1, failure);
        }
        println!();
        anyhow::bail!("Kernel smoke test failed")
    }
}
