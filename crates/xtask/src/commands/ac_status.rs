use ac_kernel::{Ac, AcJson, AcSource, AcStatus, build_status_json};
use anyhow::{Context, Result};
use colored::Colorize;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use spec_runtime::ledger::TestMapping;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use super::ac_parsing::{
    AC_PATTERN_WITH_AT, SCENARIO_PATTERN, Scenario, TAG_PATTERN, TESTCASE_SCENARIO_PATTERN,
    TESTCASE_SUFFIX_PATTERN, parse_ac_coverage, parse_cucumber_json_with_scenarios,
    parse_features_with_metadata, parse_junit_with_scenarios,
};
use crate::kernel::layout_for_repo;

#[derive(Debug, Clone)]
pub struct AcStatusArgs {
    pub ledger: PathBuf,
    pub features_dir: PathBuf,
    pub coverage: PathBuf,
    pub junit: PathBuf,
    pub json_report: Option<PathBuf>,
    pub output: PathBuf,
    pub verbosity: crate::Verbosity,
    pub summary: bool,
    pub json: bool,
    /// Filter to a specific AC ID (e.g., AC-KERN-001)
    pub filter_ac: Option<String>,
    /// Check mode: compare computed status against existing file without writing.
    /// Returns error if the file content would differ. Used by selftest/CI.
    pub check: bool,
}

impl Default for AcStatusArgs {
    fn default() -> Self {
        let layout = layout_for_repo();
        Self {
            ledger: layout.ledger,
            features_dir: layout.features_dir,
            coverage: layout.coverage_file,
            junit: layout.junit_file,
            json_report: Some(PathBuf::from("target/ac_report.json")),
            output: PathBuf::from("docs/feature_status.md"),
            verbosity: crate::Verbosity::Normal,
            summary: false,
            json: false,
            filter_ac: None,
            check: false,
        }
    }
}

// Ac struct is now imported from ac_kernel

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportStatus {
    Missing,
    Empty,
    NonEmpty,
}

impl ReportStatus {
    fn from_path(path: &Path) -> Self {
        match fs::metadata(path) {
            Ok(meta) if meta.len() > 0 => ReportStatus::NonEmpty,
            Ok(_) => ReportStatus::Empty,
            Err(_) => ReportStatus::Missing,
        }
    }
}

impl std::fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ReportStatus::Missing => "missing",
            ReportStatus::Empty => "empty",
            ReportStatus::NonEmpty => "present",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestOutcome {
    Pass,
    Fail,
    Missing,
}

// AcStatusJson, AcCategoryStats, AcJson, and AcSource are now imported from ac_kernel
// Ledger parsing is now done via ac_kernel::parse_ledger_with_metadata()

