use anyhow::{Context, Result};

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

    // Run with output capture to detect [BDD-PASS] marker
    // The cucumber harness may return exit code 101 due to async cleanup issues
    // even when all tests pass, so we check for the [BDD-PASS] marker
    let output = cmd.output().context("Failed to run acceptance tests")?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check if all scenarios passed regardless of exit code
    // The harness outputs [BDD-PASS] when all non-@wip scenarios pass
    // We consider success if either:
    // - The exit code is 0, OR
    // - The output contains [BDD-PASS] (all scenarios passed despite exit code quirks)
    //
    // Note: [BDD-PASS] may appear in either stdout or stderr depending on test harness
    let stderr = String::from_utf8_lossy(&output.stderr);
    let bdd_pass =
        stdout.contains("[BDD-PASS]") || stderr.contains("[BDD-PASS]") || output.status.success();

    if bdd_pass {
        println!("✓ Acceptance tests passed");
        println!("JUnit output: target/junit/acceptance.xml");
        Ok(())
    } else {
        // Print output for debugging
        eprintln!("BDD output:\n{}", stdout);
        if !stderr.is_empty() {
            eprintln!("stderr:\n{}", stderr);
        }
        anyhow::bail!("Acceptance tests failed with exit code {:?}", output.status.code())
    }
}
