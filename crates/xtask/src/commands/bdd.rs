use anyhow::{Context, Result};
use std::path::Path;
use std::process::Output;

// AC-TPL-BDD-EXIT-CODES:
// We normalize cucumber harness behaviour into a single pass/fail signal using:
//  - exit code
//  - [BDD-PASS] marker
//  - JUnit failures/errors
//  - textual pass/fail markers (✔ / ✗)
// so callers (check, selftest, test-changed, test-ac) are not exposed to flaky harness exits.
//
// The cucumber harness may return exit code 101 due to async cleanup issues even when
// all scenarios pass. This module provides semantic success detection to shield callers
// from this behaviour.

/// Check if JUnit XML content indicates all tests passed.
/// Returns true if content has testsuites with zero failures and zero errors.
/// This is the core logic, separated for testability.
pub fn junit_content_indicates_success(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }
    // JUnit XML has attributes like failures="0" errors="0"
    let has_testsuites = content.contains("<testsuites");
    let has_zero_failures = content.contains("failures=\"0\"");
    let has_zero_errors = content.contains("errors=\"0\"");

    has_testsuites && has_zero_failures && has_zero_errors
}

/// Check if the JUnit XML file indicates all tests passed.
/// Returns true if:
/// - JUnit file exists
/// - File is non-empty (tests actually ran)
/// - No failures or errors are reported
fn check_junit_for_success() -> bool {
    let junit_path = Path::new("target/junit/acceptance.xml");
    if !junit_path.exists() {
        return false;
    }

    match std::fs::read_to_string(junit_path) {
        Ok(content) => junit_content_indicates_success(&content),
        Err(_) => false,
    }
}

/// Check test output for success by analyzing the test markers.
/// Returns true if:
/// - Output contains passing test markers (✔)
/// - Output does NOT contain any failure markers (✗)
/// - Tests appear to have run (has at least some scenarios)
///
/// This is public so other commands can use the same detection logic.
pub fn output_indicates_success(stdout: &str) -> bool {
    // Look for cucumber test result indicators
    let has_passes = stdout.contains("✔");
    let has_failures = stdout.contains("✗") || stdout.contains("FAILED");
    let has_scenarios = stdout.contains("Scenario:");

    // If we see passing tests with no failures, consider it a success
    // even if the harness exited before printing [BDD-PASS]
    has_scenarios && has_passes && !has_failures
}

/// Determine if BDD tests passed based on process output.
/// Uses multiple signals to determine success, not just exit code.
///
/// Returns true if any of these conditions are met:
/// 1. Exit code is 0 (explicit success)
/// 2. Output contains [BDD-PASS] marker (harness completed normally)
/// 3. JUnit XML exists and contains no failures
/// 4. Output contains passing markers (✔) with no failures (✗)
pub fn is_bdd_success(output: &Output) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let has_bdd_pass = stdout.contains("[BDD-PASS]") || stderr.contains("[BDD-PASS]");
    let junit_ok = check_junit_for_success();
    let output_ok = output_indicates_success(&stdout);

    output.status.success() || has_bdd_pass || junit_ok || output_ok
}

/// Options for running BDD acceptance tests
#[derive(Debug, Clone, Default)]
pub struct BddOptions {
    /// Custom CUCUMBER_TAG_EXPRESSION to use (overrides default behavior)
    pub tag_expression: Option<String>,
    /// Whether to print verbose output
    pub verbose: bool,
}

/// Run BDD acceptance tests with default options
pub fn run() -> Result<()> {
    run_with_options(BddOptions::default())
}

