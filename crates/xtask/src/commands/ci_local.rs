use anyhow::Result;
use colored::Colorize;
use std::env;
use testing::process::EnvVarGuard;

pub fn run() -> Result<()> {
    println!("{}", "🔄 Running CI checks locally...".blue().bold());
    println!();

    let mut failed = Vec::new();

    // Step 1: Environment validation
    println!("{}", "[1/4] Environment validation...".bold());
    match crate::commands::doctor::run() {
        Ok(_) => println!("{} doctor passed\n", "✓".green()),
        Err(e) => {
            println!("{} doctor failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("doctor");
        }
    }

    // Step 2: Template selftest
    println!("{}", "[2/4] Template selftest...".bold());

    // Prevent recursive BDD execution when ci-local runs inside the acceptance suite.
    // Using EnvVarGuard for safe scoped mutation that auto-restores on drop.
    let selftest_result = {
        let guard = EnvVarGuard::new(&["XTASK_SKIP_BDD"]);
        guard.set("XTASK_SKIP_BDD", "1");
        crate::commands::selftest::run_with_verbosity(crate::Verbosity::Normal)
    }; // XTASK_SKIP_BDD restored here

    match selftest_result {
        Ok(_) => println!("{} selftest passed\n", "✓".green()),
        Err(e) => {
            println!("{} selftest failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("selftest");
        }
    }

    // Step 3: Security audit (optional - only if tools available)
    println!("{}", "[3/4] Security audit...".bold());
    match crate::commands::audit::run() {
        Ok(_) => println!("{} audit passed\n", "✓".green()),
        Err(e) => {
            // Audit failures are warnings in ci-local (not hard failures)
            println!("{} audit had issues (see above)\n", "⚠".yellow());
            eprintln!("{}", e);
        }
    }

    // Step 4: Documentation consistency
    println!("{}", "[4/4] Documentation consistency...".bold());
    match crate::commands::docs_check::run() {
        Ok(_) => println!("{} docs-check passed\n", "✓".green()),
        Err(e) => {
            println!("{} docs-check failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("docs-check");
        }
    }

    // Final: Check working tree
    let skip_git_status = env::var("XTASK_SKIP_GIT_STATUS").is_ok()
        || env::var("XTASK_LOW_RESOURCES").unwrap_or_default() == "1";

    if skip_git_status {
        println!("{} Skipping git status check (low-resource/test mode)\n", "⚠".yellow());
    } else {
        println!("{}", "Checking working tree...".bold());
        let output = std::process::Command::new("git").args(["status", "--porcelain"]).output()?;

        let status = String::from_utf8_lossy(&output.stdout);
        if !status.trim().is_empty() {
            println!("{} Working tree is dirty\n", "✗".red());
            println!("{}", status);
            failed.push("git-clean");
        } else {
            println!("{} Working tree clean\n", "✓".green());
        }
    }

    // Summary
    println!("{}", "=".repeat(40));
    if failed.is_empty() {
        println!("{}", "✓ CI-local passed! Ready to push.".green().bold());
        println!();
        println!("{}", "Next steps:".bold());
        println!("  • Review changes: {}", "git diff".cyan());
        println!("  • Push: {}", "git push".cyan());
    } else {
        println!("{}", "✗ CI-local failed".red().bold());
        println!();
        println!("Failed checks:");
        for check in &failed {
            println!("  • {}", check);
        }
        println!();
        println!("{}", "Fix issues above and re-run: cargo xtask ci-local".yellow());
        anyhow::bail!("{} check(s) failed", failed.len());
    }

    Ok(())
}
