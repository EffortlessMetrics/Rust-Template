use crate::commands::test_changed::{
    BddPlan, bdd_plan_from_changes, format_tag_expression, get_changed_files,
};
use anyhow::Result;
use colored::Colorize;
use std::env;

#[derive(Debug, Clone, Copy)]
pub struct CheckOptions {
    pub skip_fmt: bool,
    pub fmt_skip_reason: Option<&'static str>,
    pub low_resource_mode: bool,
}

impl CheckOptions {
    pub fn from_env() -> Self {
        let low_resource_mode = env::var("XTASK_LOW_RESOURCES").unwrap_or_default() == "1";
        let is_tier2_windows = cfg!(windows);
        let skip_fmt = low_resource_mode || is_tier2_windows;

        let fmt_skip_reason = if skip_fmt {
            match (is_tier2_windows, low_resource_mode) {
                (true, true) => Some("Tier-2 Windows and low-resource mode"),
                (true, false) => Some("Tier-2 Windows"),
                (false, true) => Some("low-resource mode"),
                (false, false) => None,
            }
        } else {
            None
        };

        Self { skip_fmt, fmt_skip_reason, low_resource_mode }
    }
}

/// Run all checks: fmt, clippy, nextest
pub fn run() -> Result<()> {
    run_with_options(CheckOptions::from_env())
}

/// Run all checks with explicit options (used by selftest to relax fmt on Tier-2/low-resource)
pub fn run_with_options(options: CheckOptions) -> Result<()> {
    let low_resource_mode = options.low_resource_mode;
    let skip_bdd_env = env::var("XTASK_SKIP_BDD").unwrap_or_default() == "1";
    let skip_bdd = low_resource_mode || skip_bdd_env;

    if options.skip_fmt {
        let reason = options.fmt_skip_reason.unwrap_or("Tier-2 or low-resource mode");
        // Keep the word "format" in the message so UX tests that look for it still pass even when skipped.
        println!("{} format (fmt) check skipped ({reason})", "[WARN]".yellow());
    } else {
        println!("Running format check (fmt)...");
        crate::run_cmd(&mut crate::cargo_cmd("fmt", &["--all", "--", "--check"]))?;
        println!("fmt was checked");
    }

    println!("Running clippy...");
    // On Windows, exclude xtask to avoid file locking issues (can't rebuild running binary)
    let clippy_args = if cfg!(windows) {
        vec![
            "--workspace",
            "--exclude",
            "xtask",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]
    } else {
        vec!["--all-targets", "--all-features", "--", "-D", "warnings"]
    };
    crate::run_cmd(&mut crate::cargo_cmd("clippy", &clippy_args))?;
    println!("clippy was checked");

    println!("Running tests...");
    // On Windows, exclude xtask to avoid file locking issues (can't rebuild running binary)
    let test_args = if cfg!(windows) {
        vec!["--workspace", "--exclude", "acceptance", "--exclude", "xtask"]
    } else {
        vec!["--workspace", "--exclude", "acceptance"]
    };
    crate::run_cmd(&mut crate::cargo_cmd("test", &test_args))?;
    println!("tests were run");

    if skip_bdd {
        let reason = if low_resource_mode { "XTASK_LOW_RESOURCES=1" } else { "XTASK_SKIP_BDD=1" };
        println!("Skipping acceptance tests ({reason}).");
    } else {
        let base = env::var("XTASK_CHANGED_BASE").unwrap_or_else(|_| "origin/main".to_string());
        let (base_ref, changed_files) = get_changed_files(&base)?;
        let bdd_plan = bdd_plan_from_changes(&changed_files)?;
        run_change_aware_bdd(&base_ref, &changed_files, bdd_plan)?;
    }

    println!("\nAll checks passed");
    Ok(())
}

fn run_change_aware_bdd(base_ref: &str, changed_files: &[String], plan: BddPlan) -> Result<()> {
    match plan {
        BddPlan::None { reason } => {
            println!("Skipping acceptance tests ({reason}).");
            Ok(())
        }
        BddPlan::All { reason } => {
            println!("Running acceptance tests ({reason})");
            if !changed_files.is_empty() {
                println!("Changed files vs {}:", base_ref.cyan());
                for file in changed_files {
                    println!("  - {}", file);
                }
            }
            crate::run_cmd(&mut crate::cargo_cmd(
                "test",
                &["-p", "acceptance", "--test", "acceptance"],
            ))?;
            println!("acceptance tests were run");
            Ok(())
        }
        BddPlan::Tags { ac_tags, reason } => {
            if ac_tags.is_empty() {
                println!("Skipping acceptance tests ({reason}).");
                return Ok(());
            }

            let expr = format_tag_expression(&ac_tags);
            println!("Running acceptance tests for changed ACs ({reason}): {}", expr);
            if !changed_files.is_empty() {
                println!("Changed files vs {}:", base_ref.cyan());
                for file in changed_files {
                    println!("  - {}", file);
                }
            }

            let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);
            cmd.env("CUCUMBER_TAG_EXPRESSION", &expr);
            crate::run_cmd(&mut cmd)?;
            println!("acceptance tests were run for: {}", expr);
            Ok(())
        }
    }
}
