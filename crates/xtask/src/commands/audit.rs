use crate::run_cmd;
use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "Running security & dependency audit".blue().bold());
    println!();

    let mut issues = 0;
    let warnings = 0;

    // Check if tools are available
    let has_cargo_audit = which::which("cargo-audit").is_ok();
    let has_cargo_deny = which::which("cargo-deny").is_ok();

    if !has_cargo_audit && !has_cargo_deny {
        println!("{}", "⚠️  No audit tools found".yellow());
        println!("Install via Nix: {}", "nix develop".cyan());
        println!("Or manually:");
        println!("  cargo install cargo-audit");
        println!("  cargo install cargo-deny");
        return Ok(());
    }

    // Run cargo-audit
    if has_cargo_audit {
        println!("Running cargo-audit...");
        let mut cmd = crate::cargo_cmd("audit", &["--deny", "warnings"]);

        match run_cmd(&mut cmd) {
            Ok(_) => println!("{}", "✓ No vulnerabilities".green()),
            Err(e) => {
                println!("{}", "✗ Found vulnerabilities".red());
                eprintln!("{}", e);
                issues += 1;
            }
        }
    }

    // Run cargo-deny
    if has_cargo_deny {
        println!();
        println!("Running cargo-deny...");
        let mut cmd = crate::cargo_cmd("deny", &["check"]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", "✓ All checks passed".green());
                } else {
                    println!("{}", "✗ Issues found".red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    if !stdout.is_empty() {
                        eprintln!("{}", stdout);
                    }
                    if !stderr.is_empty() {
                        eprintln!("{}", stderr);
                    }

                    issues += 1;
                }
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
                issues += 1;
            }
        }
    }

    println!();
    println!("{}", "Summary:".bold());
    if issues == 0 && warnings == 0 {
        println!("{} All security checks passed!", "✓".green().bold());
    } else {
        if issues > 0 {
            println!("{} {} issue(s) found", "✗".red().bold(), issues);
            println!();
            println!("{}", "Recovery options:".bold());
            println!("  • Update dependency: {}", "cargo update <crate>".cyan());
            println!("  • Pin safe version in Cargo.toml");
            println!("  • Document mitigation: {}", "see ADR-0007".dimmed());
            println!("  • Review: {}", "deny.toml policy configuration".dimmed());
        }
        if warnings > 0 {
            println!("{} {} warning(s)", "⚠".yellow(), warnings);
        }
    }

    if issues > 0 {
        anyhow::bail!("{} security/dependency issues found", issues);
    }

    Ok(())
}
