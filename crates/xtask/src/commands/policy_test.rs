use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Policy areas to test
const POLICY_AREAS: &[(&str, &str)] =
    &[("Ledger", "ledger"), ("Features", "features"), ("Flags", "flags"), ("Privacy", "privacy")];

/// Run conftest policy tests
pub fn run() -> Result<()> {
    // Check if conftest is available
    check_conftest_available()?;

    let workspace_root = get_workspace_root()?;
    let policy_dir = workspace_root.join("policy");
    let testdata_dir = policy_dir.join("testdata");

    if !policy_dir.exists() {
        anyhow::bail!("Policy directory not found: {}", policy_dir.display());
    }

    if !testdata_dir.exists() {
        anyhow::bail!("Policy testdata directory not found: {}", testdata_dir.display());
    }

    println!("Testing Rego policies...\n");

    let mut total_tests = 0;
    let mut failed_tests = 0;

    // Run tests for each policy area
    for (name, area) in POLICY_AREAS {
        let policy_file = policy_dir.join(format!("{}.rego", area));

        if !policy_file.exists() {
            println!("{} Policy ({}):", name, policy_file.display());
            println!("  {} Policy file not found, skipping\n", "⚠".yellow());
            continue;
        }

        println!("{} Policy ({}):", name, policy_file.display());

        // Find test fixtures for this policy area
        let fixtures = find_test_fixtures(&testdata_dir, area)?;

        if fixtures.is_empty() {
            println!("  {} No test fixtures found, skipping\n", "⚠".yellow());
            continue;
        }

        // Run tests for each fixture
        for (fixture_path, should_pass) in fixtures {
            let fixture_name =
                fixture_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

            let result = run_conftest_test(&policy_file, &fixture_path)?;

            total_tests += 1;

            match (result, should_pass) {
                (true, true) => {
                    println!("  {} {} (correctly passed)", "✓".green(), fixture_name);
                }
                (false, false) => {
                    println!("  {} {} (correctly failed)", "✓".green(), fixture_name);
                }
                (true, false) => {
                    println!("  {} {} (expected fail, got pass)", "✗".red(), fixture_name);
                    failed_tests += 1;
                }
                (false, true) => {
                    println!("  {} {} (expected pass, got fail)", "✗".red(), fixture_name);
                    failed_tests += 1;
                }
            };
        }

        println!();
    }

    // Summary
    if failed_tests == 0 && total_tests > 0 {
        println!("{} All {} policy tests passed!", "✓".green(), total_tests);
        Ok(())
    } else if total_tests == 0 {
        println!("{} No policy tests found", "⚠".yellow());
        Ok(())
    } else {
        println!("{} {} of {} policy test(s) failed", "✗".red(), failed_tests, total_tests);
        anyhow::bail!("{} policy test(s) failed", failed_tests)
    }
}

/// Check if conftest is available on PATH
fn check_conftest_available() -> Result<()> {
    let output = Command::new("conftest").arg("--version").output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => {
            anyhow::bail!(
                "conftest not found on PATH\n\
                \n\
                Install conftest:\n\
                  • macOS:     brew install conftest\n\
                  • Linux:     See https://www.conftest.dev/install/\n\
                  • Nix:       nix develop\n\
                  • Container: docker run --rm openpolicyagent/conftest"
            )
        }
    }
}

/// Get workspace root directory
fn get_workspace_root() -> Result<PathBuf> {
    // From xtask directory, workspace root is ../../
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Try to find Cargo.toml in workspace root
    let mut check_dir = current_dir.clone();
    for _ in 0..3 {
        let cargo_toml = check_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            // Verify it's the workspace root by checking for policy directory
            if check_dir.join("policy").exists() {
                return Ok(check_dir);
            }
        }
        if let Some(parent) = check_dir.parent() {
            check_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Fallback: assume we're running from workspace root
    Ok(current_dir)
}

/// Find test fixtures for a policy area
fn find_test_fixtures(testdata_dir: &Path, area: &str) -> Result<Vec<(PathBuf, bool)>> {
    let mut fixtures = Vec::new();

    // Look for valid and invalid fixtures
    let valid_file = testdata_dir.join(format!("{}_valid.json", area));
    let invalid_file = testdata_dir.join(format!("{}_invalid.json", area));

    // Also check for alternative naming patterns
    let missing_tests_file = testdata_dir.join(format!("{}_missing_tests.json", area));
    let unknown_ac_file = testdata_dir.join(format!("{}_unknown_ac.json", area));

    if valid_file.exists() {
        fixtures.push((valid_file, true));
    }

    if invalid_file.exists() {
        fixtures.push((invalid_file, false));
    }

    if missing_tests_file.exists() {
        fixtures.push((missing_tests_file, false));
    }

    if unknown_ac_file.exists() {
        fixtures.push((unknown_ac_file, false));
    }

    Ok(fixtures)
}

/// Run conftest test against a policy file and fixture
fn run_conftest_test(policy_file: &Path, fixture_file: &Path) -> Result<bool> {
    let output = Command::new("conftest")
        .arg("test")
        .arg("-p")
        .arg(policy_file)
        .arg(fixture_file)
        .output()
        .context("Failed to execute conftest")?;

    Ok(output.status.success())
}
