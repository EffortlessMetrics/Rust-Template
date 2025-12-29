use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run_with_default_targets() -> Result<()> {
    run(&["docs", "specs", "README.md", "CLAUDE.md"])
}

/// Run spellcheck only on the specified files (for fast, targeted checking)
pub fn run_for_files(files: &[String]) -> Result<()> {
    if files.is_empty() {
        println!("{} No files to spellcheck", "⊘".cyan());
        return Ok(());
    }

    let targets: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    run(&targets)
}

pub fn run(targets: &[&str]) -> Result<()> {
    println!("{}", "Running spellcheck (cspell)...".blue().bold());

    // Prefer a globally available `cspell`; fall back to `npx cspell` if needed.
    let mut cmd = if let Ok(path) = which::which("cspell") {
        if let Some(path_str) = path.to_str()
            && path_str.starts_with("/mnt/c/")
        {
            println!(
                "{} Using cspell from {}; prefer the Nix devShell binary for Tier-1 checks",
                "[WARN]".yellow(),
                path_str
            );
        }

        Command::new(path)
    } else {
        if which::which("npx").is_err() {
            println!(
                "{} cspell is not available (install or run via Nix); skipping spellcheck",
                "[WARN]".yellow()
            );
            return Ok(());
        }

        let mut c = Command::new("npx");
        c.arg("cspell");
        c
    };

    cmd.args(["--no-progress", "--config", "cspell.json", "--relative", "--gitignore"]);
    cmd.args(targets);

    crate::run_cmd(&mut cmd)
}
