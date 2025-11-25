use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run_with_default_targets() -> Result<()> {
    run(&["docs", "specs", "README.md", "CLAUDE.md"])
}

pub fn run(targets: &[&str]) -> Result<()> {
    println!("{}", "Running spellcheck (cspell)...".blue().bold());

    // Prefer a globally available `cspell`; fall back to `npx cspell` if needed.
    let mut cmd = if let Ok(path) = which::which("cspell") {
        let mut c = Command::new(path);
        c
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

    cmd.args([
        "--no-progress",
        "--config",
        "cspell.json",
        "--relative",
        "--gitignore",
    ]);
    cmd.args(targets);

    crate::run_cmd(&mut cmd)
}
