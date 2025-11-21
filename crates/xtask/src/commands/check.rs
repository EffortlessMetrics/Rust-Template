use anyhow::Result;

/// Run all checks: fmt, clippy, nextest
pub fn run() -> Result<()> {
    println!("Running format check...");
    crate::run_cmd(&mut crate::cargo_cmd("fmt", &["--all", "--", "--check"]))?;

    println!("Running clippy...");
    crate::run_cmd(&mut crate::cargo_cmd(
        "clippy",
        &["--all-targets", "--all-features", "--", "-D", "warnings"],
    ))?;

    println!("Running tests...");
    crate::run_cmd(&mut crate::cargo_cmd("test", &["--workspace", "--exclude", "acceptance"]))?;

    println!("✓ All checks passed");
    Ok(())
}
