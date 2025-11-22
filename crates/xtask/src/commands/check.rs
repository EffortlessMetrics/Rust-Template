use anyhow::Result;

/// Run all checks: fmt, clippy, nextest
pub fn run() -> Result<()> {
    println!("Running format check (fmt)...");
    crate::run_cmd(&mut crate::cargo_cmd("fmt", &["--all", "--", "--check"]))?;
    println!("✓ fmt was checked");

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
    println!("✓ clippy was checked");

    println!("Running tests...");
    // On Windows, exclude xtask to avoid file locking issues (can't rebuild running binary)
    let test_args = if cfg!(windows) {
        vec!["--workspace", "--exclude", "acceptance", "--exclude", "xtask"]
    } else {
        vec!["--workspace", "--exclude", "acceptance"]
    };
    crate::run_cmd(&mut crate::cargo_cmd("test", &test_args))?;
    println!("✓ tests were run");

    println!("\n✓ All checks passed");
    Ok(())
}
