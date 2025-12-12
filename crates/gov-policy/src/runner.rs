//! Conftest policy test runner.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Policy test areas with their file prefixes.
pub const POLICY_AREAS: &[(&str, &str)] = &[
    ("Ledger", "ledger"),
    ("Features", "features"),
    ("Flags", "flags"),
    ("Privacy", "privacy"),
    ("Template Core", "template_core"),
    ("LLM", "llm"),
    ("Kubernetes", "k8s"),
];

/// Error types for policy testing.
#[derive(Debug, thiserror::Error)]
pub enum PolicyTestError {
    #[error("conftest not found - install via 'brew install conftest' or 'nix develop'")]
    ConftestNotFound,
    #[error("policy test failed: {0}")]
    TestFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of running policy tests.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PolicyTestResult {
    pub area: String,
    pub passed: bool,
    pub output: String,
}

/// Check if conftest is available.
pub fn check_conftest_available() -> Result<(), PolicyTestError> {
    match Command::new("conftest").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(PolicyTestError::ConftestNotFound),
    }
}

/// Run policy tests against fixtures in the given policy directory.
pub fn run_policy_tests(policy_dir: &Path) -> Result<Vec<PolicyTestResult>, PolicyTestError> {
    check_conftest_available()?;

    let mut results = Vec::new();
    let testdata_dir = policy_dir.join("testdata");

    for (area_name, area_prefix) in POLICY_AREAS {
        let policy_file = policy_dir.join(format!("{area_prefix}.rego"));
        if !policy_file.exists() {
            continue;
        }

        // Find valid fixture
        let valid_fixture = testdata_dir.join(format!("{area_prefix}_valid.json"));
        if !valid_fixture.exists() {
            continue;
        }

        let output = Command::new("conftest")
            .args(["test", "-p"])
            .arg(&policy_file)
            .arg(&valid_fixture)
            .output()?;

        results.push(PolicyTestResult {
            area: area_name.to_string(),
            passed: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        });
    }

    Ok(results)
}

/// Find test fixtures for a policy area.
pub fn find_test_fixtures(testdata_dir: &Path, area: &str) -> Result<Vec<(PathBuf, bool)>> {
    let mut fixtures = Vec::new();

    // Look for valid and invalid fixtures (both JSON and YAML)
    let valid_json = testdata_dir.join(format!("{}_valid.json", area));
    let valid_yaml = testdata_dir.join(format!("{}_valid.yaml", area));
    let invalid_json = testdata_dir.join(format!("{}_invalid.json", area));
    let invalid_yaml = testdata_dir.join(format!("{}_invalid.yaml", area));

    // Also check for alternative naming patterns - these all should fail validation
    let invalid_patterns = [
        "missing_tests",
        "no_tests",
        "missing_ac",
        "unknown_ac",
        "wrong_feature",
        "missing_include",
        "zero_bytes",
        "missing_required_task",
        "unknown_field",
        "missing_max_bytes",
        "runs_as_root",
        "no_labels",
        "literal_database_url",
        "literal_api_key",
        "configmap_for_secret",
    ];

    if valid_json.exists() {
        fixtures.push((valid_json, true));
    }

    if valid_yaml.exists() {
        fixtures.push((valid_yaml, true));
    }

    if invalid_json.exists() {
        fixtures.push((invalid_json, false));
    }

    if invalid_yaml.exists() {
        fixtures.push((invalid_yaml, false));
    }

    // Check for all invalid patterns (both JSON and YAML)
    for pattern in &invalid_patterns {
        let json_file = testdata_dir.join(format!("{}_{}.json", area, pattern));
        if json_file.exists() {
            fixtures.push((json_file, false));
        }

        let yaml_file = testdata_dir.join(format!("{}_{}.yaml", area, pattern));
        if yaml_file.exists() {
            fixtures.push((yaml_file, false));
        }
    }

    Ok(fixtures)
}

/// Run conftest test against a policy file and fixture.
pub fn run_conftest_test(policy_file: &Path, fixture_file: &Path) -> Result<bool> {
    let output = Command::new("conftest")
        .arg("test")
        .arg("-p")
        .arg(policy_file)
        .arg(fixture_file)
        .output()?;

    Ok(output.status.success())
}
