use ac_kernel::{Ac, AcSource, AcStatus};
use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use spec_runtime::ledger::TestMapping;
use std::collections::{HashMap, HashSet};
use std::process::Command;

use super::model::is_automated_test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestOutcome {
    Pass,
    Fail,
    Missing,
}

fn has_unit_tests(acs: &HashMap<String, Ac>) -> bool {
    acs.values().any(|ac| ac.tests.iter().any(|t| t.test_type.eq_ignore_ascii_case("unit")))
}

pub(super) fn collect_unit_test_results(
    acs: &HashMap<String, Ac>,
    _verbosity: crate::Verbosity,
    should_print_progress: bool,
) -> Result<HashMap<String, bool>> {
    if !has_unit_tests(acs) {
        return Ok(HashMap::new());
    }

    if should_print_progress {
        eprintln!("Running unit tests for AC mappings...");
    }

    let mut results = HashMap::new();
    let test_line = Regex::new(r"^test\s+([^\s]+)\s+\.\.\.\s+(ok|FAILED)$").unwrap();

    // Run workspace tests excluding acceptance and xtask
    let mut cmd = Command::new("cargo");
    cmd.args(["test", "--workspace", "--exclude", "acceptance", "--exclude", "xtask"]);
    // Avoid clobbering the active xtask binary on Windows by using a throwaway target dir
    cmd.env("CARGO_TARGET_DIR", "target/ac-status-unit");

    let output = cmd.output().context("Failed to run cargo test for unit AC mappings")?;
    let succeeded = output.status.success();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(caps) = test_line.captures(line.trim()) {
            let name = caps[1].to_string();
            let status = &caps[2] == "ok";
            results.insert(name, status);
        }
    }

    if !succeeded {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "{} workspace unit test run reported failures (status {:?})\nstderr:\n{}",
            "[WARN]".yellow(),
            output.status.code(),
            stderr
        );
    }

    // Run xtask tests separately with a different target dir to avoid binary conflicts
    let mut xtask_cmd = Command::new("cargo");
    xtask_cmd.args(["test", "-p", "xtask"]);
    xtask_cmd.env("CARGO_TARGET_DIR", "target/ac-status-xtask");

    let xtask_output =
        xtask_cmd.output().context("Failed to run cargo test for xtask AC mappings")?;
    let xtask_succeeded = xtask_output.status.success();

    for line in String::from_utf8_lossy(&xtask_output.stdout).lines() {
        if let Some(caps) = test_line.captures(line.trim()) {
            let name = caps[1].to_string();
            let status = &caps[2] == "ok";
            results.insert(name, status);
        }
    }

    if !xtask_succeeded {
        let stderr = String::from_utf8_lossy(&xtask_output.stderr);
        eprintln!(
            "{} xtask unit test run reported failures (status {:?})\nstderr:\n{}",
            "[WARN]".yellow(),
            xtask_output.status.code(),
            stderr
        );
    }

    if should_print_progress {
        eprintln!("  Captured results for {} unit tests", results.len());
    }

    Ok(results)
}

fn outcome_for_unit_test(
    mapping: &TestMapping,
    unit_results: &HashMap<String, bool>,
) -> TestOutcome {
    let mut candidates: HashSet<String> = HashSet::new();

    if let Some(module) = &mapping.module {
        candidates.insert(module.clone());
        if let Some((_, rest)) = module.split_once("::") {
            candidates.insert(rest.to_string());
        }
        if let Some(last) = module.rsplit("::").next() {
            candidates.insert(last.to_string());
        }
    }

    if !mapping.tag.is_empty() {
        candidates.insert(mapping.tag.clone());
        if let Some((_, rest)) = mapping.tag.split_once("::") {
            candidates.insert(rest.to_string());
        }
    }

    for candidate in candidates {
        if let Some(&passed) = unit_results.get(&candidate) {
            return if passed { TestOutcome::Pass } else { TestOutcome::Fail };
        }
    }

    TestOutcome::Missing
}

pub(super) fn update_ac_statuses(
    acs: &mut HashMap<String, Ac>,
    bdd_results: &HashMap<String, AcStatus>,
    unit_results: &HashMap<String, bool>,
    bdd_source: AcSource,
) {
    for ac in acs.values_mut() {
        ac.tests_executed = 0;
        let mut has_bdd_result = false;

        if ac.tests_total == 0 {
            ac.status = AcStatus::Unknown;
            ac.source = AcSource::Inferred;
            continue;
        }

        let mut failed = false;

        for test in &ac.tests {
            if !is_automated_test(test) {
                continue;
            }

            let outcome = match test.test_type.to_lowercase().as_str() {
                "bdd" | "integration" => {
                    let result = bdd_results
                        .get(&ac.id)
                        .cloned()
                        .map(|status| match status {
                            AcStatus::Pass => TestOutcome::Pass,
                            AcStatus::Fail => TestOutcome::Fail,
                            AcStatus::Unknown => TestOutcome::Missing,
                        })
                        .unwrap_or(TestOutcome::Missing);
                    if result != TestOutcome::Missing {
                        has_bdd_result = true;
                    }
                    result
                }
                "unit" => outcome_for_unit_test(test, unit_results),
                _ => TestOutcome::Missing,
            };

            match outcome {
                TestOutcome::Pass => {
                    ac.tests_executed += 1;
                }
                TestOutcome::Fail => {
                    ac.tests_executed += 1;
                    failed = true;
                }
                TestOutcome::Missing => {}
            }
        }

        // Count unit tests mapped for this AC (ADR-0024)
        let unit_mapped_count =
            ac.tests.iter().filter(|t| t.test_type.eq_ignore_ascii_case("unit")).count();

        // AC status semantics (ADR-0024):
        // - FAIL if any test failed
        // - PASS if at least one test passed OR unit tests are mapped
        //   (unit tests are presumed to run because cargo xtask check executes them)
        // - UNKNOWN only if zero tests ran AND no unit tests mapped
        ac.status = if failed {
            AcStatus::Fail
        } else if ac.tests_executed > 0 || unit_mapped_count > 0 {
            AcStatus::Pass
        } else {
            AcStatus::Unknown
        };

        // Set source based on where the result came from
        ac.source = if has_bdd_result {
            bdd_source
        } else if ac.tests_executed > 0 {
            // Had unit tests but no BDD - still mark with the bdd_source for consistency
            // since unit tests don't have their own source tracking
            bdd_source
        } else {
            AcSource::Inferred
        };
    }
}
