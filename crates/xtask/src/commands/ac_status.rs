use anyhow::{Context, Result};
use colored::Colorize;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spec_runtime::ledger::TestMapping;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use super::ac_parsing::{
    AC_PATTERN_WITH_AT, AcStatus, SCENARIO_PATTERN, Scenario, TAG_PATTERN,
    TESTCASE_SCENARIO_PATTERN, TESTCASE_SUFFIX_PATTERN, parse_ac_coverage,
    parse_cucumber_json_with_scenarios, parse_features_with_metadata, parse_junit_with_scenarios,
};

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
}

impl Default for AcStatusArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            features_dir: PathBuf::from("specs/features"),
            coverage: PathBuf::from("target/ac/coverage.jsonl"),
            junit: PathBuf::from("target/junit/acceptance.xml"),
            json_report: Some(PathBuf::from("target/ac_report.json")),
            output: PathBuf::from("docs/feature_status.md"),
            verbosity: crate::Verbosity::Normal,
            summary: false,
            json: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Ac {
    id: String,
    story_id: String,
    req_id: String,
    text: String,
    status: AcStatus,
    scenarios: Vec<String>,
    tests: Vec<TestMapping>,
    tests_total: usize,
    tests_executed: usize,
    tags: Vec<String>,
    must_have_ac: bool,
}

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

/// JSON output structure for ac-status
#[derive(Debug, Serialize)]
struct AcStatusJson {
    timestamp: String,
    kernel_acs: AcCategoryStats,
    template_acs: AcCategoryStats,
    coverage_percent: f64,
    acs: Vec<AcJson>,
}

#[derive(Debug, Serialize)]
struct AcCategoryStats {
    total: usize,
    passing: usize,
    failing: usize,
    unknown: usize,
}

#[derive(Debug, Serialize)]
struct AcJson {
    id: String,
    story_id: String,
    req_id: String,
    text: String,
    status: AcStatus,
    scenarios: Vec<String>,
    tests: Vec<TestMapping>,
    tests_total: usize,
    tests_executed: usize,
}

impl AcStatus {
    fn icon(&self) -> &str {
        match self {
            AcStatus::Pass => "[PASS]",
            AcStatus::Fail => "[FAIL]",
            AcStatus::Unknown => "[UNKNOWN]",
        }
    }

    fn name(&self) -> &str {
        match self {
            AcStatus::Pass => "pass",
            AcStatus::Fail => "fail",
            AcStatus::Unknown => "unknown",
        }
    }
}

// Ledger deserialization structures
#[derive(Debug, Deserialize)]
struct Ledger {
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    id: String,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    id: String,
    /// Tags for categorizing requirements (e.g., @tier1, @security).
    /// Future: Used for filtering ACs by tag in `cargo xtask ac-list --tag <TAG>`.
    /// See TASK-DX-AC-FILTERING for planned tag-based AC queries.
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    id: String,
    text: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    must_have_ac: bool,
    #[serde(default)]
    tests: Vec<TestMapping>,
}

