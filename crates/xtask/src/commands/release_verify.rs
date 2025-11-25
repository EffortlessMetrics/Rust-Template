use anyhow::Result;
use colored::Colorize;
use std::env;

pub fn run() -> Result<()> {
    println!("{}", "🚀 Running release verification...".blue().bold());
    println!();

    let mut failed = Vec::new();

    // Run selftest without re-entering the BDD harness
    println!("{}", "[1/3] Running selftest...".bold());
    let prev_skip_bdd = env::var("XTASK_SKIP_BDD").ok();
    // SAFETY: Nested selftest runs within BDD need to skip BDD recursion.
    unsafe {
        env::set_var("XTASK_SKIP_BDD", "1");
    }
    let selftest_result = crate::commands::selftest::run_with_verbosity(crate::Verbosity::Normal);
    unsafe {
        if let Some(prev) = prev_skip_bdd {
            env::set_var("XTASK_SKIP_BDD", prev);
        } else {
            env::remove_var("XTASK_SKIP_BDD");
        }
    }

    match selftest_result {
        Ok(_) => println!("{} Selftest passed\n", "✓".green()),
        Err(e) => {
            println!("{} Selftest failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("selftest");
        }
    }

    // Run audit
    println!("{}", "[2/3] Running audit...".bold());
    match crate::commands::audit::run() {
        Ok(_) => println!("{} Audit passed\n", "✓".green()),
        Err(e) => {
            println!("{} Audit failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("audit");
        }
    }

    // Run docs-check
    println!("{}", "[3/3] Running docs-check...".bold());
    match crate::commands::docs_check::run() {
        Ok(_) => println!("{} Docs-check passed\n", "✓".green()),
        Err(e) => {
            println!("{} Docs-check failed\n", "✗".red());
            eprintln!("{}", e);
            failed.push("docs-check");
        }
    }

    // Check working tree
    let skip_git_status = env::var("XTASK_SKIP_GIT_STATUS").is_ok()
        || env::var("XTASK_LOW_RESOURCES").unwrap_or_default() == "1";
    if skip_git_status {
        println!("{} Skipping git status check (low-resource/test mode)\n", "?".yellow());
    } else {
        println!("{}", "Checking working tree...".bold());
        let output = std::process::Command::new("git").args(["status", "--porcelain"]).output()?;

        let status = String::from_utf8_lossy(&output.stdout);
        if !status.trim().is_empty() {
            println!("{} Working tree is dirty", "✗".red());
            println!("{}", status);
            failed.push("git-clean");
        } else {
            println!("{} Working tree clean\n", "✓".green());
        }
    }

    // Summary
    println!("{}", "=".repeat(40));
    if failed.is_empty() {
        println!("{}", "✓ Release verification passed!".green().bold());
        println!();
        println!("{}", "Git command sequence:".bold());
        println!("  {}", "git commit -am 'Release vX.Y.Z'".cyan());
        println!("  {}", "git tag -a vX.Y.Z -m 'Release vX.Y.Z'".cyan());
        println!("  {}", "git push origin main --follow-tags".cyan());
    } else {
        println!("{}", "✗ Release verification failed".red().bold());
        println!();
        println!("Failed checks:");
        for check in &failed {
            println!("  • {}", check);
        }
        println!();
        println!("{}", "Fix issues above and re-run:".bold());
        println!("  {}", "cargo xtask release-verify".cyan());
        anyhow::bail!("{} check(s) failed", failed.len());
    }

    Ok(())
}
