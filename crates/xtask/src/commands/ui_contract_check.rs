//! UI Contract Check Command
//!
//! Validates that the UI contract specification (specs/ui_contract.yaml) is consistent
//! and that HTML pages render the correct `data-uiid` attributes.
//!
//! @AC-TPL-PLATFORM-UI-CONTRACT: UI spec + anchors + endpoint are governed
//!
//! ## Checks performed
//!
//! 1. **YAML Parse**: Contract file loads without errors
//! 2. **Schema Validation**: All required fields present, unique IDs
//! 3. **Region Kind References**: All region kinds reference defined kinds
//! 4. **DOM Validation**: Runs integration tests to verify HTML has data-uiid attributes
//!
//! ## Usage
//!
//! ```bash
//! cargo xtask ui-contract-check
//! ```

use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of a single check step
#[derive(Debug, PartialEq)]
enum CheckResult {
    Pass,
    Fail(String),
    Skip(String),
}

fn print_result(result: &CheckResult) {
    match result {
        CheckResult::Pass => println!("{}", "✓".green().bold()),
        CheckResult::Fail(_) => println!("{}", "✗".red().bold()),
        CheckResult::Skip(_) => println!("{}", "⊘".yellow()),
    }
}

/// Run UI contract validation checks
///
/// This is the main entry point called from xtask main.rs
pub fn run() -> Result<()> {
    println!("{}", "🎨 Validating UI contract...".blue().bold());
    println!();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let mut results: Vec<(&str, CheckResult)> = Vec::new();

    // Step 1: Load and parse YAML
    print!("  {} UI contract YAML ", "[1/4]".dimmed());
    std::io::stdout().flush().ok();
    let yaml_result = check_yaml_parse(root);
    print_result(&yaml_result);
    let contract = match &yaml_result {
        CheckResult::Pass => {
            Some(spec_runtime::load_ui_contract(&root.join("specs/ui_contract.yaml")).unwrap())
        }
        _ => None,
    };
    results.push(("UI contract YAML", yaml_result));

    // Step 2: Validate schema/structure
    print!("  {} Schema validation ", "[2/4]".dimmed());
    std::io::stdout().flush().ok();
    let schema_result = if let Some(ref contract) = contract {
        check_schema_validation(contract)
    } else {
        CheckResult::Skip("YAML parse failed".to_string())
    };
    print_result(&schema_result);
    results.push(("Schema validation", schema_result));

    // Step 3: Region kind references
    print!("  {} Region kind refs ", "[3/4]".dimmed());
    std::io::stdout().flush().ok();
    let region_kind_result = if let Some(ref contract) = contract {
        check_region_kind_references(contract)
    } else {
        CheckResult::Skip("YAML parse failed".to_string())
    };
    print_result(&region_kind_result);
    results.push(("Region kind refs", region_kind_result));

    // Step 4: DOM validation tests
    print!("  {} DOM validation tests ", "[4/4]".dimmed());
    std::io::stdout().flush().ok();
    let dom_result = run_dom_validation_tests(root);
    print_result(&dom_result);
    results.push(("DOM validation", dom_result));

    println!();

    // Summary
    let failures: Vec<_> =
        results.iter().filter(|(_, r)| matches!(r, CheckResult::Fail(_))).collect();
    let skipped: Vec<_> =
        results.iter().filter(|(_, r)| matches!(r, CheckResult::Skip(_))).collect();

    if !skipped.is_empty() {
        for (name, result) in &skipped {
            if let CheckResult::Skip(reason) = result {
                println!("  {} {} skipped ({})", "⚠".yellow(), name, reason.dimmed());
            }
        }
    }

    if failures.is_empty() {
        println!("{}", "UI contract validation PASSED".green().bold());
        Ok(())
    } else {
        println!("{}", "UI contract validation FAILED".red().bold());
        println!();
        for (name, result) in &failures {
            if let CheckResult::Fail(reason) = result {
                println!("  {} {}: {}", "✗".red(), name, reason);
            }
        }
        println!();
        println!(
            "{}",
            "Fix the contract (specs/ui_contract.yaml) or HTML templates to match.".dimmed()
        );
        anyhow::bail!("UI contract validation failed")
    }
}

/// Check 1: YAML file loads without errors
fn check_yaml_parse(root: &Path) -> CheckResult {
    let contract_path = root.join("specs/ui_contract.yaml");

    if !contract_path.exists() {
        return CheckResult::Fail(format!("Contract file not found: {}", contract_path.display()));
    }

    match spec_runtime::load_ui_contract(&contract_path) {
        Ok(_) => CheckResult::Pass,
        Err(e) => CheckResult::Fail(format!("Failed to parse: {}", e)),
    }
}

