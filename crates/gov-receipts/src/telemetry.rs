//! Telemetry receipt types for tracking probe execution.
//!
//! The telemetry receipt captures normalized hard probe outputs - tool
//! measurements, change surface, and verification coverage.

use crate::meta::ReceiptMeta;
use serde::{Deserialize, Serialize};

/// Telemetry receipt for tracking probe execution.
///
/// This receipt captures change surface, contract changes, safety metrics,
/// structural analysis, and individual probe results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// PR number, if this receipt is associated with a pull request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,

    /// Run ID for correlation with other receipts.
    pub run_id: String,

    /// Probe profile used for this run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProbeProfile>,

    /// Change surface from git diff analysis.
    pub change_surface: ChangeSurface,

    /// Contract surface change detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<Contracts>,

    /// Unsafe code and safety-related metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<Safety>,

    /// Structural analysis (cycles, coupling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure: Option<Structure>,

    /// Test and verification artifact metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<TelemetryVerification>,

    /// List of probes that were run.
    #[serde(default)]
    pub probes: Vec<ProbeResult>,

    /// Explicit list of probes that were skipped with reasons.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub not_run: Vec<SkippedProbe>,

    /// Meta provenance for re-analysis and method versioning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ReceiptMeta>,
}

/// Probe profile determining which probes run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeProfile {
    /// Quick CI checks.
    Fast,
    /// Comprehensive analysis.
    Full,
    /// Forensic/deep dive.
    Exhibit,
}

/// Status of a probe execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    /// Probe ran successfully.
    Run,
    /// Probe was not run.
    NotRun,
    /// Probe encountered an error.
    Error,
}

/// Quantified change surface from git diff analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeSurface {
    /// Number of files changed.
    #[serde(default)]
    pub files_changed: u32,

    /// Number of lines inserted.
    #[serde(default)]
    pub insertions: u32,

    /// Number of lines deleted.
    #[serde(default)]
    pub deletions: u32,

    /// Files with concentrated changes or high churn.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hotspots: Vec<String>,

    /// Logical modules affected by the change.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules_touched: Vec<String>,

    /// Cargo crates affected by the change.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crates_touched: Vec<String>,
}

/// Contract surface change detection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contracts {
    /// Whether any schema files were modified.
    #[serde(default)]
    pub schema_changed: bool,

    /// Whether public API surface changed.
    #[serde(default)]
    pub public_api_changed: bool,

    /// Whether CLI interface changed.
    #[serde(default)]
    pub cli_changed: bool,

    /// Whether any contract changes are breaking.
    #[serde(default)]
    pub breaking: bool,

    /// Pointers to specific diffs showing contract changes.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diff_pointers: Vec<String>,
}

/// Unsafe code and safety-related metrics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Safety {
    /// Number of unsafe blocks/functions added.
    #[serde(default)]
    pub unsafe_added: u32,

    /// Number of unsafe blocks/functions removed.
    #[serde(default)]
    pub unsafe_removed: u32,

    /// Summary from cargo-geiger analysis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geiger_summary: Option<GeigerSummary>,

    /// Pointers to specific unsafe code locations.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pointers: Vec<String>,
}

/// Summary of cargo-geiger unsafe code analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeigerSummary {
    /// Count of used unsafe code regions.
    #[serde(default)]
    pub used_unsafe: u32,

    /// Count of unused unsafe code regions.
    #[serde(default)]
    pub unused_unsafe: u32,

    /// Whether #![forbid(unsafe_code)] is set.
    #[serde(default)]
    pub forbid_unsafe: bool,
}

/// Structural analysis from dependency graphs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    /// Whether any dependency cycles were detected.
    #[serde(default)]
    pub cycles_detected: bool,

    /// Net change in dependency edges.
    #[serde(default)]
    pub dependency_edges_delta: i32,

    /// Pointers to structural analysis artifacts.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pointers: Vec<String>,
}

