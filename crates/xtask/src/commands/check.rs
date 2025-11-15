use anyhow::Result;
use std::process::Command;

/// Run all checks: fmt, clippy, nextest
pub fn run() -> Result<()> {
    println!("Running format check...");
    crate::run_cmd(Command::new("cargo").args(["fmt", "--all", "--", "--check"]))?;

    println!("Running clippy...");
    crate::run_cmd(Command::new("cargo").args([
        "clippy",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ]))?;

    println!("Running tests...");
    crate::run_cmd(Command::new("cargo").args(["test", "--workspace", "--exclude", "acceptance"]))?;

    println!("✓ All checks passed");
    Ok(())
}