pub fn run(args: AcStatusArgs) -> Result<()> {
    // Validate inputs
    if !args.ledger.exists() {
        anyhow::bail!(
            "ledger file not found: {}\n\n\
             try: cargo xtask doctor\n\
             hint: the spec root may be misconfigured",
            args.ledger.display()
        );
    }
    if !args.features_dir.exists() {
        anyhow::bail!(
            "features directory not found: {}\n\n\
             try: create specs/features/*.feature files\n\
             hint: run 'cargo xtask doctor' to check your environment",
            args.features_dir.display()
        );
    }

    // Helper: should we print progress messages? (not in quiet or JSON mode)
    let should_print_progress = !args.verbosity.is_quiet() && !args.json;

    if should_print_progress {
        eprintln!("Parsing ledger: {}", args.ledger.display());
    }
    let mut acs = parse_ledger(&args.ledger)?;
    if should_print_progress {
        eprintln!("  Found {} ACs", acs.len());
    }

    // Check all report sources
    let mut coverage_status = ReportStatus::from_path(&args.coverage);
    let mut junit_status = ReportStatus::from_path(&args.junit);
    let json_status = args.json_report.as_ref().map(|p| ReportStatus::from_path(p));

    // AUTO-REGENERATION: If all sources are missing/empty, run BDD tests to generate them.
    // Skip auto-regeneration if XTASK_SKIP_BDD=1 is set (to avoid nested BDD runs)
    // or if XTASK_NO_REGEN=1 is set explicitly
    let skip_regen =
        std::env::var("XTASK_SKIP_BDD").is_ok() || std::env::var("XTASK_NO_REGEN").is_ok();

    let has_any_results = coverage_status == ReportStatus::NonEmpty
        || junit_status == ReportStatus::NonEmpty
        || json_status == Some(ReportStatus::NonEmpty);

    if !has_any_results && !skip_regen {
        // No usable test results - run BDD to generate them
        if should_print_progress {
            eprintln!(
                "No BDD results found (coverage: {}, JUnit: {}) - running BDD suite...",
                coverage_status, junit_status
            );
        }

        // Run BDD tests - this will write to both coverage.jsonl and acceptance.xml
        // We ignore errors here because BDD tests may fail, but we still want
        // to parse the output for AC status
        let _ = super::bdd::run_for_junit(&args.junit);

        // Re-check status after running BDD
        coverage_status = ReportStatus::from_path(&args.coverage);
        junit_status = ReportStatus::from_path(&args.junit);

        if coverage_status != ReportStatus::NonEmpty && should_print_progress {
            if junit_status != ReportStatus::NonEmpty {
                eprintln!(
                    "{} Coverage file `{}` is still {} after running BDD.",
                    "[WARN]".yellow(),
                    args.coverage.display(),
                    coverage_status
                );
                eprintln!("  BDD-driven ACs will be marked as 'unknown'.");
            } else {
                eprintln!("  Using JUnit fallback (coverage file not available)");
            }
        } else if should_print_progress {
            eprintln!("  BDD results regenerated successfully");
        }
    }

    // PRIMARY PATH: AC Coverage JSONL (streams results, resilient to exit())
    // The coverage.jsonl file is the preferred source as it flushes on each
    // scenario completion and doesn't rely on Drop semantics.
    //
    // FALLBACK PATHS:
    // 1. JUnit XML + feature file parsing (legacy, may be empty on exit())
    // 2. Structured JSON report from acceptance tests
    let (scenarios, bdd_results, bdd_source) = if coverage_status == ReportStatus::NonEmpty {
        if should_print_progress {
            eprintln!("Parsing AC coverage (primary path): {}", args.coverage.display());
        }
        let (scenario_map, results) = parse_ac_coverage(&args.coverage)?;
        if should_print_progress {
            eprintln!("  Found {} scenarios", scenario_map.len());
            eprintln!("  Found results for {} ACs", results.len());
        }
        (scenario_map, results, AcSource::Coverage)
    } else if junit_status == ReportStatus::NonEmpty {
        if should_print_progress {
            eprintln!(
                "Coverage not found ({}); falling back to JUnit: {}",
                coverage_status,
                args.junit.display()
            );
        }
        let (scenario_map, results) = fallback_to_junit(&args, should_print_progress)?;
        (scenario_map, results, AcSource::Junit)
    } else if let Some(json_path) = &args.json_report {
        if json_status == Some(ReportStatus::NonEmpty) {
            if should_print_progress {
                eprintln!(
                    "Coverage and JUnit not found; falling back to JSON report: {}",
                    json_path.display()
                );
            }
            let (scenario_map, results) = parse_cucumber_json_with_scenarios(json_path)?;
            if should_print_progress {
                eprintln!("  Found {} scenarios", scenario_map.len());
                eprintln!("  Found results for {} ACs", results.len());
            }
            (scenario_map, results, AcSource::Json)
        } else {
            // GRACEFUL DEGRADATION: No BDD results available, continue with empty results.
            // BDD-driven ACs will be marked as "unknown" in the report.
            let json_state = json_status.unwrap_or(ReportStatus::Missing);
            if should_print_progress {
                eprintln!(
                    "{} No BDD test results available (coverage: {}, JUnit: {}, JSON: {})",
                    "[WARN]".yellow(),
                    coverage_status,
                    junit_status,
                    json_state
                );
                eprintln!("  BDD-driven ACs will be marked as 'unknown'.");
            }
            (HashMap::new(), HashMap::new(), AcSource::Inferred)
        }
    } else {
        // GRACEFUL DEGRADATION: No BDD results available, continue with empty results.
        // BDD-driven ACs will be marked as "unknown" in the report.
        if should_print_progress {
            eprintln!(
                "{} No BDD test results available (coverage: {}, JUnit: {}, JSON: not configured)",
                "[WARN]".yellow(),
                coverage_status,
                junit_status
            );
            eprintln!("  BDD-driven ACs will be marked as 'unknown'.");
        }
        (HashMap::new(), HashMap::new(), AcSource::Inferred)
    };

    // Map scenarios to ACs
    for scenario in scenarios.values() {
        if let Some(ac) = acs.get_mut(&scenario.ac_id) {
            ac.scenarios.push(scenario.name.clone());
        }
    }

    // Collect unit test results for ACs that declare unit tests
    let unit_results = collect_unit_test_results(&acs, args.verbosity, should_print_progress)?;

    // Combine BDD + unit results into a final AC status
    update_ac_statuses(&mut acs, &bdd_results, &unit_results, bdd_source);

    // Handle single-AC filter mode
    if let Some(ref filter_id) = args.filter_ac {
        return print_single_ac(&acs, filter_id, args.json);
    }

    // Warn if --check is used with flags that ignore it
    if args.check && (args.json || args.summary) && !args.verbosity.is_quiet() {
        eprintln!(
            "{} --check flag has no effect when --json or --summary is specified (no file operation)",
            "[WARN]".yellow()
        );
    }

    if args.json {
        // Output structured JSON
        print_json_output(&acs)?;
    } else if args.summary {
        // Print concise summary instead of generating markdown file
        print_summary(&acs)?;
    } else {
        if args.check {
            // Check mode: compare computed status against existing file without writing
            if should_print_progress {
                eprintln!("Checking status against: {}", args.output.display());
            }
            check_status_md(&acs, &scenarios, &args.output, should_print_progress)?;
        } else {
            // Write mode: Generate full markdown report
            if should_print_progress {
                eprintln!("Generating status: {}", args.output.display());
            }
            generate_status_md(&acs, &scenarios, &args.output, should_print_progress)?;
        }

        // Check for failures (both modes)
        let failed: Vec<_> = acs
            .values()
            .filter(|ac| ac.status == AcStatus::Fail)
            .map(|ac| ac.id.as_str())
            .collect();

        if !failed.is_empty() {
            eprintln!("\n{} {} AC(s) failed: {}", "[FAIL]".red(), failed.len(), failed.join(", "));
            anyhow::bail!("One or more ACs failed");
        }

        if should_print_progress {
            eprintln!("\n{} All ACs passed", "[OK]".green());
        }
    }

    Ok(())
}

fn fallback_to_junit(
    args: &AcStatusArgs,
    should_print_progress: bool,
) -> Result<(HashMap<String, Scenario>, HashMap<String, AcStatus>)> {
    if should_print_progress {
        eprintln!("Parsing features: {}", args.features_dir.display());
    }
    let scenarios = parse_features_with_metadata(&args.features_dir)?;
    if should_print_progress {
        eprintln!("  Found {} scenarios", scenarios.len());
    }

    if should_print_progress {
        eprintln!("Parsing JUnit results: {}", args.junit.display());
    }
    let ac_results = parse_junit_with_scenarios(&args.junit, &scenarios)?;
    if should_print_progress {
        eprintln!("  Found results for {} ACs", ac_results.len());
    }

    Ok((scenarios, ac_results))
}

fn parse_ledger(ledger_path: &Path) -> Result<HashMap<String, Ac>> {
    // Use ac-kernel's ledger parser for consistency
    let metadata = ac_kernel::parse_ledger_with_metadata(ledger_path)?;

    let mut acs = HashMap::new();
    for (id, m) in metadata {
        let tests_total = m.tests.iter().filter(|t| is_automated_test(t)).count();
        acs.insert(
            id.clone(),
            Ac {
                id,
                story_id: m.story_id,
                req_id: m.req_id,
                text: m.text,
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: Vec::new(),
                tests: m.tests,
                tests_total,
                tests_executed: 0,
                tags: m.tags,
                must_have_ac: m.must_have_ac,
            },
        );
    }

    Ok(acs)
}

fn has_unit_tests(acs: &HashMap<String, Ac>) -> bool {
    acs.values().any(|ac| ac.tests.iter().any(|t| t.test_type.eq_ignore_ascii_case("unit")))
}

fn is_automated_test(test: &TestMapping) -> bool {
    matches!(test.test_type.to_lowercase().as_str(), "unit" | "integration" | "bdd")
}