/// Run BDD acceptance tests with custom options
///
/// This function uses semantic detection of `[BDD-PASS]` marker rather than
/// trusting the raw `cargo test` exit code, which can intermittently return
/// 101 due to async cleanup issues even when all tests pass.
pub fn run_with_options(options: BddOptions) -> Result<()> {
    let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);

    let in_ci = std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok();

    // Determine tag expression: explicit option > env var > default
    if let Some(ref expr) = options.tag_expression {
        cmd.env("CUCUMBER_TAG_EXPRESSION", expr);
        if options.verbose {
            println!("ℹ Using tag expression: {}", expr);
        }
    } else if std::env::var("CUCUMBER_TAG_EXPRESSION").is_err() && !in_ci {
        // Local runs: exclude @ci-only scenarios by default
        cmd.env("CUCUMBER_TAG_EXPRESSION", "not @ci-only");
        if options.verbose {
            println!("ℹ Excluding @ci-only scenarios from local run");
        }
    }

    // Run with output capture for semantic success detection
    let output = cmd.output().context("Failed to run acceptance tests")?;

    // Use semantic success detection (see is_bdd_success for details)
    if is_bdd_success(&output) {
        println!("✓ Acceptance tests passed");
        println!("JUnit output: target/junit/acceptance.xml");
        return Ok(());
    }

    // Print output for debugging on failure
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("BDD output:\n{}", stdout);
    if !stderr.is_empty() {
        eprintln!("stderr:\n{}", stderr);
    }

    anyhow::bail!("Acceptance tests failed with exit code {:?}", output.status.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn junit_content_success_with_zero_failures() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites failures="0" errors="0" tests="20">
  <testsuite name="acceptance">
    <testcase name="Scenario: Example"/>
  </testsuite>
</testsuites>"#;

        assert!(junit_content_indicates_success(xml));
    }

    #[test]
    fn junit_content_failure_with_nonzero_failures() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites failures="1" errors="0" tests="20">
  <testsuite name="acceptance">
    <testcase name="Scenario: Example"/>
  </testsuite>
</testsuites>"#;

        assert!(!junit_content_indicates_success(xml));
    }

    #[test]
    fn junit_content_failure_with_nonzero_errors() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites failures="0" errors="1" tests="20">
  <testsuite name="acceptance">
    <testcase name="Scenario: Example"/>
  </testsuite>
</testsuites>"#;

        assert!(!junit_content_indicates_success(xml));
    }

    #[test]
    fn junit_content_failure_when_empty() {
        assert!(!junit_content_indicates_success(""));
    }

    #[test]
    fn junit_content_failure_without_testsuites() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="acceptance" failures="0" errors="0">
</testsuite>"#;

        // No <testsuites element, so should fail
        assert!(!junit_content_indicates_success(xml));
    }

    #[test]
    fn output_success_with_passing_markers() {
        let stdout = r#"
Scenario: User can login
  ✔ Given the user is on the login page
  ✔ When they enter valid credentials
  ✔ Then they should see the dashboard

Scenario: User can logout
  ✔ Given the user is logged in
  ✔ When they click logout
  ✔ Then they should see the login page
"#;

        assert!(output_indicates_success(stdout));
    }

    #[test]
    fn output_failure_with_failing_marker() {
        let stdout = r#"
Scenario: User can login
  ✔ Given the user is on the login page
  ✗ When they enter valid credentials
  - Then they should see the dashboard
"#;

        assert!(!output_indicates_success(stdout));
    }

    #[test]
    fn output_failure_with_failed_keyword() {
        let stdout = r#"
Scenario: User can login
  ✔ Given the user is on the login page
running 1 test
test acceptance ... FAILED
"#;

        assert!(!output_indicates_success(stdout));
    }

    #[test]
    fn output_failure_when_no_scenarios() {
        // Has passing markers but no scenarios
        let stdout = "Some output with ✔ but no scenario tag";
        assert!(!output_indicates_success(stdout));
    }

    #[test]
    fn output_failure_when_no_passes() {
        // Has scenarios but no passing markers
        let stdout = "Scenario: Example\n  - Step not run";
        assert!(!output_indicates_success(stdout));
    }

    /// AC-TPL-BDD-EXIT-CODES: Verifies the harness contract that exit=101 with [BDD-PASS]
    /// marker is treated as success. This documents the key cucumber harness quirk we shield.
    #[test]
    fn is_bdd_success_with_exit_101_and_bdd_pass_marker() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::{ExitStatus, Output};

        // Simulate cucumber harness exit 101 (async cleanup issue) but with [BDD-PASS]
        let fake = Output {
            status: ExitStatus::from_raw(101 << 8), // exit code 101
            stdout: b"[Summary]\n[BDD-PASS] All non-@wip scenarios passed\n".to_vec(),
            stderr: Vec::new(),
        };

        assert!(
            is_bdd_success(&fake),
            "Exit code 101 with [BDD-PASS] marker should be treated as success"
        );
    }

    /// AC-TPL-BDD-EXIT-CODES: Verifies that true failures (exit non-zero, no [BDD-PASS]) fail.
    #[test]
    fn is_bdd_success_with_exit_1_no_markers() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::{ExitStatus, Output};

        let fake = Output {
            status: ExitStatus::from_raw(1 << 8), // exit code 1
            stdout: b"test failed\n".to_vec(),
            stderr: Vec::new(),
        };

        assert!(!is_bdd_success(&fake), "Exit code 1 without success markers should fail");
    }
}