/// Check 2: Schema validation (unique IDs, required fields)
fn check_schema_validation(contract: &spec_runtime::UiContract) -> CheckResult {
    let mut errors: Vec<String> = Vec::new();

    // Check version fields
    if contract.schema_version.is_empty() {
        errors.push("Missing schema_version".to_string());
    }
    if contract.template_version.is_empty() {
        errors.push("Missing template_version".to_string());
    }

    // Check for empty screens
    if contract.screens.is_empty() {
        errors.push("No screens defined".to_string());
    }

    // Check for unique screen IDs
    let mut screen_ids: HashSet<String> = HashSet::new();
    for screen in &contract.screens {
        if !screen_ids.insert(screen.id.clone()) {
            errors.push(format!("Duplicate screen ID: {}", screen.id));
        }

        // Check each screen has required fields
        if screen.route.is_empty() {
            errors.push(format!("Screen '{}' missing route", screen.id));
        }
        if screen.description.is_empty() {
            errors.push(format!("Screen '{}' missing description", screen.id));
        }

        // Check for unique region IDs within screen
        let mut region_ids: HashSet<String> = HashSet::new();
        for region in &screen.regions {
            if !region_ids.insert(region.id.clone()) {
                errors
                    .push(format!("Duplicate region ID '{}' in screen '{}'", region.id, screen.id));
            }

            // Check region has required fields
            if region.kind.is_empty() {
                errors
                    .push(format!("Region '{}' in screen '{}' missing kind", region.id, screen.id));
            }
            if region.description.is_empty() {
                errors.push(format!(
                    "Region '{}' in screen '{}' missing description",
                    region.id, screen.id
                ));
            }
        }
    }

    // Check region_kinds map is not empty
    if contract.region_kinds.is_empty() {
        errors.push("No region_kinds defined".to_string());
    }

    if errors.is_empty() { CheckResult::Pass } else { CheckResult::Fail(errors.join("; ")) }
}

/// Check 3: All region kinds reference defined kinds
fn check_region_kind_references(contract: &spec_runtime::UiContract) -> CheckResult {
    let mut errors: Vec<String> = Vec::new();

    let defined_kinds: HashSet<&str> = contract.region_kinds.keys().map(|s| s.as_str()).collect();

    for screen in &contract.screens {
        for region in &screen.regions {
            if !defined_kinds.contains(region.kind.as_str()) {
                errors.push(format!(
                    "Region '{}' in screen '{}' references undefined kind '{}'",
                    region.id, screen.id, region.kind
                ));
            }
        }
    }

    if errors.is_empty() { CheckResult::Pass } else { CheckResult::Fail(errors.join("; ")) }
}

/// Check 4: Run DOM validation integration tests
fn run_dom_validation_tests(root: &Path) -> CheckResult {
    // Run the specific test file that validates DOM
    let output = Command::new("cargo")
        .args(["test", "-p", "app-http", "--test", "ui_contract_dom", "--", "--nocapture"])
        .current_dir(root)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                CheckResult::Pass
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Extract test failure message
                let combined = format!("{}\n{}", stdout, stderr);
                let failure_msg = combined
                    .lines()
                    .find(|l| l.contains("FAILED") || l.contains("missing data-uiid"))
                    .unwrap_or("DOM validation tests failed")
                    .to_string();

                CheckResult::Fail(failure_msg)
            }
        }
        Err(e) => CheckResult::Fail(format!("Failed to run tests: {}", e)),
    }
}