/// Test and verification artifact metrics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryVerification {
    /// Number of test functions added.
    #[serde(default)]
    pub tests_added: u32,

    /// Number of test functions modified.
    #[serde(default)]
    pub tests_modified: u32,

    /// Path to coverage report artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_report_path: Option<String>,

    /// Path to mutation testing outcomes artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_outcomes_path: Option<String>,
}

/// Result of a single probe execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe name (e.g., 'cargo-geiger', 'cargo-deny', 'coverage').
    pub name: String,

    /// Version of the probe tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Probe execution status.
    pub status: ProbeStatus,

    /// Explanation when status is not_run or error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Probe execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Path to detailed probe output artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
}

/// Record of a probe that was skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedProbe {
    /// Name of the skipped probe.
    pub probe: String,

    /// Why the probe was skipped.
    pub reason: String,
}

impl TelemetryReceipt {
    /// Create a new telemetry receipt builder.
    pub fn builder() -> TelemetryReceiptBuilder {
        TelemetryReceiptBuilder::default()
    }

    /// Get all probes that ran successfully.
    pub fn successful_probes(&self) -> impl Iterator<Item = &ProbeResult> {
        self.probes.iter().filter(|p| p.status == ProbeStatus::Run)
    }

    /// Get all probes that failed or had errors.
    pub fn failed_probes(&self) -> impl Iterator<Item = &ProbeResult> {
        self.probes.iter().filter(|p| p.status == ProbeStatus::Error)
    }
}

/// Builder for constructing `TelemetryReceipt` instances.
#[derive(Debug, Default)]
pub struct TelemetryReceiptBuilder {
    schema_version: Option<String>,
    pr: Option<u64>,
    run_id: Option<String>,
    profile: Option<ProbeProfile>,
    change_surface: ChangeSurface,
    contracts: Option<Contracts>,
    safety: Option<Safety>,
    structure: Option<Structure>,
    verification: Option<TelemetryVerification>,
    probes: Vec<ProbeResult>,
    not_run: Vec<SkippedProbe>,
    meta: Option<ReceiptMeta>,
}

impl TelemetryReceiptBuilder {
    /// Set the schema version.
    pub fn schema_version(mut self, version: impl Into<String>) -> Self {
        self.schema_version = Some(version.into());
        self
    }

    /// Set the PR number.
    pub fn pr(mut self, pr: u64) -> Self {
        self.pr = Some(pr);
        self
    }

    /// Set the run ID.
    pub fn run_id(mut self, id: impl Into<String>) -> Self {
        self.run_id = Some(id.into());
        self
    }

