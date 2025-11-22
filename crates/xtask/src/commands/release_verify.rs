use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "🚀 Running release verification...".blue().bold());
    println!();

    let mut failed = Vec::new();

    // Run selftest
    println!("{}", "[1/3] Running selftest...".bold());
    match crate::commands::selftest::run_with_verbosity(crate::Verbosity::Normal) {
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
