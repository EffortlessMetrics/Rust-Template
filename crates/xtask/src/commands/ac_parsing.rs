//! Shared AC parsing logic for ac_coverage and ac_status commands.
//!
//! This module provides common functionality for:
//! - Parsing spec_ledger.yaml to extract ACs
//! - Parsing feature files to extract scenarios
//! - Parsing test results (JUnit XML and Cucumber JSON)
//! - Mapping scenarios to ACs
//!
//! NOTE: Core types (AcStatus, Scenario, AcMetadata, AcDetails, AcCoverageRecord)
//! are now defined in the `ac-kernel` crate. This module re-exports them for
//! backwards compatibility and provides additional parsing functionality.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Re-export core types from ac-kernel for backwards compatibility
pub use ac_kernel::{
    AcDetails, AcMetadata, AcStatus, Scenario, get_ac_details, parse_ac_coverage,
    parse_ledger_with_metadata,
};
// Note: AcCoverageRecord is available from ac_kernel but not re-exported here
// as it's only used by the acceptance crate's coverage_writer.

// Precompiled regex patterns for performance
pub(crate) static AC_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(AC-[A-Z0-9-]+)$").unwrap());
pub(crate) static AC_PATTERN_WITH_AT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@(AC-[A-Z0-9-]+)").unwrap());
pub(crate) static SCENARIO_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*Scenario(?:\s+Outline)?:\s+(.+)").unwrap());
pub(crate) static TAG_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"@[\w-]+").unwrap());
pub(crate) static TESTCASE_SCENARIO_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Scenario:\s+(.+?):\s+").unwrap());
pub(crate) static TESTCASE_SUFFIX_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s*\((?:row|example)\s+\d+\)\s*$").unwrap());

// Type aliases to reduce complexity
pub type AcsByReq = BTreeMap<String, Vec<String>>;
pub type AcToReqMap = HashMap<String, String>;

// ============================================================================
// Ledger Parsing
// ============================================================================

// Internal YAML structs for parse_ledger (kept local for backwards compatibility)
// Note: These duplicate the structs in ac-kernel::ledger but are needed for parse_ledger().
// Consider consolidating in a future refactor.

#[derive(Debug, Deserialize)]
struct Ledger {
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    #[allow(dead_code)]
    id: String,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    title: String,
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
    #[serde(default = "default_must_have_ac")]
    #[allow(dead_code)]
    must_have_ac: bool,
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

fn default_must_have_ac() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    id: String,
    #[allow(dead_code)]
    text: String,
    #[serde(default = "default_must_have_ac")]
    #[allow(dead_code)]
    must_have_ac: bool,
}

/// Parse the spec_ledger.yaml file and return all ACs mapped to their parent requirement ID.
///
/// Returns:
/// - all_acs: HashMap<AC_ID, REQ_ID> - maps each AC to its parent requirement
/// - acs_by_req: BTreeMap<REQ_ID, Vec<AC_ID>> - groups ACs by requirement
///
/// NOTE: For richer metadata including `must_have_ac`, use `parse_ledger_with_metadata`
/// from `ac-kernel` (re-exported above).
pub fn parse_ledger(ledger_path: &Path) -> Result<(AcToReqMap, AcsByReq)> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    let mut all_acs: HashMap<String, String> = HashMap::new();
    let mut acs_by_req: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for story in ledger.stories {
        for req in story.requirements {
            let mut req_acs = Vec::new();
            for ac in req.acceptance_criteria {
                all_acs.insert(ac.id.clone(), req.id.clone());
                req_acs.push(ac.id.clone());
            }
            if !req_acs.is_empty() {
                acs_by_req.insert(req.id.clone(), req_acs);
            }
        }
    }

    Ok((all_acs, acs_by_req))
}

// NOTE: `parse_ledger_with_metadata` and `get_ac_details` are now provided by ac-kernel
// and re-exported above. The local implementations have been removed.

// ============================================================================
// Feature File Parsing
// ============================================================================

