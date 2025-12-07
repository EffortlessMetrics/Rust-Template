//! Core data model types for AC governance.
//!
//! This module defines the shared types used across AC-related commands:
//! - [`AcStatus`]: Test result status (Pass, Fail, Unknown)
//! - [`AcSource`]: Where the status came from (Coverage, JUnit, JSON, Inferred)
//! - [`Scenario`]: BDD scenario metadata
//! - [`TestMapping`]: Test-to-AC mapping (re-exported from spec-runtime)

use serde::{Deserialize, Serialize};

// Re-export TestMapping from spec-runtime for convenience
pub use spec_runtime::ledger::TestMapping;

/// Status of an acceptance criterion based on test results.
///
/// # Semantics
///
/// - `Pass`: At least one test ran and all ran tests passed
/// - `Fail`: At least one test ran and failed
/// - `Unknown`: No tests have been executed for this AC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AcStatus {
    /// All executed tests passed
    Pass,
    /// At least one test failed
    Fail,
    /// No tests have been executed
    Unknown,
}

impl AcStatus {
    /// Returns a human-readable icon for the status.
    ///
    /// Used in CLI output and markdown reports.
    pub fn icon(&self) -> &'static str {
        match self {
            AcStatus::Pass => "[PASS]",
            AcStatus::Fail => "[FAIL]",
            AcStatus::Unknown => "[UNKNOWN]",
        }
    }

    /// Returns the lowercase name of the status.
    ///
    /// Used in markdown tables and JSON output.
    pub fn name(&self) -> &'static str {
        match self {
            AcStatus::Pass => "pass",
            AcStatus::Fail => "fail",
            AcStatus::Unknown => "unknown",
        }
    }
}

/// Source of AC status determination.
///
/// Tracks where the test result came from for debugging and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AcSource {
    /// Result came from coverage.jsonl (streaming BDD results)
    Coverage,
    /// Result came from JUnit XML fallback
    Junit,
    /// Result came from Cucumber JSON fallback
    Json,
    /// No test results; status inferred from ledger (Unknown)
    Inferred,
}

/// BDD scenario metadata.
///
/// Represents a scenario from a .feature file with its AC mapping.
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Scenario name (from `Scenario:` line)
    pub name: String,
    /// Associated AC ID (from `@AC-XXX-YYY` tag)
    pub ac_id: String,
    /// Relative path to the .feature file
    pub file: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac_status_icon_returns_correct_values() {
        assert_eq!(AcStatus::Pass.icon(), "[PASS]");
        assert_eq!(AcStatus::Fail.icon(), "[FAIL]");
        assert_eq!(AcStatus::Unknown.icon(), "[UNKNOWN]");
    }

    #[test]
    fn ac_status_name_returns_correct_values() {
        assert_eq!(AcStatus::Pass.name(), "pass");
        assert_eq!(AcStatus::Fail.name(), "fail");
        assert_eq!(AcStatus::Unknown.name(), "unknown");
    }

    #[test]
    fn ac_status_serializes_to_lowercase() {
        let pass = AcStatus::Pass;
        let json = serde_json::to_string(&pass).unwrap();
        assert_eq!(json, "\"pass\"");

        let fail = AcStatus::Fail;
        let json = serde_json::to_string(&fail).unwrap();
        assert_eq!(json, "\"fail\"");
    }

    #[test]
    fn ac_status_deserializes_from_lowercase() {
        let pass: AcStatus = serde_json::from_str("\"pass\"").unwrap();
        assert_eq!(pass, AcStatus::Pass);

        let fail: AcStatus = serde_json::from_str("\"fail\"").unwrap();
        assert_eq!(fail, AcStatus::Fail);
    }

    #[test]
    fn ac_source_serializes_to_lowercase() {
        let coverage = AcSource::Coverage;
        let json = serde_json::to_string(&coverage).unwrap();
        assert_eq!(json, "\"coverage\"");

        let inferred = AcSource::Inferred;
        let json = serde_json::to_string(&inferred).unwrap();
        assert_eq!(json, "\"inferred\"");
    }
}
