//! AC Kernel: Core governance logic for the Rust-as-Spec platform.
//!
//! This crate contains the shared data model and logic for:
//! - AC (Acceptance Criteria) status tracking
//! - Coverage parsing and aggregation
//! - Ledger reading and AC metadata extraction
//! - History/trend analysis
//!
//! # Architecture
//!
//! The `ac-kernel` crate is designed to be the single source of truth for AC-related
//! types and logic. It is used by:
//!
//! - `xtask`: For `ac-status`, `ac-coverage`, `ac-history` commands
//! - `acceptance`: For writing coverage records during BDD test execution
//!
//! # Modules
//!
//! - [`model`]: Core data types (AcStatus, AcSource, Scenario, etc.)
//! - [`coverage`]: AC coverage JSONL parsing
//! - [`ledger`]: spec_ledger.yaml parsing
//! - [`history`]: Time-series snapshot analysis
//! - [`json`]: JSON output schemas for ac-status
//! - [`layout`]: Path conventions for AC artifacts
//!
//! # Quick Start
//!
//! For most use cases, you can use the [`AcKernel`] facade:
//!
//! ```no_run
//! use ac_kernel::{AcKernel, SpecLayout};
//! use std::path::Path;
//!
//! let layout = SpecLayout::for_repo_root(Path::new("."));
//! let kernel = AcKernel::new(layout);
//!
//! // Load AC status from ledger + coverage
//! let acs = kernel.load_status().unwrap();
//!
//! // Or get JSON-ready output
//! let json = kernel.load_status_json().unwrap();
//!
//! // Or load historical trends
//! let history = kernel.load_history().unwrap();
//! ```

pub mod coverage;
pub mod history;
pub mod json;
pub mod layout;
pub mod ledger;
pub mod model;

use std::collections::HashMap;

// Re-export commonly used types at the crate root
pub use coverage::{AcCoverageRecord, parse_ac_coverage};
pub use history::{
    AC_HISTORY_SCHEMA_VERSION, AcHistoryReport, LoadResult, SkippedFile, SnapshotDelta,
    SnapshotMetric, build_report, load_snapshots,
};
pub use json::{
    AC_STATUS_SCHEMA_VERSION, Ac, AcCategoryStats, AcJson, AcStatusJson, build_status_json,
};
pub use layout::{SpecLayout, SpecLayoutBuilder};
pub use ledger::{AcDetails, AcMetadata, get_ac_details, parse_ledger_with_metadata};
pub use model::{AcEvidence, AcSource, AcStatus, Scenario, TestMapping};

// ============================================================================
// AcKernel Facade
// ============================================================================

/// High-level facade for AC governance operations.
///
/// `AcKernel` provides a unified API for loading and querying AC status,
/// coverage, and historical trends. It handles the details of parsing
/// ledger files, coverage results, and snapshot files.
///
/// # Example
///
/// ```no_run
/// use ac_kernel::{AcKernel, SpecLayout};
/// use std::path::Path;
///
/// let layout = SpecLayout::for_repo_root(Path::new("/path/to/repo"));
/// let kernel = AcKernel::new(layout);
///
/// // Get current AC status
/// let acs = kernel.load_status()?;
/// for (id, ac) in &acs {
///     println!("{}: {:?}", id, ac.status);
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct AcKernel {
    layout: SpecLayout,
}

impl AcKernel {
    /// Create a new `AcKernel` with the given layout.
    pub fn new(layout: SpecLayout) -> Self {
        Self { layout }
    }

    /// Get a reference to the underlying layout.
    pub fn layout(&self) -> &SpecLayout {
        &self.layout
    }