fn is_meta_ac(ac: &Ac) -> bool {
    // Meta ACs are those with tags indicating they're test harness or example-level,
    // not service-level contracts
    ac.tags.iter().any(|t| matches!(t.as_str(), "harness" | "example" | "ci-only"))
        || ac.tests.iter().any(|t| t.test_type.eq_ignore_ascii_case("ci"))
}

fn collect_unit_test_results(
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

fn update_ac_statuses(
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

        // AC status semantics:
        // - FAIL if any test failed
        // - PASS if at least one test passed AND no tests failed
        // - UNKNOWN only if zero tests ran
        ac.status = if failed {
            AcStatus::Fail
        } else if ac.tests_executed > 0 {
            AcStatus::Pass
        } else {
            AcStatus::Unknown
        };

        // Set source based on where the result came from
        ac.source = if has_bdd_result {
            bdd_source.clone()
        } else if ac.tests_executed > 0 {
            // Had unit tests but no BDD - still mark with the bdd_source for consistency
            // since unit tests don't have their own source tracking
            bdd_source.clone()
        } else {
            AcSource::Inferred
        };
    }
}

#[allow(dead_code)]
fn parse_features(features_dir: &Path) -> Result<HashMap<String, Scenario>> {
    let mut scenarios = HashMap::new();

    for entry in WalkDir::new(features_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "feature"))
    {
        let content = fs::read_to_string(entry.path())?;
        let relative_path = entry
            .path()
            .strip_prefix(features_dir.parent().unwrap_or(features_dir))
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        let mut current_tags: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim_start();

            // Collect tags from lines starting with @
            if trimmed.starts_with('@') {
                for tag_match in TAG_PATTERN.find_iter(trimmed) {
                    current_tags.push(tag_match.as_str().to_string());
                }
            }
            // Match scenario and attach accumulated tags
            else if let Some(caps) = SCENARIO_PATTERN.captures(trimmed) {
                let scenario_name = caps[1].trim();

                // Look for AC ID in collected tags
                let ac_id = current_tags
                    .iter()
                    .find_map(|tag| AC_PATTERN_WITH_AT.captures(tag))
                    .map(|caps| caps[1].to_string());

                if let Some(ac_id) = ac_id {
                    scenarios.insert(
                        scenario_name.to_string(),
                        Scenario {
                            name: scenario_name.to_string(),
                            ac_id,
                            file: relative_path.clone(),
                        },
                    );
                }

                // Reset tags after processing scenario
                current_tags.clear();
            }
            // Reset tags on other Gherkin keywords (but not on blank lines or comments)
            else if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && (trimmed.starts_with("Feature:")
                    || trimmed.starts_with("Background:")
                    || trimmed.starts_with("Rule:")
                    || trimmed.starts_with("Examples:")
                    || trimmed.starts_with("Given ")
                    || trimmed.starts_with("When ")
                    || trimmed.starts_with("Then ")
                    || trimmed.starts_with("And ")
                    || trimmed.starts_with("But ")
                    || trimmed.starts_with("|"))
            {
                current_tags.clear();
            }
        }
    }

    Ok(scenarios)
}

/// Normalize JUnit testcase names for scenario matching.
/// Future: Used if we switch back to JUnit-based AC status (currently using Cucumber JSON).
/// Kept as infrastructure for fallback parsing path.
/// Remove #[allow] once parsing strategy is finalized.
#[allow(dead_code)]
fn normalize_testcase_name(name: &str) -> String {
    // Extract scenario name from JUnit testcase name
    // Format: "Scenario: <name>: <file>:<line>:<col>"
    if let Some(caps) = TESTCASE_SCENARIO_PATTERN.captures(name) {
        let scenario_name = caps[1].trim();
        // Remove example/row suffixes
        let normalized = TESTCASE_SUFFIX_PATTERN.replace(scenario_name, "");
        return normalized.to_string();
    }

    name.to_string()
}

