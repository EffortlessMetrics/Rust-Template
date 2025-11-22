use anyhow::Result;

/// Run all checks: fmt, clippy, nextest
pub fn run() -> Result<()> {
    println!("Running format check (fmt)...");
    crate::run_cmd(&mut crate::cargo_cmd("fmt", &["--all", "--", "--check"]))?;
    println!("✓ fmt was checked");

    println!("Running clippy...");
    crate::run_cmd(&mut crate::cargo_cmd(
        "clippy",
        &["--all-targets", "--all-features", "--", "-D", "warnings"],
    ))?;
    println!("✓ clippy was checked");

    println!("Running tests...");
    crate::run_cmd(&mut crate::cargo_cmd("test", &["--workspace", "--exclude", "acceptance"]))?;
    println!("✓ tests were run");

    println!("\n✓ All checks passed");
    Ok(())
}