pub fn run(args: AcStatusArgs) -> Result<()> {
    // Validate inputs
    if !args.ledger.exists() {
        anyhow::bail!("Ledger file not found: {}", args.ledger.display());
    }
    if !args.features_dir.exists() {
        anyhow::bail!("Features directory not found: {}", args.features_dir.display());
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
    let (scenarios, bdd_results) = if coverage_status == ReportStatus::NonEmpty {
        if should_print_progress {
            eprintln!("Parsing AC coverage (primary path): {}", args.coverage.display());
        }
        let (scenario_map, results) = parse_ac_coverage(&args.coverage)?;
        if should_print_progress {
            eprintln!("  Found {} scenarios", scenario_map.len());
            eprintln!("  Found results for {} ACs", results.len());
        }
        (scenario_map, results)
    } else if junit_status == ReportStatus::NonEmpty {
        if should_print_progress {
            eprintln!(
                "Coverage not found ({}); falling back to JUnit: {}",
                coverage_status,
                args.junit.display()
            );
        }
        fallback_to_junit(&args, should_print_progress)?
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
            (scenario_map, results)
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
            (HashMap::new(), HashMap::new())
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
        (HashMap::new(), HashMap::new())
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
    update_ac_statuses(&mut acs, &bdd_results, &unit_results);

    if args.json {
        // Output structured JSON
        print_json_output(&acs)?;
    } else if args.summary {
        // Print concise summary instead of generating markdown file
        print_summary(&acs)?;
    } else {
        // Generate full markdown report
        if should_print_progress {
            eprintln!("Generating status: {}", args.output.display());
        }
        generate_status_md(&mut acs, &scenarios, &args.output, &args, should_print_progress)?;

        // Check for failures
        let failed: Vec<_> = acs
            .values()
            .filter(|ac| ac.status == AcStatus::Fail)
            .map(|ac| ac.id.as_str())
            .collect();

        if !failed.is_empty() {
            eprintln!("\n{} {} AC(s) failed: {}", "[FAIL]".red(), failed.len(), failed.join(", "));
            anyhow::bail!("One or more ACs failed");
        }

        eprintln!("\n{} All ACs passed", "[OK]".green());
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
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    let mut acs = HashMap::new();

    for story in ledger.stories {
        for req in story.requirements {
            for ac in req.acceptance_criteria {
                let tests_total = ac.tests.iter().filter(|t| is_automated_test(t)).count();
                acs.insert(
                    ac.id.clone(),
                    Ac {
                        id: ac.id.clone(),
                        story_id: story.id.clone(),
                        req_id: req.id.clone(),
                        text: ac.text,
                        status: AcStatus::Unknown,
                        scenarios: Vec::new(),
                        tests: ac.tests,
                        tests_total,
                        tests_executed: 0,
                        tags: ac.tags,
                        must_have_ac: ac.must_have_ac,
                    },
                );
            }
        }
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
) {
    for ac in acs.values_mut() {
        ac.tests_executed = 0;

        if ac.tests_total == 0 {
            ac.status = AcStatus::Unknown;
            continue;
        }

        let mut failed = false;

        for test in &ac.tests {
            if !is_automated_test(test) {
                continue;
            }

            let outcome = match test.test_type.to_lowercase().as_str() {
                "bdd" | "integration" => bdd_results
                    .get(&ac.id)
                    .cloned()
                    .map(|status| match status {
                        AcStatus::Pass => TestOutcome::Pass,
                        AcStatus::Fail => TestOutcome::Fail,
                        AcStatus::Unknown => TestOutcome::Missing,
                    })
                    .unwrap_or(TestOutcome::Missing),
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
    let passing = acs.values().filter(|ac| ac.status == AcStatus::Pass).count();
    let failing = acs.values().filter(|ac| ac.status == AcStatus::Fail).count();
    let unknown = acs.values().filter(|ac| ac.status == AcStatus::Unknown).count();

    println!("AC Status Summary:");
    println!("  {} {} passing", AcStatus::Pass.icon(), passing);
    println!("  {} {} failing", AcStatus::Fail.icon(), failing);
    println!("  {} {} unknown (no mapped tests)", AcStatus::Unknown.icon(), unknown);

    if failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

fn print_json_output(acs: &HashMap<String, Ac>) -> Result<()> {
    // Separate kernel ACs (AC-KERN-*) from template ACs (AC-TPL-*)
    let kernel_acs: Vec<_> = acs.values().filter(|ac| ac.id.starts_with("AC-KERN-")).collect();
    let template_acs: Vec<_> = acs.values().filter(|ac| ac.id.starts_with("AC-TPL-")).collect();

    let kernel_stats = AcCategoryStats {
        total: kernel_acs.len(),
        passing: kernel_acs.iter().filter(|ac| ac.status == AcStatus::Pass).count(),
        failing: kernel_acs.iter().filter(|ac| ac.status == AcStatus::Fail).count(),
        unknown: kernel_acs.iter().filter(|ac| ac.status == AcStatus::Unknown).count(),
    };

    let template_stats = AcCategoryStats {
        total: template_acs.len(),
        passing: template_acs.iter().filter(|ac| ac.status == AcStatus::Pass).count(),
        failing: template_acs.iter().filter(|ac| ac.status == AcStatus::Fail).count(),
        unknown: template_acs.iter().filter(|ac| ac.status == AcStatus::Unknown).count(),
    };

    let total_passing = kernel_stats.passing + template_stats.passing;
    let total_acs = kernel_stats.total + template_stats.total;
    let coverage_percent =
        if total_acs > 0 { (total_passing as f64 / total_acs as f64) * 100.0 } else { 0.0 };

    // Convert ACs to JSON format
    let mut acs_vec: Vec<_> = acs.values().collect();
    acs_vec.sort_by(|a, b| a.id.cmp(&b.id));

    let acs_json: Vec<AcJson> = acs_vec
        .into_iter()
        .map(|ac| AcJson {
            id: ac.id.clone(),
            story_id: ac.story_id.clone(),
            req_id: ac.req_id.clone(),
            text: ac.text.clone(),
            status: ac.status.clone(),
            scenarios: ac.scenarios.clone(),
            tests: ac.tests.clone(),
            tests_total: ac.tests_total,
            tests_executed: ac.tests_executed,
        })
        .collect();

    let output = AcStatusJson {
        timestamp: chrono::Utc::now().to_rfc3339(),
        kernel_acs: kernel_stats,
        template_acs: template_stats,
        coverage_percent,
        acs: acs_json,
    };

    let json_output =
        serde_json::to_string_pretty(&output).context("Failed to serialize AC status to JSON")?;

    println!("{}", json_output);

    // Check for failures (still need to fail the command if tests failed)
    if output.kernel_acs.failing > 0 || output.template_acs.failing > 0 {
        anyhow::bail!("One or more ACs failed");
    }

    Ok(())
}

/// Extract template_version from specs/spec_ledger.yaml for feature_status metadata
fn get_template_version() -> Option<String> {
    if let Ok(content) = fs::read_to_string("specs/spec_ledger.yaml") {
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

/// Extract last_updated from specs/spec_ledger.yaml for feature_status metadata
fn get_last_updated_date() -> Option<String> {
    if let Ok(content) = fs::read_to_string("specs/spec_ledger.yaml") {
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

fn generate_status_md(
    acs: &mut HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    output_path: &Path,
    _args: &AcStatusArgs,
    should_print_progress: bool,
) -> Result<()> {
    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

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
    output.push_str(">   for meta / CI-only ACs (see sections at the end).\n\n");
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
        for ac in service_unmapped {
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

    fs::write(output_path, output)?;
    if should_print_progress {
        eprintln!("{} Generated {}", "[OK]".green(), output_path.display());
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

        update_ac_statuses(&mut acs, &bdd_results, &unit_results);

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

        update_ac_statuses(&mut acs, &bdd_results, &unit_results);
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

    #[test]
    fn ac_status_json_shape_is_stable() {
        // This test documents the stable JSON contract for AI/IDP consumers
        // Changes to this test indicate a breaking change to the --json output
        let output = AcStatusJson {
            timestamp: "2025-11-27T00:00:00Z".to_string(),
            kernel_acs: AcCategoryStats { total: 10, passing: 8, failing: 1, unknown: 1 },
            template_acs: AcCategoryStats { total: 5, passing: 4, failing: 0, unknown: 1 },
            coverage_percent: 80.0,
            acs: vec![AcJson {
                id: "AC-TEST-001".to_string(),
                story_id: "US-TEST-001".to_string(),
                req_id: "REQ-TEST-001".to_string(),
                text: "Test AC".to_string(),
                status: AcStatus::Pass,
                scenarios: vec!["Test scenario".to_string()],
                tests: vec![],
                tests_total: 1,
                tests_executed: 1,
            }],
        };

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Required top-level fields
        let required_fields =
            ["timestamp", "kernel_acs", "template_acs", "coverage_percent", "acs"];
        for field in &required_fields {
            assert!(
                parsed.get(*field).is_some(),
                "Missing required field '{}' in ac-status --json output",
                field
            );
        }

        // Verify acs is an array
        assert!(parsed["acs"].is_array(), "acs must be an array");

        // Verify AC object shape
        let first_ac = &parsed["acs"][0];
        let ac_fields =
            ["id", "story_id", "req_id", "text", "status", "tests_total", "tests_executed"];
        for field in &ac_fields {
            assert!(
                first_ac.get(*field).is_some(),
                "Missing required field '{}' in AC object",
                field
            );
        }

        // Verify category stats shape
        let stats_fields = ["total", "passing", "failing", "unknown"];
        for field in &stats_fields {
            assert!(
                parsed["kernel_acs"].get(*field).is_some(),
                "Missing required field '{}' in kernel_acs",
                field
            );
            assert!(
                parsed["template_acs"].get(*field).is_some(),
                "Missing required field '{}' in template_acs",
                field
            );
        }
    }
}
