//! Gate receipt types for recording gate execution results.
//!
//! A gate receipt provides machine-generated proof that governance gates ran
//! and what they found. This is the core receipt type for CI/CD evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The core gate receipt: what gates ran and whether they passed.
///
/// This receipt is generated after running governance gates (fmt, clippy, tests, selftest)
/// and captures the results for evidence and audit purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// Unique identifier for this run (e.g., "2026-01-07T14:32Z-pr209").
    pub run_id: String,

    /// PR number, if this run is associated with a pull request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,

    /// Git commit SHA.
    pub commit: String,

    /// When the gate run started.
    pub started_at: DateTime<Utc>,

    /// When the gate run finished.
    pub finished_at: DateTime<Utc>,

    /// Results for each individual gate.
    pub gates: Vec<GateResult>,

    /// Overall status of the gate run.
    pub overall_status: GateStatus,

    /// Repository version (e.g., "v3.3.14").
    pub repo_version: String,

    /// Environment information for reproducibility.
    pub environment: Environment,
}

/// Result of a single gate execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateResult {
    /// Gate name (e.g., "fmt", "clippy", "tests", "selftest").
    pub name: String,

    /// Command that was executed.
    pub command: String,

    /// Gate execution status.
    pub status: GateStatus,

    /// Duration in milliseconds.
    pub duration_ms: u64,

    /// Additional details specific to the gate type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<GateDetails>,
}

/// Status of a gate execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateStatus {
    /// Gate passed successfully.
    Pass,
    /// Gate failed.
    Fail,
    /// Gate was skipped.
    Skipped,
}

/// Gate-specific details.
///
/// Different gate types report different details. This enum captures
/// the structured details for known gate types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GateDetails {
    /// Test execution details.
    Tests(TestDetails),
    /// Selftest execution details.
    Selftest(SelftestDetails),
    /// Generic key-value details for other gates.
    Generic(serde_json::Value),
}

/// Test execution details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestDetails {
    /// Number of tests that passed.
    pub passed: u32,
    /// Number of tests that failed.
    pub failed: u32,
    /// Number of tests that were ignored.
    #[serde(default)]
    pub ignored: u32,
}

/// Selftest execution details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelftestDetails {
    /// Number of selftest steps that passed.
    pub steps_passed: u32,
    /// Total number of selftest steps.
    pub steps_total: u32,
}

/// Environment information for the gate run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// Operating system (e.g., "linux", "darwin", "windows").
    pub os: String,

    /// Rust version (e.g., "1.83.0").
    pub rust_version: String,

    /// Whether the run was inside a Nix shell.
    #[serde(default)]
    pub nix_shell: bool,
}

impl GateReceipt {
    /// Check if all gates passed.
    pub fn all_passed(&self) -> bool {
        self.overall_status == GateStatus::Pass
    }

    /// Get the total duration in milliseconds.
    pub fn total_duration_ms(&self) -> u64 {
        self.gates.iter().map(|g| g.duration_ms).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_status_serde() {
        assert_eq!(serde_json::to_string(&GateStatus::Pass).unwrap(), r#""pass""#);
        assert_eq!(serde_json::to_string(&GateStatus::Fail).unwrap(), r#""fail""#);
        assert_eq!(serde_json::to_string(&GateStatus::Skipped).unwrap(), r#""skipped""#);

        assert_eq!(serde_json::from_str::<GateStatus>(r#""pass""#).unwrap(), GateStatus::Pass);
        assert_eq!(serde_json::from_str::<GateStatus>(r#""fail""#).unwrap(), GateStatus::Fail);
        assert_eq!(
            serde_json::from_str::<GateStatus>(r#""skipped""#).unwrap(),
            GateStatus::Skipped
        );
    }

    #[test]
    fn test_gate_receipt_roundtrip() {
        let receipt = GateReceipt {
            schema_version: "1.0".to_string(),
            run_id: "2026-01-07T14:32Z-pr209".to_string(),
            pr: Some(209),
            commit: "abc123def".to_string(),
            started_at: "2026-01-07T14:32:00Z".parse().unwrap(),
            finished_at: "2026-01-07T14:35:42Z".parse().unwrap(),
            gates: vec![
                GateResult {
                    name: "fmt".to_string(),
                    command: "cargo fmt --all --check".to_string(),
                    status: GateStatus::Pass,
                    duration_ms: 1234,
                    details: None,
                },
                GateResult {
                    name: "tests".to_string(),
                    command: "cargo test --all".to_string(),
                    status: GateStatus::Pass,
                    duration_ms: 23456,
                    details: Some(GateDetails::Tests(TestDetails {
                        passed: 142,
                        failed: 0,
                        ignored: 3,
                    })),
                },
            ],
            overall_status: GateStatus::Pass,
            repo_version: "v3.3.14".to_string(),
            environment: Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: true,
            },
        };

        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: GateReceipt = serde_json::from_str(&json).unwrap();

        assert_eq!(receipt, parsed);
    }

    #[test]
    fn test_total_duration() {
        let receipt = GateReceipt {
            schema_version: "1.0".to_string(),
            run_id: "test-run".to_string(),
            pr: None,
            commit: "abc123".to_string(),
            started_at: "2026-01-07T14:32:00Z".parse().unwrap(),
            finished_at: "2026-01-07T14:35:42Z".parse().unwrap(),
            gates: vec![
                GateResult {
                    name: "fmt".to_string(),
                    command: "cargo fmt".to_string(),
                    status: GateStatus::Pass,
                    duration_ms: 1000,
                    details: None,
                },
                GateResult {
                    name: "tests".to_string(),
                    command: "cargo test".to_string(),
                    status: GateStatus::Pass,
                    duration_ms: 2000,
                    details: None,
                },
            ],
            overall_status: GateStatus::Pass,
            repo_version: "v3.3.14".to_string(),
            environment: Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: false,
            },
        };

        assert_eq!(receipt.total_duration_ms(), 3000);
    }
}
