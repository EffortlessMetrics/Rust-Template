use anyhow::Result;

/// Run BDD acceptance tests
pub fn run() -> Result<()> {
    println!("Running acceptance tests...");
    crate::run_cmd(&mut crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]))?;

    println!("✓ Acceptance tests passed");
    println!("JUnit output: target/junit/acceptance.xml");
    Ok(())
}
