use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::time::Instant;

/// Run full template self-test suite
#[allow(dead_code)]
pub fn run() -> Result<()> {
    run_with_verbosity(crate::Verbosity::Normal)
}

/// Run full template self-test suite with specified verbosity
pub fn run_with_verbosity(verbosity: crate::Verbosity) -> Result<()> {
    let start_time = Instant::now();
    println!("{}", "======================================".blue());
    println!("{}", "  Template Self-Test Suite".blue());
    println!("{}", "======================================".blue());
    println!();

    let mut failed = 0;

    // Step 1: Core checks
    println!("{}", "[1/5] Running core checks (fmt, clippy, tests)...".blue());
    let step_start = Instant::now();
    match crate::commands::check::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Core checks passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Core checks passed", "✓".green());
            }
        }
        Err(e) => {
            eprintln!("  {} Core checks failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 2: BDD acceptance tests
    println!("{}", "[2/5] Running BDD acceptance tests...".blue());
    let step_start = Instant::now();
    match crate::commands::bdd::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} BDD scenarios passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} BDD scenarios passed", "✓".green());
            }
            if Path::new("target/junit/acceptance.xml").exists() {
                println!("  {} JUnit XML generated", "✓".green());
            } else {
                println!("  {} JUnit XML not found", "⚠".yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} BDD tests failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 3: AC status mapping
    println!("{}", "[3/5] Running AC status mapping...".blue());
    let step_start = Instant::now();
    match run_ac_status(verbosity) {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!(
                    "  {} AC status script executed ({:.2}s)",
                    "✓".green(),
                    elapsed.as_secs_f64()
                );
            } else {
                println!("  {} AC status script executed", "✓".green());
            }
            if Path::new("docs/feature_status.md").exists() {
                println!("  {} Feature status generated", "✓".green());
            } else {
                println!("  {} Feature status not found", "⚠".yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} AC status failed: {}", "✗".red(), e);
            // Don't fail the suite if AC status has issues - it's informational
            println!("  {} Continuing (AC status is informational)", "⚠".yellow());
        }
    }
    println!();

    // Step 4: LLM context bundler
    println!("{}", "[4/5] Testing LLM context bundler...".blue());
    let step_start = Instant::now();
    match crate::commands::bundle::run("implement_ac") {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Bundle generated ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Bundle generated", "✓".green());
            }
            if let Ok(metadata) = std::fs::metadata(".llm/bundle/implement_ac.md") {
                println!("  {} Bundle size: {} bytes", "✓".green(), metadata.len());
            }
        }
        Err(e) => {
            eprintln!("  {} Bundler failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 5: Policy tests (if conftest available)
    println!("{}", "[5/5] Running policy tests...".blue());
    let step_start = Instant::now();
    match crate::commands::policy_test::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Policy tests passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Policy tests passed", "✓".green());
            }
        }
        Err(e) => {
            eprintln!("  {} Policy tests: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Summary
    let total_elapsed = start_time.elapsed();
    println!("{}", "======================================".blue());
    if failed == 0 {
        println!("{}", "✓ All self-tests passed!".green());
        if verbosity.is_verbose() {
            println!("\n{} {:.2}s", "Total elapsed time:".bold(), total_elapsed.as_secs_f64());
        }
        println!();
        println!("The template is working correctly:");
        println!("  • xtask commands functional");
        println!("  • BDD scenarios passing");
        println!("  • AC mapping operational");
        println!("  • LLM bundler working");
        println!();
        println!("Ready for:");
        println!("  • Service development: {}", "docs/how-to/new-service-from-template.md".blue());
        println!("  • AC-first workflow: {}", "docs/tutorials/first-ac-change.md".blue());
        println!("{}", "======================================".blue());
        Ok(())
    } else {
        eprintln!("{}", format!("✗ {} test suite(s) failed", failed).red());
        if verbosity.is_verbose() {
            eprintln!("\n{} {:.2}s", "Total elapsed time:".bold(), total_elapsed.as_secs_f64());
        }
        println!("{}", "======================================".blue());
        anyhow::bail!("{} test suites failed", failed)
    }
}

fn run_ac_status(verbosity: crate::Verbosity) -> Result<()> {
    // Use Rust-native AC status implementation
    crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity,
        ..Default::default()
    })
}