/// Parse feature files and extract scenarios with their AC tags.
///
/// Returns HashMap<scenario_title, AC_ID> mapping scenario names to ACs.
pub fn parse_features(features_dir: &Path) -> Result<HashMap<String, String>> {
    let mut scenarios: HashMap<String, String> = HashMap::new();

    if !features_dir.exists() {
        return Ok(scenarios);
    }

    for entry in WalkDir::new(features_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "feature"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let mut current_acs: Vec<String> = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim_start();

                // Look for AC tags
                if let Some(ac_id) = trimmed.strip_prefix('@').and_then(|s| {
                    if s.starts_with("AC-") {
                        Some(s.split_whitespace().next()?.to_string())
                    } else {
                        None
                    }
                }) {
                    current_acs.push(ac_id);
                }

                // Look for Scenario lines
                if trimmed.starts_with("Scenario:") {
                    if let Some(scenario_title) =
                        trimmed.strip_prefix("Scenario:").map(|s| s.trim())
                    {
                        // Map this scenario title to all current ACs
                        for ac_id in &current_acs {
                            scenarios.insert(scenario_title.to_string(), ac_id.clone());
                        }
                    }
                    current_acs.clear();
                }
            }
        }
    }

    Ok(scenarios)
}

/// Parse feature files and extract scenarios with full metadata.
///
/// Returns HashMap<scenario_name, Scenario> with complete scenario information.
pub fn parse_features_with_metadata(features_dir: &Path) -> Result<HashMap<String, Scenario>> {
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

// ============================================================================
// Test Results Parsing - AC Coverage JSONL
// ============================================================================

// NOTE: `AcCoverageRecord` and `parse_ac_coverage` are now provided by ac-kernel
// and re-exported above. The local implementations have been removed.

// ============================================================================
// Test Results Parsing - Cucumber JSON
// ============================================================================

#[derive(Debug, Deserialize)]
struct CucumberReport(Vec<CucumberFeature>);

#[derive(Debug, Deserialize)]
struct CucumberFeature {
    /// Feature name from BDD .feature file.
    /// Future: Used in enhanced coverage reports to group scenarios by feature.
    /// See TASK-DX-BDD-REPORTING for planned BDD report improvements.
    #[allow(dead_code)]
    name: String,
    uri: String,
    elements: Vec<CucumberElement>,
}

#[derive(Debug, Deserialize)]
struct CucumberElement {
    name: String,
    #[serde(rename = "type")]
    element_type: String,
    tags: Vec<CucumberTag>,
    /// Line number in .feature file where scenario starts.
    /// Future: Used for generating clickable links in IDE integration.
    /// See TASK-DX-IDE-INTEGRATION for planned editor jump-to-definition features.
    #[allow(dead_code)]
    line: Option<u32>,
    steps: Vec<CucumberStep>,
}

#[derive(Debug, Deserialize)]
struct CucumberTag {
    name: String,
    /// Line number where tag appears in .feature file.
    /// Future: Used for IDE integration and precise source mapping.
    #[serde(default)]
    #[allow(dead_code)]
    line: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CucumberStep {
    /// Step keyword (Given/When/Then/And/But).
    /// Future: Used in detailed BDD failure reports showing which step failed.
    #[allow(dead_code)]
    keyword: String,
    /// Step description text.
    /// Future: Used in BDD failure output to show exact failing step.
    #[allow(dead_code)]
    name: String,
    /// Line number of step in .feature file.
    /// Future: Used for IDE integration to jump to failing step.
    #[allow(dead_code)]
    line: Option<u32>,
    result: CucumberStepResult,
}

#[derive(Debug, Deserialize)]
struct CucumberStepResult {
    status: String,
    /// Step execution duration in nanoseconds.
    /// Future: Used in performance regression detection for slow BDD scenarios.
    /// See TASK-DX-PERF-TRACKING for planned test performance monitoring.
    #[serde(default)]
    #[allow(dead_code)]
    duration: Option<u64>, // nanoseconds
    /// Error message if step failed.
    /// Future: Used in detailed failure reports with root cause analysis.
    #[serde(default)]
    #[allow(dead_code)]
    error_message: Option<String>,
}

/// Parse Cucumber JSON report and extract AC test results.
///
/// Returns HashMap<AC_ID, AcStatus> mapping AC IDs to their test status.
pub fn parse_cucumber_json(json_path: &Path) -> Result<HashMap<String, AcStatus>> {
    let content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON report: {}", json_path.display()))?;

    let report: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", json_path.display()))?;

    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();

    if let Some(features) = report.as_array() {
        for feature in features {
            if let Some(elements) = feature.get("elements").and_then(|e| e.as_array()) {
                for element in elements {
                    if let Some(element_type) = element.get("type").and_then(|t| t.as_str())
                        && element_type == "scenario"
                    {
                        // Extract AC IDs from tags
                        let ac_ids: Vec<String> = element
                            .get("tags")
                            .and_then(|t| t.as_array())
                            .map(|tags| {
                                tags.iter()
                                    .filter_map(|tag| {
                                        tag.get("name").and_then(|n| n.as_str()).and_then(|s| {
                                            if s.starts_with("AC-") {
                                                Some(s.to_string())
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Determine if scenario passed
                        let passed = element
                            .get("steps")
                            .and_then(|s| s.as_array())
                            .map(|steps| {
                                steps.iter().all(|step| {
                                    step.get("result")
                                        .and_then(|r| r.get("status"))
                                        .and_then(|st| st.as_str())
                                        .map(|status| status == "passed")
                                        .unwrap_or(false)
                                })
                            })
                            .unwrap_or(false);

                        for ac_id in ac_ids {
                            ac_results.entry(ac_id).or_default().push(passed);
                        }
                    }
                }
            }
        }
    }

    // Aggregate: AC passes only if all scenarios pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status = if results.iter().all(|&p| p) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok(ac_status)
}

/// Parse Cucumber JSON report and extract both scenarios and AC results.
///
/// Returns tuple of (scenarios, ac_results) for use by ac_status.rs.
pub fn parse_cucumber_json_with_scenarios(
    json_path: &Path,
) -> Result<(HashMap<String, Scenario>, HashMap<String, AcStatus>)> {
    let content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON report: {}", json_path.display()))?;

    let report: CucumberReport = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", json_path.display()))?;

    let mut scenarios: HashMap<String, Scenario> = HashMap::new();
    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();

    for feature in report.0 {
        for element in feature.elements {
            // Only process scenarios (not hooks or backgrounds)
            if element.element_type == "scenario" {
                // Extract AC IDs from tags (Cucumber JSON doesn't include @ in tag names)
                let ac_ids: Vec<String> = element
                    .tags
                    .iter()
                    .filter_map(|tag| {
                        AC_PATTERN.captures(&tag.name).map(|caps| caps[1].to_string())
                    })
                    .collect();

                // Determine if scenario passed (all steps passed)
                let passed = element.steps.iter().all(|step| step.result.status == "passed");

                // Record result for each AC ID
                for ac_id in &ac_ids {
                    ac_results.entry(ac_id.clone()).or_default().push(passed);

                    // Store scenario information (using first AC ID)
                    scenarios.insert(
                        element.name.clone(),
                        Scenario {
                            name: element.name.clone(),
                            ac_id: ac_id.clone(),
                            file: feature.uri.clone(),
                        },
                    );
                }
            }
        }
    }

    // Aggregate: AC passes only if all scenarios pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status =
            if results.iter().all(|&passed| passed) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok((scenarios, ac_status))
}

// ============================================================================
// Test Results Parsing - JUnit XML
// ============================================================================

/// Extract scenario title from JUnit testcase name.
///
/// Format: "Scenario: <title>: ?<path>:<line>:<col>"
pub fn extract_scenario_title(testcase_name: &str) -> Option<String> {
    if let Some(after_scenario) = testcase_name.strip_prefix("Scenario: ") {
        // Find the ": ?" separator that precedes the file path
        if let Some(idx) = after_scenario.find(": ?") {
            return Some(after_scenario[..idx].trim().to_string());
        }
    }
    None
}

/// Normalize JUnit testcase name by extracting scenario name and removing suffixes.
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

/// Parse JUnit XML and extract AC test results.
///
/// Returns HashMap<AC_ID, AcStatus> mapping AC IDs to their test status.
pub fn parse_junit(
    junit_path: &Path,
    scenarios: &HashMap<String, String>,
) -> Result<HashMap<String, AcStatus>> {
    let content = fs::read_to_string(junit_path)?;
    let mut reader = Reader::from_str(&content);
    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();
    let mut buf = Vec::new();

    let mut current_testcase: Option<String> = None;
    let mut testcase_passed = true;
    let mut has_skipped = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"testcase" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"name" {
                            let name = String::from_utf8_lossy(&attr.value).to_string();
                            current_testcase = Some(name);
                            testcase_passed = true;
                            has_skipped = false;
                        }
                    }
                }
                b"failure" | b"error" => {
                    testcase_passed = false;
                }
                b"skipped" => {
                    has_skipped = true;
                }
                _ => {}
            },
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"testcase"
                    && let Some(tc_name) = current_testcase.take()
                {
                    // Extract scenario title from junit testcase name
                    if let Some(scenario_title) = extract_scenario_title(&tc_name) {
                        // Look up AC ID from scenario title
                        if let Some(ac_id) = scenarios.get(&scenario_title) {
                            // Skip skipped tests - don't count them as pass or fail
                            if !has_skipped {
                                ac_results.entry(ac_id.clone()).or_default().push(testcase_passed);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Aggregate: AC passes only if all testcases pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status = if results.iter().all(|&p| p) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok(ac_status)
}

/// Parse JUnit XML and extract AC test results using Scenario metadata.
///
/// Returns HashMap<AC_ID, AcStatus> mapping AC IDs to their test status.
/// This version uses the more robust normalization for ac_status.rs.
pub fn parse_junit_with_scenarios(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_scenario_title() {
        assert_eq!(
            extract_scenario_title("Scenario: Basic test: ?path/to/file.feature:10:5"),
            Some("Basic test".to_string())
        );
        assert_eq!(
            extract_scenario_title("Scenario: Test with colons: More: ?file:1:1"),
            Some("Test with colons: More".to_string())
        );
        assert_eq!(extract_scenario_title("Not a scenario"), None);
    }

    #[test]
    fn test_normalize_testcase_name() {
        assert_eq!(normalize_testcase_name("Scenario: Test name: file.feature:10:5"), "Test name");
        assert_eq!(
            normalize_testcase_name("Scenario: Parameterized (row 1): file:10:5"),
            "Parameterized"
        );
        assert_eq!(
            normalize_testcase_name("Scenario: Example test (example 2): file:10:5"),
            "Example test"
        );
    }

    // ============================================================================
    // Tests for parse_ac_coverage
    // ============================================================================

    fn write_coverage_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_ac_coverage_all_pass() {
        let content = r#"{"ac_id":"AC-TEST-001","status":"passed","feature":"test.feature","scenario":"Scenario A","tags":["AC-TEST-001"]}
{"ac_id":"AC-TEST-001","status":"passed","feature":"test.feature","scenario":"Scenario B","tags":["AC-TEST-001"]}"#;

        let file = write_coverage_file(content);
        let (scenarios, results) = parse_ac_coverage(file.path()).unwrap();

        assert_eq!(scenarios.len(), 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get("AC-TEST-001"), Some(&AcStatus::Pass));
    }

    #[test]
    fn test_parse_ac_coverage_any_fail() {
        let content = r#"{"ac_id":"AC-TEST-001","status":"passed","feature":"test.feature","scenario":"Scenario A","tags":["AC-TEST-001"]}
{"ac_id":"AC-TEST-001","status":"failed","feature":"test.feature","scenario":"Scenario B","tags":["AC-TEST-001"]}"#;

        let file = write_coverage_file(content);
        let (_, results) = parse_ac_coverage(file.path()).unwrap();

        assert_eq!(results.get("AC-TEST-001"), Some(&AcStatus::Fail));
    }

    #[test]
    fn test_parse_ac_coverage_skipped_ignored() {
        // An AC with only skipped scenarios should have no entry
        let content = r#"{"ac_id":"AC-SKIPPED-001","status":"skipped","feature":"test.feature","scenario":"Scenario A","tags":["AC-SKIPPED-001"]}"#;

        let file = write_coverage_file(content);
        let (scenarios, results) = parse_ac_coverage(file.path()).unwrap();

        // Scenario is recorded but no pass/fail entry
        assert_eq!(scenarios.len(), 1);
        assert!(results.get("AC-SKIPPED-001").is_none());
    }

    #[test]
    fn test_parse_ac_coverage_skipped_with_passed() {
        // If some scenarios pass and some are skipped, the AC should pass
        let content = r#"{"ac_id":"AC-MIXED-001","status":"passed","feature":"test.feature","scenario":"Scenario A","tags":["AC-MIXED-001"]}
{"ac_id":"AC-MIXED-001","status":"skipped","feature":"test.feature","scenario":"Scenario B","tags":["AC-MIXED-001"]}"#;

        let file = write_coverage_file(content);
        let (_, results) = parse_ac_coverage(file.path()).unwrap();

        assert_eq!(results.get("AC-MIXED-001"), Some(&AcStatus::Pass));
    }

    #[test]
    fn test_parse_ac_coverage_multiple_acs() {
        let content = r#"{"ac_id":"AC-ONE","status":"passed","feature":"f1.feature","scenario":"Scenario 1","tags":["AC-ONE"]}
{"ac_id":"AC-TWO","status":"failed","feature":"f2.feature","scenario":"Scenario 2","tags":["AC-TWO"]}
{"ac_id":"AC-THREE","status":"passed","feature":"f3.feature","scenario":"Scenario 3","tags":["AC-THREE"]}"#;

        let file = write_coverage_file(content);
        let (_, results) = parse_ac_coverage(file.path()).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results.get("AC-ONE"), Some(&AcStatus::Pass));
        assert_eq!(results.get("AC-TWO"), Some(&AcStatus::Fail));
        assert_eq!(results.get("AC-THREE"), Some(&AcStatus::Pass));
    }

    #[test]
    fn test_parse_ac_coverage_scenario_identity() {
        // Same scenario name in different features should be distinct
        let content = r#"{"ac_id":"AC-FEAT-A","status":"passed","feature":"feature_a.feature","scenario":"Same Name","tags":["AC-FEAT-A"]}
{"ac_id":"AC-FEAT-B","status":"passed","feature":"feature_b.feature","scenario":"Same Name","tags":["AC-FEAT-B"]}"#;

        let file = write_coverage_file(content);
        let (scenarios, _) = parse_ac_coverage(file.path()).unwrap();

        // Should have 2 distinct scenario entries
        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.contains_key("feature_a.feature::Same Name"));
        assert!(scenarios.contains_key("feature_b.feature::Same Name"));
    }

    #[test]
    fn test_parse_ac_coverage_empty_file() {
        let file = write_coverage_file("");
        let (scenarios, results) = parse_ac_coverage(file.path()).unwrap();

        assert!(scenarios.is_empty());
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_ac_coverage_missing_file() {
        let path = std::path::Path::new("/nonexistent/path/coverage.jsonl");
        let (scenarios, results) = parse_ac_coverage(path).unwrap();

        assert!(scenarios.is_empty());
        assert!(results.is_empty());
    }

    // ============================================================================
    // Tests for parse_ledger_with_metadata - must_have_ac AND semantics
    // ============================================================================

    fn write_ledger_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_must_have_ac_defaults_to_true() {
        // When neither requirement nor AC specifies must_have_ac, it should default to true
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC without must_have_ac specified"
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(metadata.contains_key("AC-TEST-001"));
        assert!(
            metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should default to true when not specified"
        );
    }

    #[test]
    fn test_must_have_ac_and_semantics_both_true() {
        // When both requirement and AC have must_have_ac=true, effective is true
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: true
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be true when both REQ and AC are true"
        );
    }

    #[test]
    fn test_must_have_ac_and_semantics_req_false() {
        // When requirement has must_have_ac=false, even if AC is true, effective is false
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: true
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            !metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be false when REQ is false (AND semantics)"
        );
    }

    #[test]
    fn test_must_have_ac_and_semantics_ac_false() {
        // When AC has must_have_ac=false, even if REQ is true, effective is false
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: false
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            !metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be false when AC is false (AND semantics)"
        );
    }

    #[test]
    fn test_must_have_ac_and_semantics_both_false() {
        // When both are false, effective is false
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: false
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
            must_have_ac: false
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            !metadata.get("AC-TEST-001").unwrap().must_have_ac,
            "must_have_ac should be false when both are false"
        );
    }

    #[test]
    fn test_must_have_ac_mixed_acs_under_same_req() {
        // Different ACs under the same requirement can have different must_have_ac values
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        must_have_ac: true
        acceptance_criteria:
          - id: AC-KERNEL-001
            text: "Kernel AC"
            must_have_ac: true
          - id: AC-OPTIONAL-001
            text: "Optional AC"
            must_have_ac: false
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert!(
            metadata.get("AC-KERNEL-001").unwrap().must_have_ac,
            "AC-KERNEL-001 should be must_have_ac=true"
        );
        assert!(
            !metadata.get("AC-OPTIONAL-001").unwrap().must_have_ac,
            "AC-OPTIONAL-001 should be must_have_ac=false"
        );
    }

    #[test]
    fn test_must_have_ac_preserves_req_id() {
        // Verify that req_id is correctly preserved in metadata
        let content = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-PARENT-001
        acceptance_criteria:
          - id: AC-CHILD-001
            text: "Child AC"
"#;
        let file = write_ledger_file(content);
        let metadata = parse_ledger_with_metadata(file.path()).unwrap();

        assert_eq!(
            metadata.get("AC-CHILD-001").unwrap().req_id,
            "REQ-PARENT-001",
            "req_id should be correctly preserved"
        );
    }
}
