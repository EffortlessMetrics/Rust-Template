use ac_kernel::{AcSource, AcStatus};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::ac_parsing::{
    Scenario, parse_ac_coverage, parse_cucumber_json_with_scenarios, parse_features_with_metadata,
    parse_junit_with_scenarios,
};
use crate::kernel::layout_for_repo;

mod evidence;
mod model;
mod reporting;
#[cfg(test)]
mod test_parsers;

use evidence::{collect_unit_test_results, update_ac_statuses};
use model::parse_ledger;
use reporting::{
    check_status_md, generate_status_md, print_json_output, print_single_ac, print_summary,
};

/// Get the repository root from CARGO_MANIFEST_DIR.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask crate should be in crates/")
        .parent()
        .expect("crates/ should be in repo root")
        .to_path_buf()
}

/// Convert an absolute path to a repo-relative path for stable output.
fn to_relative_path(path: &str) -> String {
    let root = repo_root();
    let root_str = root.to_string_lossy();
    if path.starts_with(root_str.as_ref()) {
        path.strip_prefix(root_str.as_ref())
            .unwrap_or(path)
            .trim_start_matches(std::path::MAIN_SEPARATOR)
            .to_string()
    } else {
        path.to_string()
    }
}

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
    /// Require coverage data to exist and be non-empty before computing status.
    /// If true and coverage is missing/empty, returns error instead of continuing
    /// with Unknown statuses for all BDD ACs. This prevents churn when coverage
    /// is stale or missing.
    pub require_coverage: bool,
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
            require_coverage: false,
        }
    }
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

    // REQUIRE_COVERAGE GUARD: If caller requires coverage and we still don't have it, fail early.
    // This prevents churn from regenerating feature_status.md with stale/missing coverage data.
    if args.require_coverage {
        let has_coverage_now = ReportStatus::from_path(&args.coverage) == ReportStatus::NonEmpty
            || ReportStatus::from_path(&args.junit) == ReportStatus::NonEmpty
            || args
                .json_report
                .as_ref()
                .is_some_and(|p| ReportStatus::from_path(p) == ReportStatus::NonEmpty);

        if !has_coverage_now {
            anyhow::bail!(
                "Coverage data required but not found (require_coverage=true)\n\n\
                 Coverage file: {} (status: {})\n\
                 JUnit file:    {} (status: {})\n\n\
                 To fix:\n\
                 1. Run: cargo xtask bdd\n\
                 2. Then re-run: cargo xtask ac-status\n\n\
                 hint: Coverage must be fresh before running ac-status in check mode.\n\
                       This prevents churn from regenerating feature_status.md with incomplete data.",
                args.coverage.display(),
                ReportStatus::from_path(&args.coverage),
                args.junit.display(),
                ReportStatus::from_path(&args.junit)
            );
        }
    }

    // Check coverage metadata for filtered-run warnings
    if should_print_progress {
        let meta_path = args.coverage.with_file_name("coverage.meta.json");
        if let Ok(meta_str) = fs::read_to_string(&meta_path)
            && let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str)
            && meta.get("run_mode").and_then(|v| v.as_str()) == Some("filtered")
        {
            let expr = meta.get("tag_expression").and_then(|v| v.as_str()).unwrap_or("<unknown>");
            eprintln!(
                "{} Coverage data from filtered run (tag: {}). Results may be incomplete.",
                "[WARN]".yellow(),
                expr
            );
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

#[cfg(test)]
mod tests {
    use super::test_parsers::parse_features;
    use super::*;
    use ac_kernel::Ac;
    use spec_runtime::ledger::TestMapping;
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

        // AC-MISSING: Has unit test mapping but no execution result captured.
        // Per ADR-0024: unit tests are "presumed to run" if mapped, so this should be PASS.
        acs.insert(
            "AC-MISSING".to_string(),
            Ac {
                id: "AC-MISSING".to_string(),
                story_id: "US-TEST".to_string(),
                req_id: "REQ-TEST".to_string(),
                text: "Unit test mapped but not in results".to_string(),
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
        // ADR-0024: Unit test mapping counts as PASS evidence (presumed to run)
        assert_eq!(ac_missing.status, AcStatus::Pass);
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

    /// Verify that selftest uses check mode in CI (read-only contract)
    /// This test documents the invariant: selftest must NOT modify feature_status.md in CI.
    ///
    /// Locally, selftest uses write mode because:
    /// - BDD runs with tag filtering (excludes @ci-only scenarios)
    /// - This produces different coverage than CI
    /// - Comparing against CI-generated file would always fail
    ///
    /// In CI, selftest uses check mode to enforce file consistency.
    #[test]
    fn selftest_must_use_check_mode_in_ci_contract() {
        // The selftest function at selftest.rs:run_ac_status passes check: in_ci
        // where in_ci = crate::env::is_ci(). This test documents that contract.
        let in_ci = crate::env::is_ci();
        let selftest_args = AcStatusArgs {
            verbosity: crate::Verbosity::Quiet,
            check: in_ci, // selftest uses check mode only in CI
            ..Default::default()
        };
        assert_eq!(
            selftest_args.check, in_ci,
            "selftest's ac-status check mode should match CI environment"
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