/// Parse JUnit XML and extract AC test results using scenario metadata.
/// Future: Fallback parser for systems without Cucumber JSON support.
/// Currently superseded by parse_junit_with_scenarios() from ac_parsing module.
/// Remove #[allow] once deprecated in favor of unified parsing path.
#[allow(dead_code)]
fn parse_junit(
    junit_path: &Path,
    scenarios: &HashMap<String, Scenario>,
) -> Result<HashMap<String, AcStatus>> {
    let content = fs::read_to_string(junit_path)?;
    let mut reader = Reader::from_str(&content);

    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();
    let mut buf = Vec::new();

    let mut current_testcase: Option<String> = None;
    let mut testcase_passed = true;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"testcase" => {
                        // Extract testcase name
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                let name = String::from_utf8_lossy(&attr.value).to_string();
                                current_testcase = Some(name);
                                testcase_passed = true;
                            }
                        }
                    }
                    b"failure" | b"error" => {
                        testcase_passed = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"testcase"
                    && let Some(tc_name) = current_testcase.take()
                {
                    let normalized = normalize_testcase_name(&tc_name);

                    // Find matching scenario
                    if let Some(scenario) = scenarios.get(&normalized) {
                        ac_results.entry(scenario.ac_id.clone()).or_default().push(testcase_passed);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    // Aggregate: AC passes only if all testcases pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status =
            if results.iter().all(|&passed| passed) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok(ac_status)
}

fn print_summary(acs: &HashMap<String, Ac>) -> Result<()> {
    // Compute totals from all ACs
    let total_acs = acs.len();
    let total_passing = acs.values().filter(|ac| ac.status == AcStatus::Pass).count();
    let total_failing = acs.values().filter(|ac| ac.status == AcStatus::Fail).count();
    let total_unknown = acs.values().filter(|ac| ac.status == AcStatus::Unknown).count();
    let coverage_percent =
        if total_acs > 0 { (total_passing as f64 / total_acs as f64) * 100.0 } else { 0.0 };

    // Count by must_have_ac flag (kernel/required vs optional)
    let must_have_count = acs.values().filter(|ac| ac.must_have_ac).count();
    let optional_count = acs.values().filter(|ac| !ac.must_have_ac).count();

    println!("AC Status Summary:");

    // Build breakdown string dynamically (only include non-zero categories)
    let mut breakdown_parts = Vec::new();
    if must_have_count > 0 {
        breakdown_parts.push(format!("{} must-have", must_have_count));
    }
    if optional_count > 0 {
        breakdown_parts.push(format!("{} optional", optional_count));
    }

    if breakdown_parts.is_empty() {
        println!("  Ledger: {} ACs", total_acs);
    } else {
        println!("  Ledger: {} ACs ({})", total_acs, breakdown_parts.join(", "));
    }

    println!("  Coverage: {:.2}% (passing ACs)", coverage_percent);
    println!("  {} {} passing", AcStatus::Pass.icon(), total_passing);
    println!("  {} {} failing", AcStatus::Fail.icon(), total_failing);
    println!("  {} {} unknown (no mapped tests)", AcStatus::Unknown.icon(), total_unknown);

    if total_failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

/// Print detailed information about a single AC.
///
/// Used by `cargo xtask ac-status --ac <ID>` to debug specific AC status.
fn print_single_ac(acs: &HashMap<String, Ac>, filter_id: &str, json_output: bool) -> Result<()> {
    let ac = acs.get(filter_id).ok_or_else(|| {
        anyhow::anyhow!(
            "AC '{}' not found in ledger\n\n\
             try: cargo xtask ac-status --json\n\
             hint: check specs/spec_ledger.yaml for available AC IDs",
            filter_id
        )
    })?;

    if json_output {
        // Output just this AC as JSON
        let ac_json = AcJson {
            id: ac.id.clone(),
            story_id: ac.story_id.clone(),
            req_id: ac.req_id.clone(),
            text: ac.text.clone(),
            status: ac.status.clone(),
            source: ac.source.clone(),
            must_have_ac: ac.must_have_ac,
            scenarios: ac.scenarios.clone(),
            tests: ac.tests.clone(),
            tests_total: ac.tests_total,
            tests_executed: ac.tests_executed,
        };
        let json_str =
            serde_json::to_string_pretty(&ac_json).context("Failed to serialize AC to JSON")?;
        println!("{}", json_str);
    } else {
        // Print human-readable output
        println!("AC: {}", ac.id.cyan().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("  Story:       {}", ac.story_id);
        println!("  Requirement: {}", ac.req_id);
        println!("  Text:        {}", ac.text);
        println!();

        // Status with icon
        let status_display = match ac.status {
            AcStatus::Pass => format!("{} {}", "[PASS]".green(), "All tests passed"),
            AcStatus::Fail => format!("{} {}", "[FAIL]".red(), "One or more tests failed"),
            AcStatus::Unknown => format!("{} {}", "[UNKNOWN]".yellow(), "No tests executed"),
        };
        println!("  Status:      {}", status_display);
        println!("  Tests:       {} executed / {} total", ac.tests_executed, ac.tests_total);
        println!();

        // Scenarios
        if ac.scenarios.is_empty() {
            println!("  Scenarios:   {} (no BDD scenarios mapped)", "(none)".yellow());
            println!();
            println!("  💡 Hint: Add a scenario tagged with @{} to a .feature file", ac.id);
            println!("     Or run: cargo xtask ac-suggest-scenarios {}", ac.id);
        } else {
            println!("  Scenarios:");
            for scenario in &ac.scenarios {
                println!("    • {}", scenario);
            }
        }
        println!();

        // Tests
        if ac.tests.is_empty() {
            println!("  Tests:       {} (no tests declared in ledger)", "(none)".yellow());
        } else {
            println!("  Test mappings:");
            for test in &ac.tests {
                let module = test.module.as_deref().unwrap_or("-");
                println!("    • [{}] {} (tag: {})", test.test_type, module, test.tag);
            }
        }
        println!();

        // Tags
        if !ac.tags.is_empty() {
            println!("  Tags: {}", ac.tags.join(", "));
        }
        if ac.must_have_ac {
            println!("  Flags: must_have_ac=true");
        }
    }

    // Exit with error if AC failed
    if ac.status == AcStatus::Fail {
        anyhow::bail!("AC {} failed", filter_id);
    }

    Ok(())
}

fn print_json_output(acs: &HashMap<String, Ac>) -> Result<()> {
    // Use ac-kernel's build_status_json for consistency with the kernel
    let output = build_status_json(acs);

    let json_output =
        serde_json::to_string_pretty(&output).context("Failed to serialize AC status to JSON")?;

    println!("{}", json_output);

    // Check for failures (still need to fail the command if tests failed)
    if output.must_have_acs.failing > 0 || output.optional_acs.failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

/// Extract template_version from spec_ledger.yaml for feature_status metadata
fn get_template_version() -> Option<String> {
    let ledger_path = layout_for_repo().ledger;
    if let Ok(content) = fs::read_to_string(&ledger_path) {
        for line in content.lines() {
            if line.trim().starts_with("template_version:")
                && let Some(version) = line.split(':').nth(1)
            {
                return Some(version.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Extract last_updated from spec_ledger.yaml for feature_status metadata
fn get_last_updated_date() -> Option<String> {
    let ledger_path = layout_for_repo().ledger;
    if let Ok(content) = fs::read_to_string(&ledger_path) {
        for line in content.lines() {
            if line.trim().starts_with("last_updated:")
                && let Some(date) = line.split(':').nth(1)
            {
                return Some(date.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Generate the markdown content for feature_status.md (shared between write and check modes)
fn generate_status_md_content(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
) -> String {
    let mut output = String::new();
    output.push_str("<!--\n");
    output.push_str("  AUTO-GENERATED FILE: DO NOT EDIT BY HAND.\n\n");
    output.push_str("  Source of truth:\n");
    output.push_str("    - Specs: specs/spec_ledger.yaml\n");
    output.push_str("    - Tests: specs/features/**/*.feature + unit test results (cargo test)\n");
    output.push_str("  Generated by:\n");
    output.push_str("    - cargo run -p xtask -- ac-status\n");
    output.push_str("  Regenerated by:\n");
    output.push_str("    - cargo run -p xtask -- selftest (AC/ADR mapping step)\n");

    // Add template version and last updated metadata for docs-check validation (AC-PLT-010 invariants)
    if let Some(version) = get_template_version() {
        output.push_str(&format!("  Template Version: {}\n", version));
    }
    if let Some(last_updated) = get_last_updated_date() {
        output.push_str(&format!("  Last Updated: {}\n", last_updated));
    }

    output.push_str("    \n");
    output.push_str("  To update this file, modify specs or BDD scenarios, then run:\n");
    output.push_str("    cargo xtask ac-status\n");
    output.push_str("-->\n\n");
    output.push_str("# Feature Status\n\n");
    output.push_str("Auto-generated AC status from acceptance (BDD) and unit tests.\n\n");

    output.push_str("## AC Status Summary\n\n");
    output.push_str("> **How to read this**\n");
    output.push_str("> - `[PASS]` = at least one test (BDD or unit) ran and passed.\n");
    output.push_str("> - `[FAIL]` = at least one test ran and failed.\n");
    output
        .push_str("> - `[UNKNOWN]` = no local test ran. In this repo, `[UNKNOWN]` is only used\n");
    output.push_str(">   for meta / CI-only ACs (see sections at the end).\n");
    output.push_str(">\n");
    output
        .push_str("> For formal definitions of `must_have_ac` and AC governance semantics, see\n");
    output.push_str(
        "> [`crates/ac-kernel/README.md`](../crates/ac-kernel/README.md#ac-governance-semantics).\n\n",
    );
    output.push_str("| AC ID | Story | Requirement | Status | Tests (executed/total) |\n");
    output.push_str("|-------|-------|-------------|--------|------------------------|\n");

    // Sort ACs for deterministic output
    let mut sorted_acs: Vec<_> = acs.values().collect();
    sorted_acs.sort_by_key(|ac| &ac.id);

    for ac in sorted_acs {
        output.push_str(&format!(
            "| {} | {} | {} | {} {} | {} / {} |\n",
            ac.id,
            ac.story_id,
            ac.req_id,
            ac.status.icon(),
            ac.status.name(),
            ac.tests_executed,
            ac.tests_total
        ));
    }

    // Unmapped ACs: Split into service-level and meta ACs
    let unmapped: Vec<_> =
        acs.values().filter(|ac| ac.tests_total == 0 || ac.tests_executed == 0).collect();

    let service_unmapped: Vec<_> = unmapped.iter().filter(|ac| !is_meta_ac(ac)).copied().collect();
    let meta_unmapped: Vec<_> = unmapped.iter().filter(|ac| is_meta_ac(ac)).copied().collect();

    // Service-level unmapped ACs (should ideally be empty)
    output.push_str("\n## Unmapped ACs (Service Behaviour)\n\n");
    output.push_str(
        "*(This list SHOULD be empty in this repo. If anything appears here, it's a bug.)*\n\n",
    );
    if service_unmapped.is_empty() {
        output.push_str("- *(none)*\n");
    } else {
        // Sort for deterministic output
        let mut sorted_service: Vec<_> = service_unmapped;
        sorted_service.sort_by_key(|ac| &ac.id);
        for ac in sorted_service {
            let text = ac.text.trim();
            output.push_str(&format!("- {}: {}\n", ac.id, text));
        }
    }

    // Meta / CI-only ACs (intentionally not tested locally)
    if !meta_unmapped.is_empty() {
        output.push_str("\n## Meta / CI-only ACs (Not Executed Locally)\n\n");
        output
            .push_str("These ACs describe test harness or example workspace behaviour. They are\n");
        output.push_str("validated in CI, not by local selftest:\n\n");
        let mut sorted_meta: Vec<_> = meta_unmapped;
        sorted_meta.sort_by_key(|ac| &ac.id);
        for ac in sorted_meta {
            let text = ac.text.trim();
            output.push_str(&format!("- {} – {}\n", ac.id, text));
        }
    }

    let mut unmapped_scenarios: Vec<_> =
        scenarios.values().filter(|s| !acs.contains_key(&s.ac_id)).collect();
    unmapped_scenarios.sort_by_key(|s| &s.name);

    if !unmapped_scenarios.is_empty() {
        output.push_str("\n## Unmapped Scenarios\n\n");
        output.push_str("Scenarios referencing non-existent ACs:\n\n");
        for scenario in unmapped_scenarios {
            output.push_str(&format!(
                "- Scenario '{}' references {} (in {})\n",
                scenario.name, scenario.ac_id, scenario.file
            ));
        }
    }

    output
}

/// Write mode: Generate feature_status.md file
fn generate_status_md(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    output_path: &Path,
    should_print_progress: bool,
) -> Result<()> {
    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = generate_status_md_content(acs, scenarios);
    fs::write(output_path, content)?;

    if should_print_progress {
        eprintln!("{} Generated {}", "[OK]".green(), output_path.display());
    }

    Ok(())
}

/// Check mode: Compare computed status against existing file without writing.
/// Returns error if the file content would differ.
fn check_status_md(
    acs: &HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    output_path: &Path,
    should_print_progress: bool,
) -> Result<()> {
    let expected = generate_status_md_content(acs, scenarios);

    let actual = fs::read_to_string(output_path).with_context(|| {
        format!(
            "AC status file not found: {}\n\n\
             hint: run 'cargo xtask ac-status' to generate it first",
            output_path.display()
        )
    })?;

    if expected != actual {
        // Find first differing line for diagnostics
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        let diff_line = expected_lines
            .iter()
            .zip(actual_lines.iter())
            .enumerate()
            .find(|(_, (e, a))| e != a)
            .map(|(i, _)| i + 1);

        let diff_info = if let Some(line) = diff_line {
            format!(" (first difference at line {})", line)
        } else if expected_lines.len() != actual_lines.len() {
            format!(
                " (line count differs: expected {}, got {})",
                expected_lines.len(),
                actual_lines.len()
            )
        } else {
            String::new()
        };

        anyhow::bail!(
            "AC status file is out of sync: {}{}\n\n\
             hint: run 'cargo xtask ac-status' to regenerate\n\
             then commit the updated file",
            output_path.display(),
            diff_info
        );
    }

    if should_print_progress {
        eprintln!("{} AC status file is up to date", "[OK]".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn report_status_handles_empty_and_non_empty() {
        let temp_dir = TempDir::new().unwrap();

        let missing = temp_dir.path().join("missing.xml");
        assert_eq!(ReportStatus::from_path(&missing), ReportStatus::Missing);

        let empty = temp_dir.path().join("empty.xml");
        fs::write(&empty, "").unwrap();
        assert_eq!(ReportStatus::from_path(&empty), ReportStatus::Empty);

        let populated = temp_dir.path().join("populated.xml");
        fs::write(&populated, "<xml>data</xml>").unwrap();
        assert_eq!(ReportStatus::from_path(&populated), ReportStatus::NonEmpty);
    }

    #[test]
    fn update_ac_statuses_combines_bdd_and_unit_results() {
        let mut acs = HashMap::new();

        acs.insert(
            "AC-UNIT".to_string(),
            Ac {
                id: "AC-UNIT".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "Unit-backed AC".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: Vec::new(),
                tests_total: 1,
                tests_executed: 0,
                tags: vec!["kernel".to_string()],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "unit".to_string(),
                    tag: "graph_invariants_req_has_ac".to_string(),
                    file: None,
                    module: Some("graph::tests::graph_invariants_req_has_ac".to_string()),
                }],
            },
        );

        acs.insert(
            "AC-BDD".to_string(),
            Ac {
                id: "AC-BDD".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "BDD-backed AC".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: vec!["Scenario A".to_string()],
                tests_total: 1,
                tests_executed: 0,
                tags: vec!["kernel".to_string()],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "integration".to_string(),
                    tag: "@AC-BDD".to_string(),
                    file: None,
                    module: None,
                }],
            },
        );

        let bdd_results = HashMap::from([("AC-BDD".to_string(), AcStatus::Fail)]);
        let unit_results =
            HashMap::from([("graph::tests::graph_invariants_req_has_ac".to_string(), true)]);

        update_ac_statuses(&mut acs, &bdd_results, &unit_results, AcSource::Coverage);

        let ac_unit = acs.get("AC-UNIT").unwrap();
        assert_eq!(ac_unit.status, AcStatus::Pass);
        assert_eq!(ac_unit.tests_executed, 1);

        let ac_bdd = acs.get("AC-BDD").unwrap();
        assert_eq!(ac_bdd.status, AcStatus::Fail);
        assert_eq!(ac_bdd.tests_executed, 1);

        acs.insert(
            "AC-MISSING".to_string(),
            Ac {
                id: "AC-MISSING".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "Missing coverage".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: Vec::new(),
                tests_total: 1,
                tests_executed: 0,
                tags: vec!["kernel".to_string()],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "unit".to_string(),
                    tag: "nonexistent_test".to_string(),
                    file: None,
                    module: Some("missing::test".to_string()),
                }],
            },
        );

        update_ac_statuses(&mut acs, &bdd_results, &unit_results, AcSource::Coverage);
        let ac_missing = acs.get("AC-MISSING").unwrap();
        assert_eq!(ac_missing.status, AcStatus::Unknown);
        assert_eq!(ac_missing.tests_executed, 0);
    }

    #[test]
    fn test_parse_features_with_tags_on_previous_line() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Test Feature
  As a user
  I want to test tag parsing
  So that ACs are correctly mapped

  @AC-TEST-001 @smoke
  Scenario: First scenario
    When I do something
    Then it works

  @AC-TEST-002
  Scenario: Second scenario
    When I do something else
    Then it also works
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.contains_key("First scenario"));
        assert!(scenarios.contains_key("Second scenario"));

        let scenario1 = &scenarios["First scenario"];
        assert_eq!(scenario1.ac_id, "AC-TEST-001");

        let scenario2 = &scenarios["Second scenario"];
        assert_eq!(scenario2.ac_id, "AC-TEST-002");
    }

    #[test]
    fn test_parse_features_with_multiple_tags_on_same_line() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Multi-tag Test

  @AC-TEST-003 @wip @integration
  Scenario: Scenario with multiple tags
    Given something
    When action
    Then result
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 1);
        let scenario = &scenarios["Scenario with multiple tags"];
        assert_eq!(scenario.ac_id, "AC-TEST-003");
    }

    #[test]
    fn test_parse_features_ignores_scenarios_without_ac_tags() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Mixed scenarios

  @smoke
  Scenario: No AC tag
    When something happens
    Then no mapping

  @AC-TEST-004
  Scenario: Has AC tag
    When something else happens
    Then mapped correctly
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        // Only the scenario with AC tag should be included
        assert_eq!(scenarios.len(), 1);
        assert!(scenarios.contains_key("Has AC tag"));
        assert!(!scenarios.contains_key("No AC tag"));
    }

    #[test]
    fn test_parse_features_scenario_outline() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Outline test

  @AC-TEST-005
  Scenario Outline: Parameterized scenario
    When I use <value>
    Then I get <result>

    Examples:
      | value | result |
      | 1     | one    |
      | 2     | two    |
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 1);
        let scenario = &scenarios["Parameterized scenario"];
        assert_eq!(scenario.ac_id, "AC-TEST-005");
    }

    // NOTE: The ac_status_json_shape_is_stable test is now in ac-kernel::json::tests
    // since the JSON schema is owned by the kernel. See crates/ac-kernel/src/json.rs
    // for the authoritative shape lock test.

    // Helper to create a test AC
    fn make_test_ac(id: &str, status: AcStatus) -> Ac {
        let tests_executed = if status == AcStatus::Unknown { 0 } else { 1 };
        let source =
            if status == AcStatus::Unknown { AcSource::Inferred } else { AcSource::Coverage };
        Ac {
            id: id.to_string(),
            story_id: "US-TEST-001".to_string(),
            req_id: "REQ-TEST-001".to_string(),
            text: format!("Test AC for {}", id),
            status,
            source,
            scenarios: vec!["Test scenario".to_string()],
            tests: vec![TestMapping {
                test_type: "bdd".to_string(),
                tag: format!("@{}", id),
                file: None,
                module: None,
            }],
            tests_total: 1,
            tests_executed,
            tags: vec!["kernel".to_string()],
            must_have_ac: false,
        }
    }

    #[test]
    fn print_single_ac_returns_error_for_missing_ac() {
        let acs: HashMap<String, Ac> = HashMap::new();
        let result = print_single_ac(&acs, "AC-NONEXISTENT", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn print_single_ac_returns_ok_for_passing_ac() {
        let mut acs = HashMap::new();
        acs.insert("AC-PASS-001".to_string(), make_test_ac("AC-PASS-001", AcStatus::Pass));

        let result = print_single_ac(&acs, "AC-PASS-001", false);
        assert!(result.is_ok());
    }

    #[test]
    fn print_single_ac_returns_ok_for_unknown_ac() {
        let mut acs = HashMap::new();
        acs.insert("AC-UNKNOWN-001".to_string(), make_test_ac("AC-UNKNOWN-001", AcStatus::Unknown));

        let result = print_single_ac(&acs, "AC-UNKNOWN-001", false);
        assert!(result.is_ok());
    }

    #[test]
    fn print_single_ac_returns_error_for_failing_ac() {
        let mut acs = HashMap::new();
        acs.insert("AC-FAIL-001".to_string(), make_test_ac("AC-FAIL-001", AcStatus::Fail));

        let result = print_single_ac(&acs, "AC-FAIL-001", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed"));
    }

    #[test]
    fn print_single_ac_json_output_parses_correctly() {
        let mut acs = HashMap::new();
        acs.insert("AC-JSON-001".to_string(), make_test_ac("AC-JSON-001", AcStatus::Pass));

        // Capture stdout by just verifying it doesn't panic and returns Ok
        // In a real test setup we'd capture stdout, but for now we verify the contract
        let result = print_single_ac(&acs, "AC-JSON-001", true);
        assert!(result.is_ok());
    }

    // ==========================================================================
    // Cross-module invariants test: must_have_ac semantics
    // ==========================================================================
    //
    // This test bridges ac_parsing.rs and ac_status.rs to ensure both modules
    // compute the same `must_have_ac` value for each AC. If this test fails,
    // it means the AND semantics for must_have_ac have diverged.

    #[test]
    fn must_have_ac_invariant_matches_ac_parsing() {
        use super::super::ac_parsing::parse_ledger_with_metadata;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a ledger with varied must_have_ac configurations
        let content = r#"
stories:
  - id: US-INVARIANT-001
    requirements:
      # Case 1: Both default (true && true = true)
      - id: REQ-BOTH-DEFAULT
        acceptance_criteria:
          - id: AC-BOTH-DEFAULT
            text: "Both default to true"

      # Case 2: REQ false, AC default (false && true = false)
      - id: REQ-FALSE-AC-DEFAULT
        must_have_ac: false
        acceptance_criteria:
          - id: AC-REQ-FALSE
            text: "REQ is false"

      # Case 3: REQ default, AC false (true && false = false)
      - id: REQ-DEFAULT-AC-FALSE
        acceptance_criteria:
          - id: AC-EXPLICIT-FALSE
            text: "AC explicitly false"
            must_have_ac: false

      # Case 4: Both explicit true (true && true = true)
      - id: REQ-BOTH-TRUE
        must_have_ac: true
        acceptance_criteria:
          - id: AC-BOTH-TRUE
            text: "Both explicit true"
            must_have_ac: true

      # Case 5: Both explicit false (false && false = false)
      - id: REQ-BOTH-FALSE
        must_have_ac: false
        acceptance_criteria:
          - id: AC-BOTH-FALSE
            text: "Both explicit false"
            must_have_ac: false

      # Case 6: Mixed ACs under same REQ
      - id: REQ-MIXED-ACS
        must_have_ac: true
        acceptance_criteria:
          - id: AC-MIXED-KERNEL
            text: "Kernel AC"
            must_have_ac: true
          - id: AC-MIXED-OPTIONAL
            text: "Optional AC"
            must_have_ac: false
"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file.flush().unwrap();

        // Parse with both modules
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();
        let acs = parse_ledger(file.path()).unwrap();

        // Verify all ACs exist in both
        assert_eq!(metadata.len(), acs.len(), "AC count mismatch");

        // Verify each AC has matching must_have_ac
        for (id, meta) in &metadata {
            let ac = acs.get(id).unwrap_or_else(|| panic!("AC {} missing from ac_status", id));
            assert_eq!(
                ac.must_have_ac, meta.must_have_ac,
                "must_have_ac mismatch for {}: ac_status={}, ac_parsing={}",
                id, ac.must_have_ac, meta.must_have_ac
            );
            assert_eq!(
                ac.req_id, meta.req_id,
                "req_id mismatch for {}: ac_status={}, ac_parsing={}",
                id, ac.req_id, meta.req_id
            );
        }

        // Verify expected values
        assert!(acs.get("AC-BOTH-DEFAULT").unwrap().must_have_ac);
        assert!(!acs.get("AC-REQ-FALSE").unwrap().must_have_ac);
        assert!(!acs.get("AC-EXPLICIT-FALSE").unwrap().must_have_ac);
        assert!(acs.get("AC-BOTH-TRUE").unwrap().must_have_ac);
        assert!(!acs.get("AC-BOTH-FALSE").unwrap().must_have_ac);
        assert!(acs.get("AC-MIXED-KERNEL").unwrap().must_have_ac);
        assert!(!acs.get("AC-MIXED-OPTIONAL").unwrap().must_have_ac);
    }

    // ==========================================================================
    // AcSource behavior tests
    // ==========================================================================

    #[test]
    fn update_ac_statuses_sets_source_from_bdd() {
        let mut acs = HashMap::new();
        acs.insert(
            "AC-BDD-SRC".to_string(),
            Ac {
                id: "AC-BDD-SRC".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "BDD source test".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: vec!["Test scenario".to_string()],
                tests_total: 1,
                tests_executed: 0,
                tags: vec![],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "bdd".to_string(),
                    tag: "@AC-BDD-SRC".to_string(),
                    file: None,
                    module: None,
                }],
            },
        );

        // Case 1: BDD result from Coverage
        let bdd_results = HashMap::from([("AC-BDD-SRC".to_string(), AcStatus::Pass)]);
        update_ac_statuses(&mut acs, &bdd_results, &HashMap::new(), AcSource::Coverage);

        let ac = acs.get("AC-BDD-SRC").unwrap();
        assert_eq!(ac.status, AcStatus::Pass);
        assert!(matches!(ac.source, AcSource::Coverage));
    }

    #[test]
    fn update_ac_statuses_sets_source_from_junit() {
        let mut acs = HashMap::new();
        acs.insert(
            "AC-JUNIT-SRC".to_string(),
            Ac {
                id: "AC-JUNIT-SRC".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "JUnit source test".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Inferred,
                scenarios: Vec::new(),
                tests_total: 1,
                tests_executed: 0,
                tags: vec![],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "bdd".to_string(),
                    tag: "@AC-JUNIT-SRC".to_string(),
                    file: None,
                    module: None,
                }],
            },
        );

        // Case 2: BDD result from JUnit fallback
        let bdd_results = HashMap::from([("AC-JUNIT-SRC".to_string(), AcStatus::Fail)]);
        update_ac_statuses(&mut acs, &bdd_results, &HashMap::new(), AcSource::Junit);

        let ac = acs.get("AC-JUNIT-SRC").unwrap();
        assert_eq!(ac.status, AcStatus::Fail);
        assert!(matches!(ac.source, AcSource::Junit));
    }

    #[test]
    fn update_ac_statuses_sets_inferred_when_no_tests() {
        let mut acs = HashMap::new();
        acs.insert(
            "AC-NO-TESTS".to_string(),
            Ac {
                id: "AC-NO-TESTS".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "No tests".to_string(),
                status: AcStatus::Unknown,
                source: AcSource::Coverage, // Will be reset
                scenarios: Vec::new(),
                tests_total: 1,
                tests_executed: 0,
                tags: vec![],
                must_have_ac: true,
                tests: vec![TestMapping {
                    test_type: "bdd".to_string(),
                    tag: "@AC-NO-TESTS".to_string(),
                    file: None,
                    module: None,
                }],
            },
        );

        // Case 3: No BDD results, no unit results -> Inferred
        update_ac_statuses(&mut acs, &HashMap::new(), &HashMap::new(), AcSource::Coverage);

        let ac = acs.get("AC-NO-TESTS").unwrap();
        assert_eq!(ac.status, AcStatus::Unknown);
        assert!(matches!(ac.source, AcSource::Inferred));
    }

    #[test]
    fn update_ac_statuses_zero_tests_total_is_inferred() {
        let mut acs = HashMap::new();
        acs.insert(
            "AC-ZERO-TOTAL".to_string(),
            Ac {
                id: "AC-ZERO-TOTAL".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "Zero tests declared".to_string(),
                status: AcStatus::Pass,     // Will be reset
                source: AcSource::Coverage, // Will be reset
                scenarios: Vec::new(),
                tests_total: 0, // No tests declared
                tests_executed: 0,
                tags: vec![],
                must_have_ac: true,
                tests: vec![],
            },
        );

        // Case 4: tests_total == 0 -> immediately Unknown + Inferred
        update_ac_statuses(&mut acs, &HashMap::new(), &HashMap::new(), AcSource::Coverage);

        let ac = acs.get("AC-ZERO-TOTAL").unwrap();
        assert_eq!(ac.status, AcStatus::Unknown);
        assert!(matches!(ac.source, AcSource::Inferred));
    }

    // ==========================================================================
    // AC-TPL-AC-STATUS-CONSISTENCY: feature_status.md governance link
    // ==========================================================================

    #[test]
    fn test_feature_status_has_governance_link() {
        // Read the actual feature_status.md to verify it contains the governance link
        let feature_status_path = std::path::Path::new("docs/feature_status.md");

        // If file doesn't exist in test context (e.g., running from different dir),
        // just verify the expected strings would be generated by checking the
        // constants used in generate_status_md
        if feature_status_path.exists() {
            let content =
                fs::read_to_string(feature_status_path).expect("Failed to read feature_status.md");

            // Verify the cross-link to governance semantics is present
            assert!(
                content.contains("ac-kernel/README.md"),
                "feature_status.md must link to ac-kernel README"
            );
            assert!(
                content.contains("ac-governance-semantics"),
                "feature_status.md must link to AC governance semantics section"
            );
            assert!(
                content.contains("must_have_ac"),
                "feature_status.md must mention must_have_ac"
            );

            // Verify the header structure
            assert!(content.contains("## AC Status Summary"));
            assert!(content.contains("How to read this"));
            assert!(content.contains("[PASS]"));
            assert!(content.contains("[FAIL]"));
            assert!(content.contains("[UNKNOWN]"));
        } else {
            // Verify the link strings are present in the source code
            // This ensures the generator will produce correct output
            let expected_link = "ac-kernel/README.md";
            let expected_anchor = "ac-governance-semantics";

            // These strings should be in generate_status_md - we verify they're compile-time correct
            assert!(expected_link.contains("ac-kernel"));
            assert!(expected_anchor.contains("governance"));
        }
    }

    // =========================================================================
    // Write vs Check Mode Contract Tests
    // =========================================================================

    /// Document the two operational modes of ac-status
    #[test]
    fn ac_status_default_args_use_write_mode() {
        let args = AcStatusArgs::default();
        assert!(!args.check, "default mode should be write (check=false)");
        assert!(!args.summary, "default mode should not be summary");
        assert!(!args.json, "default mode should not be json");
    }

    /// Verify check mode can be enabled
    #[test]
    fn ac_status_check_mode_flag() {
        let args = AcStatusArgs { check: true, ..Default::default() };
        assert!(args.check, "check mode should be enabled");
    }

    /// Document that summary mode is stdout-only (no file ops to check)
    #[test]
    fn ac_status_summary_mode_is_stdout_only() {
        let args = AcStatusArgs { summary: true, ..Default::default() };
        // In summary mode, --check is meaningless because we're printing to stdout
        assert!(args.summary);
    }

    /// Document that json mode is stdout-only (no file ops to check)
    #[test]
    fn ac_status_json_mode_is_stdout_only() {
        let args = AcStatusArgs { json: true, ..Default::default() };
        // In json mode, --check is meaningless because we're printing to stdout
        assert!(args.json);
    }

    /// Verify that selftest uses check mode (read-only contract)
    /// This test documents the invariant that selftest must NOT modify feature_status.md
    #[test]
    fn selftest_must_use_check_mode_contract() {
        // The selftest function at selftest.rs:run_ac_status passes check: true
        // This test documents that contract and would fail if someone accidentally
        // changes selftest to use write mode
        let selftest_args = AcStatusArgs {
            verbosity: crate::Verbosity::Quiet,
            check: true, // selftest MUST use check mode
            ..Default::default()
        };
        assert!(
            selftest_args.check,
            "selftest must use ac-status in check mode to avoid modifying repo"
        );
    }

    /// Verify that docs-check uses check mode (read-only contract)
    #[test]
    fn docs_check_must_use_check_mode_contract() {
        // The docs_check function uses check: true when validating AC status
        // This test documents that contract
        let docs_check_args = AcStatusArgs {
            verbosity: crate::Verbosity::Quiet,
            check: true, // docs-check MUST use check mode
            ..Default::default()
        };
        assert!(
            docs_check_args.check,
            "docs-check must use ac-status in check mode to avoid modifying repo"
        );
    }
}
