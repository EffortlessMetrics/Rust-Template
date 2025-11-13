use anyhow::Result;
use std::process::Command;

/// Run BDD acceptance tests
pub fn run() -> Result<()> {
    println!("Running acceptance tests...");
    crate::run_cmd(
        Command::new("cargo")
            .args(["test", "-p", "acceptance", "--test", "acceptance"])
    )?;

    println!("✓ Acceptance tests passed");
    println!("JUnit output: target/junit/acceptance.xml");
    Ok(())
}
