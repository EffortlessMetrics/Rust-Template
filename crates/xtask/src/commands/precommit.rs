use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "Running pre-commit checks...".blue().bold());

    crate::commands::check::run()?;

    if std::env::var("XTASK_SKIP_DOCS_CHECK").unwrap_or_default() == "1" {
        println!("{} Skipping docs-check because XTASK_SKIP_DOCS_CHECK=1", "[WARN]".yellow());
    } else {
        crate::commands::docs_check::run()?;
    }

    if std::env::var("XTASK_SKIP_SPELLCHECK").unwrap_or_default() == "1" {
        println!("{} Skipping spellcheck because XTASK_SKIP_SPELLCHECK=1", "[WARN]".yellow());
    } else {
        crate::commands::spellcheck::run_with_default_targets()?;
    }

    println!("{}", "Pre-commit checks completed".green().bold());
    Ok(())
}
