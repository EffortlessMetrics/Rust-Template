//! JSON output schemas for ac-status.
//!
//! This module defines the stable JSON contract for `ac-status --json` output.
//! Changes to these structures should be reflected in the schema version.

use crate::model::{AcSource, AcStatus, TestMapping};
use serde::Serialize;
use std::collections::HashMap;

/// Current schema version for ac-status JSON output.
///
/// Bump this when making breaking changes to the JSON structure.
/// - v1.0: Initial schema with kernel/template prefix-based categorization
/// - v2.0: Switch to must_have_ac metadata-based categorization
pub const AC_STATUS_SCHEMA_VERSION: &str = "2.0";

/// JSON output structure for ac-status.
///
/// Schema version 2.0 - uses must_have_ac metadata instead of prefix-based categorization.
#[derive(Debug, Serialize)]
pub struct AcStatusJson {
    /// Schema version for forward compatibility (bump on breaking changes)
    pub schema_version: String,
    /// ISO 8601 timestamp of when the report was generated
    pub timestamp: String,
    /// Stats for ACs with must_have_ac=true (kernel/required ACs)
    pub must_have_acs: AcCategoryStats,
    /// Stats for ACs with must_have_ac=false (optional/exploratory ACs)
    pub optional_acs: AcCategoryStats,
    /// Overall coverage percentage (passing / total ACs)
    pub coverage_percent: f64,
    /// List of all ACs with their status details
    pub acs: Vec<AcJson>,
}

/// Category statistics for a group of ACs.
#[derive(Debug, Serialize)]
pub struct AcCategoryStats {
    /// Total number of ACs in this category
    pub total: usize,
    /// Number of passing ACs
    pub passing: usize,
    /// Number of failing ACs
    pub failing: usize,
    /// Number of unknown ACs (no tests executed)
    pub unknown: usize,
}

/// Single AC entry in JSON output.
#[derive(Debug, Serialize)]
pub struct AcJson {
    /// AC ID (e.g., "AC-TPL-001")
    pub id: String,
    /// Parent story ID
    pub story_id: String,
    /// Parent requirement ID
    pub req_id: String,
    /// AC description text
    pub text: String,
    /// Test result status
    pub status: AcStatus,
    /// Source of the status determination
    pub source: AcSource,
    /// Whether this AC is enforced in strict coverage mode
    pub must_have_ac: bool,
    /// List of BDD scenarios covering this AC
    pub scenarios: Vec<String>,
    /// Test mappings from the ledger
    pub tests: Vec<TestMapping>,
    /// Total number of tests declared
    pub tests_total: usize,
    /// Number of tests that were executed
    pub tests_executed: usize,
}

/// Internal AC representation used during status computation.
///
/// This is the working structure used by `ac-status` before converting
/// to the JSON output format.
#[derive(Debug, Clone)]
pub struct Ac {
    /// AC ID (e.g., "AC-TPL-001")
    pub id: String,
    /// Parent story ID
    pub story_id: String,
    /// Parent requirement ID
    pub req_id: String,
    /// AC description text
    pub text: String,
    /// Test result status
    pub status: AcStatus,
    /// Source of the status determination
    pub source: AcSource,
    /// List of BDD scenarios covering this AC
    pub scenarios: Vec<String>,
    /// Test mappings from the ledger
    pub tests: Vec<TestMapping>,
    /// Total number of automated tests declared
    pub tests_total: usize,
    /// Number of tests that were executed
    pub tests_executed: usize,
    /// Tags from the ledger
    pub tags: Vec<String>,
    /// Whether this AC must have BDD coverage
    pub must_have_ac: bool,
}