    /// Load AC status from the ledger and coverage files.
    ///
    /// This is the main entry point for getting current AC status:
    /// 1. Parses the spec ledger to get all ACs with metadata
    /// 2. Parses the coverage.jsonl to get BDD test results
    /// 3. Merges them into a unified view with computed status
    ///
    /// # Errors
    ///
    /// Returns an error if the ledger cannot be read or parsed.
    /// Missing coverage files are handled gracefully (ACs will have Unknown status).
    pub fn load_status(&self) -> anyhow::Result<HashMap<String, Ac>> {
        // Load metadata from ledger
        let meta = ledger::parse_ledger_with_metadata(&self.layout.ledger)?;

        // Build initial AC index from ledger metadata
        let mut acs = build_ac_index(&meta);

        // Load BDD coverage results
        let (scenarios, bdd_results) = coverage::parse_ac_coverage(&self.layout.coverage_file)?;

        // Update AC statuses from coverage
        update_ac_statuses(&mut acs, &scenarios, &bdd_results, AcSource::Coverage);

        Ok(acs)
    }

    /// Load AC status and format as JSON output.
    ///
    /// This is a convenience method that calls `load_status()` and then
    /// converts the result to the JSON schema used by `ac-status --json`.
    pub fn load_status_json(&self) -> anyhow::Result<AcStatusJson> {
        let acs = self.load_status()?;
        Ok(json::build_status_json(&acs))
    }

    /// Load historical AC coverage trends.
    ///
    /// Reads snapshot files from the history directory and computes
    /// trends and deltas between snapshots.
    ///
    /// # Errors
    ///
    /// Returns an error if the history directory doesn't exist.
    pub fn load_history(&self) -> anyhow::Result<AcHistoryReport> {
        let result = history::load_snapshots(&self.layout.history_dir)?;
        Ok(history::build_report(result.snapshots, result.skipped))
    }

