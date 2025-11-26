use anyhow::Result;

/// Run BDD acceptance tests
pub fn run() -> Result<()> {
    println!("Running acceptance tests...");

    // Exclude @ci-only scenarios from local runs unless CI environment is detected
    // or CUCUMBER_TAG_EXPRESSION is explicitly set
    let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);

    // Only apply the default filter if no tag expression is already set
    if std::env::var("CUCUMBER_TAG_EXPRESSION").is_err() {
        // Check if running in CI (common CI env vars)
        let is_ci = std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok();

        if !is_ci {
            // Exclude @ci-only scenarios from local development runs
            cmd.env("CUCUMBER_TAG_EXPRESSION", "not @ci-only");
            println!("ℹ Excluding @ci-only scenarios from local run");
            println!("  (Set CUCUMBER_TAG_EXPRESSION to override)");
        }
    }

    crate::run_cmd(&mut cmd)?;

    println!("✓ Acceptance tests passed");
    println!("JUnit output: target/junit/acceptance.xml");
    Ok(())
}
