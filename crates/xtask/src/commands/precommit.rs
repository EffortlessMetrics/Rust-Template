use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "Running pre-commit checks...".blue().bold());

    // Run core checks (fmt, clippy, tests)
    crate::commands::check::run()?;

    // Run AC status and auto-stage feature_status.md if it changed
    run_ac_status_with_autostage()?;

    // Run docs-check and spellcheck in soft mode (warnings only, unless XTASK_STRICT_PRECOMMIT=1)
    let strict = std::env::var("XTASK_STRICT_PRECOMMIT").unwrap_or_default() == "1";

    if std::env::var("XTASK_SKIP_DOCS_CHECK").unwrap_or_default() == "1" {
        println!("{} Skipping docs-check because XTASK_SKIP_DOCS_CHECK=1", "[WARN]".yellow());
    } else {
        match crate::commands::docs_check::run() {
            Ok(_) => {}
            Err(e) => {
                if strict {
                    println!("{} docs-check failed (strict mode enabled)", "[FAIL]".red());
                    return Err(e);
                } else {
                    println!("{} docs-check failed (continuing in soft mode)", "[WARN]".yellow());
                    println!("  {}", e.to_string().lines().next().unwrap_or(""));
                    println!("  💡 To fail on docs issues: XTASK_STRICT_PRECOMMIT=1");
                }
            }
        }
    }

    if std::env::var("XTASK_SKIP_SPELLCHECK").unwrap_or_default() == "1" {
        println!("{} Skipping spellcheck because XTASK_SKIP_SPELLCHECK=1", "[WARN]".yellow());
    } else {
        match crate::commands::spellcheck::run_with_default_targets() {
            Ok(_) => {}
            Err(e) => {
                if strict {
                    println!("{} spellcheck failed (strict mode enabled)", "[FAIL]".red());
                    return Err(e);
                } else {
                    println!("{} spellcheck failed (continuing in soft mode)", "[WARN]".yellow());
                    println!("  {}", e.to_string().lines().next().unwrap_or(""));
                    println!("  💡 To fail on spelling issues: XTASK_STRICT_PRECOMMIT=1");
                }
            }
        }
    }

    println!("{}", "Pre-commit checks completed".green().bold());
    Ok(())
}

fn run_ac_status_with_autostage() -> Result<()> {
    // Run AC status to regenerate docs/feature_status.md
    match crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        ..Default::default()
    }) {
        Ok(_) => {}
        Err(e) => {
            // AC status might fail if ACs are failing, but we still want to
            // auto-stage the generated file and continue
            println!(
                "{} AC status reported failures (will auto-stage feature_status.md anyway)",
                "[WARN]".yellow()
            );
            println!("  {}", e.to_string().lines().next().unwrap_or(""));
        }
    }

    // Check if docs/feature_status.md changed
    let status_output =
        Command::new("git").args(["status", "--porcelain", "docs/feature_status.md"]).output()?;

    let status_str = String::from_utf8_lossy(&status_output.stdout);
    if !status_str.trim().is_empty() {
        // File changed, auto-stage it
        Command::new("git").args(["add", "docs/feature_status.md"]).status()?;

        println!("{} Updated docs/feature_status.md via ac-status (auto-staged)", "✓".green());
    }

    Ok(())
}