    /// Check if the essential files exist for AC status computation.
    ///
    /// Returns true if at least the ledger exists. Coverage is optional
    /// (ACs will have Unknown status without it).
    pub fn is_ready(&self) -> bool {
        self.layout.has_ledger()
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Build an AC index from ledger metadata.
///
/// Creates an initial `Ac` for each entry in the metadata map,
/// with status set to Unknown and source set to Inferred.
fn build_ac_index(meta: &HashMap<String, AcMetadata>) -> HashMap<String, Ac> {
    meta.iter()
        .map(|(id, m)| {
            let tests_total = m.tests.iter().filter(|t| is_automated_test(t)).count();
            (
                id.clone(),
                Ac {
                    id: id.clone(),
                    story_id: m.story_id.clone(),
                    req_id: m.req_id.clone(),
                    text: m.text.clone(),
                    status: AcStatus::Unknown,
                    source: AcSource::Inferred,
                    scenarios: Vec::new(),
                    tests: m.tests.clone(),
                    tests_total,
                    tests_executed: 0,
                    tags: m.tags.clone(),
                    must_have_ac: m.must_have_ac,
                },
            )
        })
        .collect()
}

/// Check if a test mapping represents an automated test.
fn is_automated_test(test: &TestMapping) -> bool {
    matches!(test.test_type.to_lowercase().as_str(), "unit" | "integration" | "bdd")
}

/// Update AC statuses from coverage results.
///
/// Merges BDD test results into the AC index, updating status and source
/// for each AC that has coverage.
fn update_ac_statuses(
    acs: &mut HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    bdd_results: &HashMap<String, AcStatus>,
    source: AcSource,
) {
    // Add scenario names to their respective ACs
    for scenario in scenarios.values() {
        if let Some(ac) = acs.get_mut(&scenario.ac_id) {
            ac.scenarios.push(scenario.name.clone());
        }
    }

    // Update status from BDD results
    for (ac_id, status) in bdd_results {
        if let Some(ac) = acs.get_mut(ac_id) {
            ac.status = *status;
            ac.source = source;
            ac.tests_executed = ac.scenarios.len();
        }
    }
}

// ============================================================================
// AcEvidence Builder (ADR-0024)
// ============================================================================

/// Build evidence map from ledger metadata and BDD coverage.
///
/// This is the primary entry point for computing AC evidence from
/// all available sources:
/// 1. Parse spec_ledger.yaml for test mappings (unit_mapped, bdd_mapped)
/// 2. Parse coverage.jsonl for BDD results (bdd_passed, bdd_failed)
/// 3. Merge into unified AcEvidence structs
///
/// # Arguments
///
/// * `layout` - Path layout for finding spec_ledger.yaml and coverage.jsonl
///
/// # Returns
///
/// HashMap<AC_ID, AcEvidence> with computed evidence for each AC.
///
/// # Errors
///
/// Returns an error if the ledger cannot be read. Missing coverage is handled
/// gracefully (all BDD counts will be zero).
pub fn build_evidence(layout: &SpecLayout) -> anyhow::Result<HashMap<String, AcEvidence>> {
    // Load metadata from ledger
    let meta = ledger::parse_ledger_with_metadata(&layout.ledger)?;

    // Load BDD coverage (gracefully handles missing file)
    let (scenarios, bdd_results) = coverage::parse_ac_coverage(&layout.coverage_file)?;

    // Build evidence map from spec
    let mut evidence: HashMap<String, AcEvidence> = HashMap::new();

    for (ac_id, m) in &meta {
        let mut ev = AcEvidence::new(ac_id.clone(), m.must_have_ac);

        // Count spec mappings by type
        for test in &m.tests {
            match test.test_type.to_lowercase().as_str() {
                "unit" => ev.unit_mapped += 1,
                "bdd" | "integration" => ev.bdd_mapped += 1,
                _ => {} // Ignore ci, manual, docs, etc.
            }
        }

        evidence.insert(ac_id.clone(), ev);
    }

    // Merge BDD coverage results
    // Note: bdd_results is aggregated by AC ID
    for (ac_id, status) in &bdd_results {
        if let Some(ev) = evidence.get_mut(ac_id) {
            match status {
                AcStatus::Pass => ev.bdd_passed += 1,
                AcStatus::Fail => ev.bdd_failed += 1,
                AcStatus::Unknown => {}
            }
        }
    }

    // Also count scenarios for more granular evidence
    // (multiple scenarios per AC possible)
    let mut scenario_counts: HashMap<String, (usize, usize)> = HashMap::new(); // (passed, failed)
    for scenario in scenarios.values() {
        if let Some(status) = bdd_results.get(&scenario.ac_id) {
            let entry = scenario_counts.entry(scenario.ac_id.clone()).or_default();
            match status {
                AcStatus::Pass => entry.0 += 1,
                AcStatus::Fail => entry.1 += 1,
                AcStatus::Unknown => {}
            }
        }
    }

    // Use scenario counts if more accurate than aggregated results
    for (ac_id, (passed, failed)) in scenario_counts {
        if let Some(ev) = evidence.get_mut(&ac_id) {
            // Scenario counts are more granular
            if passed > ev.bdd_passed {
                ev.bdd_passed = passed;
            }
            if failed > ev.bdd_failed {
                ev.bdd_failed = failed;
            }
        }
    }

    Ok(evidence)
}

/// Compute kernel coverage summary from evidence.
///
/// Returns a tuple of (total, passing, failing, unknown) counts for kernel ACs only.
///
/// # Arguments
///
/// * `evidence` - Evidence map from `build_evidence()`
///
/// # Returns
///
/// `(total_kernel, passing_kernel, failing_kernel, unknown_kernel)`
pub fn kernel_coverage_summary(
    evidence: &HashMap<String, AcEvidence>,
) -> (usize, usize, usize, usize) {
    let kernel_evidence: Vec<_> = evidence.values().filter(|e| e.is_kernel).collect();

    let total = kernel_evidence.len();
    let passing = kernel_evidence.iter().filter(|e| e.status() == AcStatus::Pass).count();
    let failing = kernel_evidence.iter().filter(|e| e.status() == AcStatus::Fail).count();
    let unknown = kernel_evidence.iter().filter(|e| e.status() == AcStatus::Unknown).count();

    (total, passing, failing, unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create directory structure
        std::fs::create_dir_all(dir.path().join("specs")).unwrap();
        std::fs::create_dir_all(dir.path().join("target/ac")).unwrap();
        std::fs::create_dir_all(dir.path().join("artifacts/ac-status")).unwrap();

        // Write a simple ledger with test mappings
        let ledger = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        title: Test Requirement
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC 1"
            tests:
              - type: bdd
                tag: "@AC-TEST-001"
          - id: AC-TEST-002
            text: "Test AC 2"
            tests:
              - type: bdd
                tag: "@AC-TEST-002"
"#;
        std::fs::write(dir.path().join("specs/spec_ledger.yaml"), ledger).unwrap();

        // Write coverage file
        let coverage = r#"{"ac_id":"AC-TEST-001","status":"passed","feature":"test.feature","scenario":"Test scenario","tags":["AC-TEST-001"]}
"#;
        std::fs::write(dir.path().join("target/ac/coverage.jsonl"), coverage).unwrap();

        dir
    }

    #[test]
    fn ac_kernel_loads_status() {
        let dir = create_test_repo();
        let layout = SpecLayout::for_repo_root(dir.path());
        let kernel = AcKernel::new(layout);

        let acs = kernel.load_status().unwrap();

        assert_eq!(acs.len(), 2);
        assert_eq!(acs.get("AC-TEST-001").unwrap().status, AcStatus::Pass);
        assert_eq!(acs.get("AC-TEST-002").unwrap().status, AcStatus::Unknown);
    }

    #[test]
    fn ac_kernel_loads_status_json() {
        let dir = create_test_repo();
        let layout = SpecLayout::for_repo_root(dir.path());
        let kernel = AcKernel::new(layout);

        let json = kernel.load_status_json().unwrap();

        assert_eq!(json.schema_version, AC_STATUS_SCHEMA_VERSION);
        assert_eq!(json.acs.len(), 2);
    }

    #[test]
    fn ac_kernel_is_ready_checks_ledger() {
        let dir = create_test_repo();
        let layout = SpecLayout::for_repo_root(dir.path());
        let kernel = AcKernel::new(layout);

        assert!(kernel.is_ready());

        // Nonexistent repo should not be ready
        let bad_layout = SpecLayout::for_repo_root(std::path::Path::new("/nonexistent"));
        let bad_kernel = AcKernel::new(bad_layout);
        assert!(!bad_kernel.is_ready());
    }

    #[test]
    fn ac_kernel_handles_missing_coverage() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("specs")).unwrap();

        let ledger = r#"
stories:
  - id: US-TEST-001
    requirements:
      - id: REQ-TEST-001
        acceptance_criteria:
          - id: AC-TEST-001
            text: "Test AC"
"#;
        std::fs::write(dir.path().join("specs/spec_ledger.yaml"), ledger).unwrap();

        let layout = SpecLayout::for_repo_root(dir.path());
        let kernel = AcKernel::new(layout);

        let acs = kernel.load_status().unwrap();

        assert_eq!(acs.len(), 1);
        assert_eq!(acs.get("AC-TEST-001").unwrap().status, AcStatus::Unknown);
    }

    #[test]
    fn ac_kernel_load_history_works() {
        let dir = create_test_repo();

        // Add a snapshot file
        let snapshot = r#"{
            "schema_version": "2.0",
            "timestamp": "2025-12-01T10:00:00Z",
            "must_have_acs": {"total": 2, "passing": 1, "failing": 0, "unknown": 1},
            "optional_acs": {"total": 0, "passing": 0, "failing": 0, "unknown": 0},
            "coverage_percent": 50.0,
            "acs": [
                {"id": "AC-TEST-001", "status": "pass", "must_have_ac": true}
            ]
        }"#;
        std::fs::write(dir.path().join("artifacts/ac-status/ac-status-abc123.json"), snapshot)
            .unwrap();

        let layout = SpecLayout::for_repo_root(dir.path());
        let kernel = AcKernel::new(layout);

        let history = kernel.load_history().unwrap();

        assert_eq!(history.snapshot_count, 1);
        assert_eq!(history.snapshots[0].commit, "abc123");
    }
}