/// Run UI contract check as a library call (for selftest integration)
pub fn run_check() -> Result<()> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test schema validation catches duplicate screen IDs
    #[test]
    fn test_schema_validation_duplicate_screen_ids() {
        let contract = spec_runtime::UiContract {
            schema_version: "1.0".to_string(),
            template_version: "v3.0.0".to_string(),
            screens: vec![
                spec_runtime::Screen {
                    id: "dashboard".to_string(),
                    route: "/".to_string(),
                    aliases: vec![],
                    description: "Main dashboard".to_string(),
                    regions: vec![],
                },
                spec_runtime::Screen {
                    id: "dashboard".to_string(), // Duplicate!
                    route: "/other".to_string(),
                    aliases: vec![],
                    description: "Other dashboard".to_string(),
                    regions: vec![],
                },
            ],
            region_kinds: HashMap::from([("panel".to_string(), "Panel region".to_string())]),
        };

        let result = check_schema_validation(&contract);
        assert!(
            matches!(result, CheckResult::Fail(msg) if msg.contains("Duplicate screen ID")),
            "Should fail on duplicate screen ID"
        );
    }

    /// Test schema validation catches duplicate region IDs
    #[test]
    fn test_schema_validation_duplicate_region_ids() {
        let contract = spec_runtime::UiContract {
            schema_version: "1.0".to_string(),
            template_version: "v3.0.0".to_string(),
            screens: vec![spec_runtime::Screen {
                id: "dashboard".to_string(),
                route: "/".to_string(),
                aliases: vec![],
                description: "Main dashboard".to_string(),
                regions: vec![
                    spec_runtime::Region {
                        id: "dashboard.health".to_string(),
                        kind: "panel".to_string(),
                        description: "Health panel".to_string(),
                    },
                    spec_runtime::Region {
                        id: "dashboard.health".to_string(), // Duplicate!
                        kind: "panel".to_string(),
                        description: "Another health panel".to_string(),
                    },
                ],
            }],
            region_kinds: HashMap::from([("panel".to_string(), "Panel region".to_string())]),
        };

        let result = check_schema_validation(&contract);
        assert!(
            matches!(result, CheckResult::Fail(msg) if msg.contains("Duplicate region ID")),
            "Should fail on duplicate region ID"
        );
    }

    /// Test region kind validation catches undefined kinds
    #[test]
    fn test_region_kind_validation() {
        let contract = spec_runtime::UiContract {
            schema_version: "1.0".to_string(),
            template_version: "v3.0.0".to_string(),
            screens: vec![spec_runtime::Screen {
                id: "dashboard".to_string(),
                route: "/".to_string(),
                aliases: vec![],
                description: "Main dashboard".to_string(),
                regions: vec![spec_runtime::Region {
                    id: "dashboard.health".to_string(),
                    kind: "undefined_kind".to_string(), // Not in region_kinds!
                    description: "Health panel".to_string(),
                }],
            }],
            region_kinds: HashMap::from([("panel".to_string(), "Panel region".to_string())]),
        };

        let result = check_region_kind_references(&contract);
        assert!(
            matches!(result, CheckResult::Fail(msg) if msg.contains("undefined kind")),
            "Should fail on undefined region kind"
        );
    }

    /// Test valid contract passes all checks
    #[test]
    fn test_valid_contract_passes() {
        let contract = spec_runtime::UiContract {
            schema_version: "1.0".to_string(),
            template_version: "v3.0.0".to_string(),
            screens: vec![spec_runtime::Screen {
                id: "dashboard".to_string(),
                route: "/".to_string(),
                aliases: vec!["/ui".to_string()],
                description: "Main dashboard".to_string(),
                regions: vec![
                    spec_runtime::Region {
                        id: "dashboard.health".to_string(),
                        kind: "panel".to_string(),
                        description: "Health panel".to_string(),
                    },
                    spec_runtime::Region {
                        id: "dashboard.nav".to_string(),
                        kind: "navigation".to_string(),
                        description: "Navigation bar".to_string(),
                    },
                ],
            }],
            region_kinds: HashMap::from([
                ("panel".to_string(), "Panel region".to_string()),
                ("navigation".to_string(), "Navigation region".to_string()),
            ]),
        };

        let schema_result = check_schema_validation(&contract);
        let kind_result = check_region_kind_references(&contract);

        assert_eq!(schema_result, CheckResult::Pass);
        assert_eq!(kind_result, CheckResult::Pass);
    }

    /// @AC-TPL-PLATFORM-UI-CONTRACT: Real contract file passes validation
    #[test]
    fn test_real_contract_passes() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();

        // Check YAML loads
        let yaml_result = check_yaml_parse(root);
        assert_eq!(yaml_result, CheckResult::Pass, "Real contract should parse");

        // Load and validate
        let contract =
            spec_runtime::load_ui_contract(&root.join("specs/ui_contract.yaml")).unwrap();

        let schema_result = check_schema_validation(&contract);
        assert_eq!(schema_result, CheckResult::Pass, "Real contract should pass schema validation");

        let kind_result = check_region_kind_references(&contract);
        assert_eq!(kind_result, CheckResult::Pass, "Real contract should have valid region kinds");
    }
}
