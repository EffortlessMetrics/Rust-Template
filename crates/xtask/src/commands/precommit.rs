use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "Running pre-commit checks...".blue().bold());

    // 0. Auto-fix fmt first, then let check run on clean tree
    run_fmt_with_autostage()?;

    // 1. Core checks (fmt --check, clippy, tests)
    crate::commands::check::run()?;

    // Run Skills format and lint if relevant files changed
    run_skills_lint_if_changed()?;

    // Run Agents lint if relevant files changed
    run_agents_lint_if_changed()?;

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

fn stage_skill_docs_if_modified() -> Result<()> {
    let out = Command::new("git")
        .args(["diff", "--name-only", "--", ".claude/skills"])
        .output()
        .context("failed to run git diff for skills")?;

    let changed = String::from_utf8_lossy(&out.stdout);
    let files: Vec<&str> = changed.lines().map(str::trim).filter(|p| !p.is_empty()).collect();

    if files.is_empty() {
        return Ok(());
    }

    println!("{} Staging formatted Skills:", "✓".green());
    for f in &files {
        println!("  - {}", f);
        Command::new("git").args(["add", f]).status()?;
    }
    Ok(())
}

fn run_skills_lint_if_changed() -> Result<()> {
    // Check if any Skills-related files have changed (both staged and unstaged)
    let paths_to_check = [
        ".claude/skills/**",
        "docs/SKILLS_*.md",
        "specs/spec_ledger.yaml",
        "crates/xtask/src/commands/skills.rs",
    ];

    let mut has_changes = false;

    for pattern in &paths_to_check {
        // Check staged changes
        let staged_output = Command::new("git")
            .args(["diff", "--cached", "--name-only", "--", pattern])
            .output()?;

        let staged_str = String::from_utf8_lossy(&staged_output.stdout);
        if !staged_str.trim().is_empty() {
            has_changes = true;
            break;
        }

        // Check unstaged changes
        let unstaged_output =
            Command::new("git").args(["diff", "--name-only", "--", pattern]).output()?;

        let unstaged_str = String::from_utf8_lossy(&unstaged_output.stdout);
        if !unstaged_str.trim().is_empty() {
            has_changes = true;
            break;
        }
    }

    if !has_changes {
        println!("{} No Skills changes detected, skipping skills-lint", "⊘".cyan());
        return Ok(());
    }

    // Run Skills format first
    match crate::commands::skills::run_fmt() {
        Ok(_) => {}
        Err(_) => {
            // Skills fmt exits with code 1 if files were modified, which is expected
            println!("{} Skills format applied (files were modified)", "✓".green());
        }
    }

    // Auto-stage any formatted SKILL.md files
    stage_skill_docs_if_modified()?;

    // Then run lint (with errors causing failure)
    match crate::commands::skills::run_lint() {
        Ok(_) => {
            println!("{} Skills governance check passed", "✓".green());
        }
        Err(e) => {
            println!("{} Skills governance check failed: {}", "✗".red(), e);
            return Err(e);
        }
    }

    Ok(())
}

fn stage_agent_docs_if_modified() -> Result<()> {
    let out = Command::new("git")
        .args(["diff", "--name-only", "--", ".claude/agents"])
        .output()
        .context("failed to run git diff for agents")?;

    let changed = String::from_utf8_lossy(&out.stdout);
    let files: Vec<&str> = changed.lines().map(str::trim).filter(|p| !p.is_empty()).collect();

    if files.is_empty() {
        return Ok(());
    }

    println!("{} Staging formatted Agents:", "✓".green());
    for f in &files {
        println!("  - {}", f);
        Command::new("git").args(["add", f]).status()?;
    }
    Ok(())
}

fn run_agents_lint_if_changed() -> Result<()> {
    // Check if any Agents-related files have changed (both staged and unstaged)
    let paths_to_check = [
        ".claude/agents/**",
        "docs/AGENTS_*.md",
        "specs/spec_ledger.yaml",
        "crates/xtask/src/commands/agents.rs",
    ];

    let mut has_changes = false;

    for pattern in &paths_to_check {
        // Check staged changes
        let staged_output = Command::new("git")
            .args(["diff", "--cached", "--name-only", "--", pattern])
            .output()?;

        let staged_str = String::from_utf8_lossy(&staged_output.stdout);
        if !staged_str.trim().is_empty() {
            has_changes = true;
            break;
        }

        // Check unstaged changes
        let unstaged_output =
            Command::new("git").args(["diff", "--name-only", "--", pattern]).output()?;

        let unstaged_str = String::from_utf8_lossy(&unstaged_output.stdout);
        if !unstaged_str.trim().is_empty() {
            has_changes = true;
            break;
        }
    }

    if !has_changes {
        println!("{} No Agents changes detected, skipping agents-lint", "⊘".cyan());
        return Ok(());
    }

    // Run Agents format first
    match crate::commands::agents::run_fmt() {
        Ok(_) => {}
        Err(_) => {
            // Agents fmt exits with code 1 if files were modified, which is expected
            println!("{} Agents format applied (files were modified)", "✓".green());
        }
    }

    // Auto-stage any formatted agent files
    stage_agent_docs_if_modified()?;

    // Then run lint (with errors causing failure)
    match crate::commands::agents::run_lint() {
        Ok(_) => {
            println!("{} Agents governance check passed", "✓".green());
        }
        Err(e) => {
            println!("{} Agents governance check failed: {}", "✗".red(), e);
            return Err(e);
        }
    }

    Ok(())
}

fn run_ac_status_with_autostage() -> Result<()> {
    // Check if coverage exists - if not, skip regeneration to prevent churn.
    // Without fresh coverage, we'd regenerate feature_status.md with many [UNKNOWN] entries,
    // which would then differ from the committed state and cause spurious diffs.
    let layout = crate::kernel::layout_for_repo();

    if !layout.has_coverage() {
        println!("{} Skipping AC status regeneration: coverage.jsonl missing", "[WARN]".yellow());
        println!("  hint: Run 'cargo xtask bdd' to generate coverage first.");
        println!("  💡 feature_status.md will be validated (not regenerated) in selftest.");
        return Ok(());
    }

    // Run AC status to regenerate docs/feature_status.md
    // We use require_coverage: true to ensure we don't proceed with stale data
    match crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        require_coverage: true, // Guard against regenerating with missing coverage
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

fn run_fmt_with_autostage() -> Result<()> {
    println!("{}", "Running cargo fmt (auto-fix)…".blue());

    let status = Command::new("cargo")
        .args(["fmt", "--all"])
        .status()
        .context("failed to run `cargo fmt --all`")?;

    if !status.success() {
        // fmt failure is almost always a syntax error, so we block
        println!("{} `cargo fmt` failed – fix syntax errors before committing", "[FAIL]".red());
        anyhow::bail!("`cargo fmt` failed");
    }

    // Stage any Rust files that changed due to formatting
    let diff = Command::new("git")
        .args(["diff", "--name-only", "--", "*.rs", "*.rs.in"])
        .output()
        .context("failed to run `git diff` for fmt")?;

    let changed = String::from_utf8_lossy(&diff.stdout);
    let files: Vec<&str> = changed.lines().map(str::trim).filter(|p| !p.is_empty()).collect();

    if files.is_empty() {
        println!("{} No Rust formatting changes", "⊘".cyan());
    } else {
        println!("{} Staging formatted Rust files:", "✓".green());
        for f in &files {
            println!("  - {}", f);
            Command::new("git").args(["add", f]).status()?;
        }
    }

    Ok(())
}
