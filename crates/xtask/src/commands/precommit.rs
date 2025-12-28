use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

/// Detect what categories of files have changed
#[derive(Debug, Default)]
struct ChangeCategories {
    rust: bool,
    docs: bool,
    claude: bool,
    specs: bool,
}

impl ChangeCategories {
    fn detect(staged_only: bool) -> Result<Self> {
        let changed = get_changed_files(staged_only)?;
        let mut cats = ChangeCategories::default();

        for file in &changed {
            if file.ends_with(".rs") {
                cats.rust = true;
            }
            if file.starts_with("docs/")
                || file.ends_with(".md")
                || file == "README.md"
                || file == "CLAUDE.md"
            {
                cats.docs = true;
            }
            if file.starts_with(".claude/") {
                cats.claude = true;
            }
            if file.starts_with("specs/") || file.ends_with(".feature") {
                cats.specs = true;
            }
        }

        Ok(cats)
    }
}

/// Run git with --name-only and return the list of files, failing loudly on errors
fn git_name_only(args: &[&str], what: &str) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run git for {}", what))?;

    if !out.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn get_changed_files(staged_only: bool) -> Result<Vec<String>> {
    // Get staged changes - fail loudly if git isn't available or we're outside a repo
    let mut files = git_name_only(&["diff", "--cached", "--name-only"], "staged changes")?;

    // Also include unstaged changes unless staged_only is set
    if !staged_only {
        files.extend(git_name_only(&["diff", "--name-only"], "unstaged changes")?);
    }

    files.sort();
    files.dedup();
    Ok(files)
}

pub fn run(mode: &str, staged_only: bool) -> Result<()> {
    match mode {
        "full" => run_full(),
        _ => run_fast(staged_only),
    }
}

/// Fast mode: change-aware routing
fn run_fast(staged_only: bool) -> Result<()> {
    let mode_desc = if staged_only { "fast mode, staged only" } else { "fast mode" };
    println!("{}", format!("Running pre-commit checks ({})...", mode_desc).blue().bold());

    let cats = ChangeCategories::detect(staged_only)?;

    if !cats.rust && !cats.docs && !cats.claude && !cats.specs {
        println!("{} No significant changes detected", "⊘".cyan());
        println!("{}", "Pre-commit checks completed".green().bold());
        return Ok(());
    }

    println!(
        "  Changes: {}{}{}{}",
        if cats.rust { "rust " } else { "" },
        if cats.docs { "docs " } else { "" },
        if cats.claude { "claude " } else { "" },
        if cats.specs { "specs " } else { "" }
    );

    // Rust changes: fmt + clippy + test-changed
    // In staged-only mode, skip Rust compilation checks to avoid tripping on unstaged WIP
    if cats.rust {
        if staged_only {
            println!(
                "{} Rust changes staged; fast --staged-only skips fmt/clippy/tests to avoid tripping on unstaged WIP.",
                "[WARN]".yellow()
            );
            println!("  💡 Run: cargo xtask precommit --mode full  (receipt-grade)");
        } else {
            run_fmt_with_autostage()?;
            run_clippy()?;
            run_test_changed()?;
        }
    }

    // Claude changes: skills/agents lint
    if cats.claude {
        run_skills_lint_if_changed()?;
        run_agents_lint_if_changed()?;
    }

    // Specs/features changes: ac-status regen
    if cats.specs {
        run_ac_status_with_autostage()?;
    }

    // Docs or specs changes: docs-check (run once for either)
    if cats.docs || cats.specs {
        run_docs_check_soft()?;
    }

    // Docs changes: spellcheck
    if cats.docs {
        run_spellcheck_soft()?;
    }

    println!("{}", "Pre-commit checks completed".green().bold());
    Ok(())
}

/// Full mode: all checks (current behavior)
fn run_full() -> Result<()> {
    println!("{}", "Running pre-commit checks (full mode)...".blue().bold());

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
    run_docs_check_soft()?;
    run_spellcheck_soft()?;

    println!("{}", "Pre-commit checks completed".green().bold());
    Ok(())
}

fn run_clippy() -> Result<()> {
    println!("{}", "Running clippy...".blue());
    let status = Command::new("cargo")
        .args(["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
        .status()
        .context("failed to run clippy")?;

    if !status.success() {
        anyhow::bail!("clippy found warnings");
    }
    Ok(())
}

fn run_test_changed() -> Result<()> {
    println!("{}", "Running test-changed...".blue());
    crate::commands::test_changed::run(crate::commands::test_changed::TestChangedArgs::default())?;
    Ok(())
}

fn run_docs_check_soft() -> Result<()> {
    let strict = std::env::var("XTASK_STRICT_PRECOMMIT").unwrap_or_default() == "1";

    if std::env::var("XTASK_SKIP_DOCS_CHECK").unwrap_or_default() == "1" {
        println!("{} Skipping docs-check because XTASK_SKIP_DOCS_CHECK=1", "[WARN]".yellow());
        return Ok(());
    }

    if let Err(e) = crate::commands::docs_check::run() {
        if strict {
            println!("{} docs-check failed (strict mode enabled)", "[FAIL]".red());
            return Err(e);
        }
        println!("{} docs-check failed (continuing in soft mode)", "[WARN]".yellow());
        println!("  {}", e.to_string().lines().next().unwrap_or(""));
        println!("  💡 To fail on docs issues: XTASK_STRICT_PRECOMMIT=1");
    }
    Ok(())
}

fn run_spellcheck_soft() -> Result<()> {
    let strict = std::env::var("XTASK_STRICT_PRECOMMIT").unwrap_or_default() == "1";

    if std::env::var("XTASK_SKIP_SPELLCHECK").unwrap_or_default() == "1" {
        println!("{} Skipping spellcheck because XTASK_SKIP_SPELLCHECK=1", "[WARN]".yellow());
        return Ok(());
    }

    if let Err(e) = crate::commands::spellcheck::run_with_default_targets() {
        if strict {
            println!("{} spellcheck failed (strict mode enabled)", "[FAIL]".red());
            return Err(e);
        }
        println!("{} spellcheck failed (continuing in soft mode)", "[WARN]".yellow());
        println!("  {}", e.to_string().lines().next().unwrap_or(""));
        println!("  💡 To fail on spelling issues: XTASK_STRICT_PRECOMMIT=1");
    }
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
