use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Custom error type for policy tests
#[derive(Debug)]
pub enum PolicyTestError {
    ConftestNotFound(String),
    Other(anyhow::Error),
}

#[derive(Serialize)]
struct PolicyStatusReport {
    timestamp: String,
    summary: String,
    details: Vec<PolicyResult>,
}

#[derive(Serialize)]
struct PolicyResult {
    name: String,
    status: String,
}

impl std::fmt::Display for PolicyTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyTestError::ConftestNotFound(msg) => write!(f, "{}", msg),
            PolicyTestError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PolicyTestError {}

impl From<anyhow::Error> for PolicyTestError {
    fn from(e: anyhow::Error) -> Self {
        PolicyTestError::Other(e)
    }
}

/// Policy areas to test
const POLICY_AREAS: &[(&str, &str)] = &[
    ("Ledger", "ledger"),
    ("Features", "features"),
    ("Flags", "flags"),
    ("Privacy", "privacy"),
    ("Template Core", "template_core"),
    ("LLM", "llm"),
    ("Kubernetes", "k8s"),
];

/// Run conftest policy tests
pub fn run() -> Result<(), PolicyTestError> {
    // Check if conftest is available
    check_conftest_available()?;

    let workspace_root = get_workspace_root()?;
    let policy_dir = workspace_root.join("policy");
    let testdata_dir = policy_dir.join("testdata");

    if !policy_dir.exists() {
        return Err(PolicyTestError::Other(anyhow::anyhow!(
            "Policy directory not found: {}",
            policy_dir.display()
        )));
    }

    if !testdata_dir.exists() {
        return Err(PolicyTestError::Other(anyhow::anyhow!(
            "Policy testdata directory not found: {}",
            testdata_dir.display()
        )));
    }

    println!("Testing Rego policies...\n");

    let mut total_tests = 0;
    let mut failed_tests = 0;
    let mut policy_results = Vec::new();

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

        let mut area_failed = 0;

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
                    area_failed += 1;
                }
                (false, true) => {
                    println!("  {} {} (expected pass, got fail)", "✗".red(), fixture_name);
                    failed_tests += 1;
                    area_failed += 1;
                }
            };
        }

        policy_results.push(PolicyResult {
            name: name.to_string(),
            status: if area_failed == 0 { "pass".to_string() } else { "fail".to_string() },
        });

        println!();
    }

    // Write policy status JSON
    let report = PolicyStatusReport {
        timestamp: DateTime::<Utc>::from(SystemTime::now()).to_rfc3339(),
        summary: if failed_tests == 0 { "pass".to_string() } else { "fail".to_string() },
        details: policy_results,
    };

    let target_dir = workspace_root.join("target");
    fs::create_dir_all(&target_dir).map_err(|e| PolicyTestError::Other(e.into()))?;

    let json =
        serde_json::to_string_pretty(&report).map_err(|e| PolicyTestError::Other(e.into()))?;
    fs::write(target_dir.join("policy_status.json"), json)
        .map_err(|e| PolicyTestError::Other(e.into()))?;

    // Summary
    if failed_tests == 0 && total_tests > 0 {
        println!("{} All {} policy tests passed!", "✓".green(), total_tests);
        Ok(())
    } else if total_tests == 0 {
        println!("{} No policy tests found", "⚠".yellow());
        Ok(())
    } else {
        println!("{} {} of {} policy test(s) failed", "✗".red(), failed_tests, total_tests);
        Err(PolicyTestError::Other(anyhow::anyhow!("{} policy test(s) failed", failed_tests)))
    }
}

/// Check if conftest is available on PATH
fn check_conftest_available() -> Result<(), PolicyTestError> {
    let output = Command::new("conftest").arg("--version").output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(PolicyTestError::ConftestNotFound(
            "conftest not found on PATH\n\
            \n\
            Install conftest:\n\
              • macOS:     brew install conftest\n\
              • Linux:     See https://www.conftest.dev/install/\n\
              • Nix:       nix develop\n\
              • Container: docker run --rm openpolicyagent/conftest"
                .to_string(),
        )),
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