    /// Set the probe profile.
    pub fn profile(mut self, profile: ProbeProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Set the change surface.
    pub fn change_surface(mut self, surface: ChangeSurface) -> Self {
        self.change_surface = surface;
        self
    }

    /// Set the contracts.
    pub fn contracts(mut self, contracts: Contracts) -> Self {
        self.contracts = Some(contracts);
        self
    }

    /// Set the safety metrics.
    pub fn safety(mut self, safety: Safety) -> Self {
        self.safety = Some(safety);
        self
    }

    /// Set the structural analysis.
    pub fn structure(mut self, structure: Structure) -> Self {
        self.structure = Some(structure);
        self
    }

    /// Set the verification metrics.
    pub fn verification(mut self, verification: TelemetryVerification) -> Self {
        self.verification = Some(verification);
        self
    }

    /// Add a probe result.
    pub fn probe(mut self, probe: ProbeResult) -> Self {
        self.probes.push(probe);
        self
    }

    /// Set all probe results.
    pub fn probes(mut self, probes: Vec<ProbeResult>) -> Self {
        self.probes = probes;
        self
    }

    /// Add a skipped probe.
    pub fn skipped(mut self, skipped: SkippedProbe) -> Self {
        self.not_run.push(skipped);
        self
    }

    /// Set the meta provenance.
    pub fn meta(mut self, meta: ReceiptMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Build the telemetry receipt.
    ///
    /// # Panics
    ///
    /// Panics if run_id is not set.
    pub fn build(self) -> TelemetryReceipt {
        TelemetryReceipt {
            schema_version: self.schema_version.unwrap_or_else(|| "1.0".to_string()),
            pr: self.pr,
            run_id: self.run_id.expect("run_id is required"),
            profile: self.profile,
            change_surface: self.change_surface,
            contracts: self.contracts,
            safety: self.safety,
            structure: self.structure,
            verification: self.verification,
            probes: self.probes,
            not_run: self.not_run,
            meta: self.meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_receipt_roundtrip() {
        let receipt = TelemetryReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: "test-run".to_string(),
            profile: Some(ProbeProfile::Full),
            change_surface: ChangeSurface {
                files_changed: 5,
                insertions: 100,
                deletions: 50,
                hotspots: vec!["lib.rs".to_string()],
                ..Default::default()
            },
            contracts: Some(Contracts { schema_changed: true, ..Default::default() }),
            safety: None,
            structure: None,
            verification: None,
            probes: vec![ProbeResult {
                name: "cargo-clippy".to_string(),
                version: Some("0.1.0".to_string()),
                status: ProbeStatus::Run,
                reason: None,
                duration_ms: Some(1234),
                artifact_path: None,
            }],
            not_run: vec![],
            meta: None,
        };

        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: TelemetryReceipt = serde_json::from_str(&json).unwrap();

        assert_eq!(receipt, parsed);
    }

    #[test]
    fn test_telemetry_receipt_builder() {
        let receipt = TelemetryReceipt::builder()
            .run_id("test-run")
            .pr(123)
            .profile(ProbeProfile::Fast)
            .change_surface(ChangeSurface {
                files_changed: 3,
                insertions: 50,
                ..Default::default()
            })
            .probe(ProbeResult {
                name: "fmt".to_string(),
                version: None,
                status: ProbeStatus::Run,
                reason: None,
                duration_ms: Some(500),
                artifact_path: None,
            })
            .build();

        assert_eq!(receipt.pr, Some(123));
        assert_eq!(receipt.profile, Some(ProbeProfile::Fast));
        assert_eq!(receipt.change_surface.files_changed, 3);
        assert_eq!(receipt.probes.len(), 1);
    }

    #[test]
    fn test_probe_profile_serde() {
        assert_eq!(serde_json::to_string(&ProbeProfile::Fast).unwrap(), r#""fast""#);
        assert_eq!(serde_json::to_string(&ProbeProfile::Full).unwrap(), r#""full""#);
        assert_eq!(serde_json::to_string(&ProbeProfile::Exhibit).unwrap(), r#""exhibit""#);
    }

    #[test]
    fn test_probe_status_serde() {
        assert_eq!(serde_json::to_string(&ProbeStatus::Run).unwrap(), r#""run""#);
        assert_eq!(serde_json::to_string(&ProbeStatus::NotRun).unwrap(), r#""not_run""#);
        assert_eq!(serde_json::to_string(&ProbeStatus::Error).unwrap(), r#""error""#);
    }

    #[test]
    fn test_successful_and_failed_probes() {
        let receipt = TelemetryReceipt::builder()
            .run_id("test")
            .probe(ProbeResult {
                name: "fmt".to_string(),
                version: None,
                status: ProbeStatus::Run,
                reason: None,
                duration_ms: None,
                artifact_path: None,
            })
            .probe(ProbeResult {
                name: "clippy".to_string(),
                version: None,
                status: ProbeStatus::Error,
                reason: Some("timeout".to_string()),
                duration_ms: None,
                artifact_path: None,
            })
            .build();

        assert_eq!(receipt.successful_probes().count(), 1);
        assert_eq!(receipt.failed_probes().count(), 1);
    }
}
