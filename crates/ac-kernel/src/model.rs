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

// ============================================================================
// AcEvidence: Unified Evidence Model (ADR-0024)
// ============================================================================

/// Unified evidence model for an Acceptance Criterion.
///
/// Combines spec-derived mappings with runtime test results to provide
/// a complete picture of AC coverage for governance gates.
///
/// # Evidence Sources
///
/// - **Spec-derived**: Counts from `spec_ledger.yaml` test mappings
///   - `unit_mapped`: Number of unit tests declared
///   - `bdd_mapped`: Number of BDD/integration tests declared
///
/// - **Runtime**: Results from actual test execution
///   - `bdd_passed`: Count of BDD scenarios that passed
///   - `bdd_failed`: Count of BDD scenarios that failed
///
/// # Status Derivation (ADR-0024)
///
/// The `status()` method computes AC status from evidence:
/// 1. **FAIL** if `bdd_failed > 0`
/// 2. **PASS** if `bdd_passed > 0 OR unit_mapped > 0`
/// 3. **UNKNOWN** otherwise
///
/// Note: Unit tests are "presumed to run" because `cargo xtask check`
/// (which runs in selftest) executes all unit tests. If a unit test
/// is mapped, it will have been executed before selftest completes.
///
/// See ADR-0024 for formal semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcEvidence {
    /// The AC ID (e.g., "AC-KERN-001")
    pub ac_id: String,

    /// Whether this AC is a kernel AC (must_have_ac=true)
    pub is_kernel: bool,

    // ---- Spec-derived (from spec_ledger.yaml) ----
    /// Number of unit tests declared in spec_ledger.yaml for this AC
    pub unit_mapped: usize,

    /// Number of BDD/integration tests declared for this AC
    pub bdd_mapped: usize,

    // ---- Runtime evidence (from coverage.jsonl) ----
    /// Number of BDD scenarios that passed for this AC
    pub bdd_passed: usize,

    /// Number of BDD scenarios that failed for this AC
    pub bdd_failed: usize,
}

impl AcEvidence {
    /// Create a new AcEvidence with all counts at zero.
    pub fn new(ac_id: impl Into<String>, is_kernel: bool) -> Self {
        Self {
            ac_id: ac_id.into(),
            is_kernel,
            unit_mapped: 0,
            bdd_mapped: 0,
            bdd_passed: 0,
            bdd_failed: 0,
        }
    }

    /// Compute the AC status from the collected evidence.
    ///
    /// # Status Rules (ADR-0024)
    ///
    /// 1. **FAIL**: Any BDD scenario failed (`bdd_failed > 0`)
    /// 2. **PASS**: At least one BDD passed OR unit tests are mapped
    ///    (unit tests are presumed to run in selftest's `cargo xtask check`)
    /// 3. **UNKNOWN**: No BDD results and no unit mappings
    pub fn status(&self) -> AcStatus {
        if self.bdd_failed > 0 {
            AcStatus::Fail
        } else if self.bdd_passed > 0 || self.unit_mapped > 0 {
            AcStatus::Pass
        } else {
            AcStatus::Unknown
        }
    }

    /// Check if this AC has any evidence of coverage.
    pub fn has_evidence(&self) -> bool {
        self.bdd_passed > 0 || self.bdd_failed > 0 || self.unit_mapped > 0 || self.bdd_mapped > 0
    }

    /// Check if this AC has runtime verification (not just spec mappings).
    pub fn has_runtime_verification(&self) -> bool {
        self.bdd_passed > 0 || self.bdd_failed > 0
    }

    /// Total tests mapped in spec (BDD + unit).
    pub fn tests_mapped(&self) -> usize {
        self.unit_mapped + self.bdd_mapped
    }

    /// Total BDD tests executed (passed + failed).
    pub fn bdd_executed(&self) -> usize {
        self.bdd_passed + self.bdd_failed
    }
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

    // ---- AcEvidence tests ----

    #[test]
    fn evidence_status_fail_when_bdd_failed() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.bdd_passed = 5;
        ev.bdd_failed = 1;
        assert_eq!(ev.status(), AcStatus::Fail);
    }

    #[test]
    fn evidence_status_pass_when_bdd_passed() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.bdd_passed = 1;
        assert_eq!(ev.status(), AcStatus::Pass);
    }

    #[test]
    fn evidence_status_pass_when_unit_mapped() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.unit_mapped = 1;
        assert_eq!(ev.status(), AcStatus::Pass);
    }

    #[test]
    fn evidence_status_unknown_when_no_evidence() {
        let ev = AcEvidence::new("AC-TEST", true);
        assert_eq!(ev.status(), AcStatus::Unknown);
    }

    #[test]
    fn evidence_status_unknown_when_only_bdd_mapped() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.bdd_mapped = 3; // Mapped but not executed
        assert_eq!(ev.status(), AcStatus::Unknown);
    }

    #[test]
    fn evidence_has_evidence_true_when_any_count() {
        let mut ev = AcEvidence::new("AC-TEST", false);
        assert!(!ev.has_evidence());

        ev.unit_mapped = 1;
        assert!(ev.has_evidence());

        ev = AcEvidence::new("AC-TEST", false);
        ev.bdd_passed = 1;
        assert!(ev.has_evidence());
    }

    #[test]
    fn evidence_has_runtime_verification() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.unit_mapped = 1;
        assert!(!ev.has_runtime_verification()); // Unit mapping is not runtime

        ev.bdd_passed = 1;
        assert!(ev.has_runtime_verification());
    }

    #[test]
    fn evidence_tests_mapped_sums_correctly() {
        let mut ev = AcEvidence::new("AC-TEST", true);
        ev.unit_mapped = 2;
        ev.bdd_mapped = 3;
        assert_eq!(ev.tests_mapped(), 5);
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
