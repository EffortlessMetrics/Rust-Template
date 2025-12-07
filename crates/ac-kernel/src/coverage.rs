//! AC Coverage: Parse coverage.jsonl files produced by BDD tests.
//!
//! This module handles the streaming JSONL format used to capture AC coverage
//! from cucumber tests. The format is resilient to process exits (each line
//! is flushed immediately).

use crate::model::{AcStatus, Scenario};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A single AC coverage record from the JSONL file.
///
/// This is the wire format for streaming coverage results from BDD tests.
/// Each scenario completion produces one record per AC ID found in the tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcCoverageRecord {
    /// The AC ID (e.g., "AC-KERN-001")
    pub ac_id: String,
    /// The status of the scenario: "passed", "failed", or "skipped"
    pub status: String,
    /// The feature file path
    pub feature: String,
    /// The scenario name
    pub scenario: String,
    /// All tags from feature, rule, and scenario
    pub tags: Vec<String>,
}

/// Parse the AC coverage JSONL file and extract AC test results.
///
/// This is the preferred primary source for AC coverage as it streams
/// results and is resilient to cucumber's `exit()` behavior.
///
/// # Scenario Identity
///
/// Scenarios are keyed by `feature::scenario_name` to avoid collisions when
/// the same scenario name appears in multiple feature files.
///
/// # Skipped Scenario Semantics
///
/// Skipped scenarios (status = "skipped") are excluded from pass/fail aggregation:
/// - AC with only "passed" scenarios -> Pass
/// - AC with any "failed" scenarios -> Fail
/// - AC with only "skipped" scenarios -> no entries -> Unknown (when merged with ledger)
///
/// This means "skipped" is treated as "not proven" rather than "failed".
///
/// # Returns
///
/// Returns tuple of `(scenarios, ac_results)`:
/// - `scenarios`: Map of scenario key to Scenario metadata
/// - `ac_results`: Map of AC ID to aggregated status
pub fn parse_ac_coverage(
    coverage_path: &Path,
) -> Result<(HashMap<String, Scenario>, HashMap<String, AcStatus>)> {
    let file = match fs::File::open(coverage_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // No coverage file yet - return empty results
            return Ok((HashMap::new(), HashMap::new()));
        }
        Err(e) => {
            return Err(anyhow::anyhow!(e).context("opening AC coverage file"));
        }
    };

    let reader = BufReader::new(file);
    let mut scenarios: HashMap<String, Scenario> = HashMap::new();
    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let record: AcCoverageRecord =
            serde_json::from_str(&line).context("parsing AC coverage line")?;

        // Use feature::scenario_name as key to avoid collisions across features
        let scenario_key = format!("{}::{}", record.feature, record.scenario);

        // Store scenario information
        scenarios.insert(
            scenario_key,
            Scenario {
                name: record.scenario.clone(),
                ac_id: record.ac_id.clone(),
                file: record.feature.clone(),
            },
        );

        // Determine if scenario passed.
        // Skipped scenarios are excluded from aggregation - an AC with only skipped
        // scenarios will have no entries and be treated as Unknown when merged with ledger.
        let passed = match record.status.as_str() {
            "passed" => true,
            "failed" => false,
            "skipped" => continue,
            _ => continue,
        };

        ac_results.entry(record.ac_id).or_default().push(passed);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        let path = Path::new("/nonexistent/path/coverage.jsonl");
        let (scenarios, results) = parse_ac_coverage(path).unwrap();

        assert!(scenarios.is_empty());
        assert!(results.is_empty());
    }

    #[test]
    fn coverage_record_round_trip() {
        let record = AcCoverageRecord {
            ac_id: "AC-TEST-001".to_string(),
            status: "passed".to_string(),
            feature: "test.feature".to_string(),
            scenario: "Test scenario".to_string(),
            tags: vec!["AC-TEST-001".to_string(), "smoke".to_string()],
        };

        let json = serde_json::to_string(&record).unwrap();
        let parsed: AcCoverageRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ac_id, "AC-TEST-001");
        assert_eq!(parsed.status, "passed");
        assert_eq!(parsed.feature, "test.feature");
        assert_eq!(parsed.scenario, "Test scenario");
        assert_eq!(parsed.tags.len(), 2);
    }
}