/// Build the JSON output structure from a map of ACs.
///
/// Computes category statistics and formats the output according to
/// the v2.0 schema.
pub fn build_status_json(acs: &HashMap<String, Ac>) -> AcStatusJson {
    // Categorize ACs by must_have_ac flag (kernel/required vs optional)
    let must_have_acs: Vec<_> = acs.values().filter(|ac| ac.must_have_ac).collect();
    let optional_acs: Vec<_> = acs.values().filter(|ac| !ac.must_have_ac).collect();

    let must_have_stats = AcCategoryStats {
        total: must_have_acs.len(),
        passing: must_have_acs.iter().filter(|ac| ac.status == AcStatus::Pass).count(),
        failing: must_have_acs.iter().filter(|ac| ac.status == AcStatus::Fail).count(),
        unknown: must_have_acs.iter().filter(|ac| ac.status == AcStatus::Unknown).count(),
    };

    let optional_stats = AcCategoryStats {
        total: optional_acs.len(),
        passing: optional_acs.iter().filter(|ac| ac.status == AcStatus::Pass).count(),
        failing: optional_acs.iter().filter(|ac| ac.status == AcStatus::Fail).count(),
        unknown: optional_acs.iter().filter(|ac| ac.status == AcStatus::Unknown).count(),
    };

    let total_passing = must_have_stats.passing + optional_stats.passing;
    let total_acs = acs.len();
    let coverage_percent =
        if total_acs > 0 { (total_passing as f64 / total_acs as f64) * 100.0 } else { 0.0 };

    // Convert ACs to JSON format, sorted by ID
    let mut acs_vec: Vec<_> = acs.values().collect();
    acs_vec.sort_by(|a, b| a.id.cmp(&b.id));

    let acs_json: Vec<AcJson> = acs_vec
        .into_iter()
        .map(|ac| AcJson {
            id: ac.id.clone(),
            story_id: ac.story_id.clone(),
            req_id: ac.req_id.clone(),
            text: ac.text.clone(),
            status: ac.status,
            source: ac.source,
            must_have_ac: ac.must_have_ac,
            scenarios: ac.scenarios.clone(),
            tests: ac.tests.clone(),
            tests_total: ac.tests_total,
            tests_executed: ac.tests_executed,
        })
        .collect();

    AcStatusJson {
        schema_version: AC_STATUS_SCHEMA_VERSION.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        must_have_acs: must_have_stats,
        optional_acs: optional_stats,
        coverage_percent,
        acs: acs_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ac(id: &str, status: AcStatus, must_have: bool) -> Ac {
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
            must_have_ac: must_have,
        }
    }

    #[test]
    fn build_status_json_computes_stats() {
        let mut acs = HashMap::new();
        acs.insert("AC-PASS-001".to_string(), make_test_ac("AC-PASS-001", AcStatus::Pass, true));
        acs.insert("AC-FAIL-001".to_string(), make_test_ac("AC-FAIL-001", AcStatus::Fail, true));
        acs.insert("AC-OPT-001".to_string(), make_test_ac("AC-OPT-001", AcStatus::Pass, false));

        let output = build_status_json(&acs);

        assert_eq!(output.schema_version, "2.0");
        assert_eq!(output.must_have_acs.total, 2);
        assert_eq!(output.must_have_acs.passing, 1);
        assert_eq!(output.must_have_acs.failing, 1);
        assert_eq!(output.optional_acs.total, 1);
        assert_eq!(output.optional_acs.passing, 1);
        assert_eq!(output.acs.len(), 3);
    }

    #[test]
    fn build_status_json_sorts_by_id() {
        let mut acs = HashMap::new();
        acs.insert("AC-ZZZ-001".to_string(), make_test_ac("AC-ZZZ-001", AcStatus::Pass, true));
        acs.insert("AC-AAA-001".to_string(), make_test_ac("AC-AAA-001", AcStatus::Pass, true));
        acs.insert("AC-MMM-001".to_string(), make_test_ac("AC-MMM-001", AcStatus::Pass, true));

        let output = build_status_json(&acs);

        assert_eq!(output.acs[0].id, "AC-AAA-001");
        assert_eq!(output.acs[1].id, "AC-MMM-001");
        assert_eq!(output.acs[2].id, "AC-ZZZ-001");
    }

    #[test]
    fn build_status_json_handles_empty() {
        let acs = HashMap::new();
        let output = build_status_json(&acs);

        assert_eq!(output.must_have_acs.total, 0);
        assert_eq!(output.optional_acs.total, 0);
        assert_eq!(output.coverage_percent, 0.0);
        assert!(output.acs.is_empty());
    }

    /// Shape lock test: Documents the stable JSON contract for AI/IDP consumers.
    /// Changes to this test indicate a breaking change to the --json output.
    #[test]
    fn ac_status_json_shape_is_stable() {
        let mut acs = HashMap::new();
        acs.insert("AC-TEST-001".to_string(), make_test_ac("AC-TEST-001", AcStatus::Pass, true));

        let output = build_status_json(&acs);
        let json_str = serde_json::to_string_pretty(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Required top-level fields (v2.0 schema)
        let required_fields = [
            "schema_version",
            "timestamp",
            "must_have_acs",
            "optional_acs",
            "coverage_percent",
            "acs",
        ];
        for field in &required_fields {
            assert!(
                parsed.get(*field).is_some(),
                "Missing required field '{}' in ac-status --json output",
                field
            );
        }

        // Verify schema_version is 2.0
        assert_eq!(parsed["schema_version"].as_str().unwrap(), "2.0");

        // Verify AC object shape
        let first_ac = &parsed["acs"][0];
        let ac_fields = [
            "id",
            "story_id",
            "req_id",
            "text",
            "status",
            "source",
            "must_have_ac",
            "tests_total",
            "tests_executed",
        ];
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
                parsed["must_have_acs"].get(*field).is_some(),
                "Missing required field '{}' in must_have_acs",
                field
            );
        }
    }
}
