//! AC History: Time-series analysis of AC coverage snapshots.
//!
//! This module provides types and functions for analyzing AC coverage trends
//! over time from CI-generated snapshots.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Current schema version for ac-history JSON output.
///
/// Bump this when making breaking changes to the JSON structure.
/// - v1.0: Initial schema with snapshots, deltas, skipped_files
pub const AC_HISTORY_SCHEMA_VERSION: &str = "1.0";

// ===========================================================================
// Data types - mirror ac_status.rs JSON output (schema v2.0)
// ===========================================================================

/// Deserialized AC status snapshot (from ac-status --json).
#[derive(Debug, Deserialize)]
struct AcStatusSnapshot {
    schema_version: String,
    timestamp: String,
    must_have_acs: AcCategoryStats,
    optional_acs: AcCategoryStats,
    coverage_percent: f64,
    #[serde(default)]
    acs: Vec<AcJson>,
}

#[derive(Debug, Deserialize)]
struct AcCategoryStats {
    total: usize,
    passing: usize,
    failing: usize,
    unknown: usize,
}

#[derive(Debug, Deserialize)]
struct AcJson {
    id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    must_have_ac: bool,
}

// ===========================================================================
// Public types
// ===========================================================================

/// Aggregated metrics from a single snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotMetric {
    /// Commit SHA (extracted from filename)
    pub commit: String,
    /// Timestamp from JSON
    pub timestamp: String,

    // Must-have AC stats
    /// Total must-have ACs
    pub must_have_total: usize,
    /// Passing must-have ACs
    pub must_have_passing: usize,
    /// Failing must-have ACs
    pub must_have_failing: usize,
    /// Unknown must-have ACs
    pub must_have_unknown: usize,

    // Optional AC stats
    /// Total optional ACs
    pub optional_total: usize,
    /// Passing optional ACs
    pub optional_passing: usize,
    /// Failing optional ACs
    pub optional_failing: usize,
    /// Unknown optional ACs
    pub optional_unknown: usize,

    /// Overall coverage percentage
    pub coverage_percent: f64,

    /// List of failing must-have AC IDs (blockers)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kernel_blockers: Vec<String>,
}

/// History report containing all snapshots.
#[derive(Debug, Serialize)]
pub struct AcHistoryReport {
    /// Schema version for forward compatibility (bump on breaking changes)
    pub schema_version: String,
    /// Number of snapshots analyzed
    pub snapshot_count: usize,
    /// Date range (first timestamp to last)
    pub date_range: Option<(String, String)>,
    /// All snapshots sorted by timestamp
    pub snapshots: Vec<SnapshotMetric>,
    /// Delta analysis (new blockers between snapshots)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deltas: Vec<SnapshotDelta>,
    /// Files skipped due to incompatible schema or parse errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skipped_files: Vec<SkippedFile>,
}

/// Information about a skipped snapshot file.
#[derive(Debug, Clone, Serialize)]
pub struct SkippedFile {
    /// Filename that was skipped
    pub filename: String,
    /// Reason it was skipped
    pub reason: String,
}

/// Delta between consecutive snapshots.
#[derive(Debug, Serialize)]
pub struct SnapshotDelta {
    /// Commit SHA of the newer snapshot
    pub commit: String,
    /// New blockers introduced in this commit
    pub new_blockers: Vec<String>,
    /// Blockers resolved in this commit
    pub resolved_blockers: Vec<String>,
    /// Coverage change (percentage points)
    pub coverage_delta: f64,
}

/// Result of loading snapshots from a directory.
#[derive(Debug)]
pub struct LoadResult {
    /// Successfully parsed snapshots
    pub snapshots: Vec<SnapshotMetric>,
    /// Files that were skipped
    pub skipped: Vec<SkippedFile>,
}

// ===========================================================================
// Public API
// ===========================================================================

/// Extract commit SHA from filename like "ac-status-abcd1234.json".
pub fn extract_commit_from_filename(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    // Expected format: ac-status-<sha>
    stem.strip_prefix("ac-status-").map(|sha| sha.to_string())
}

/// Load all snapshots from a directory.
///
/// Reads all files matching `ac-status-*.json` pattern and parses them
/// as AC status snapshots.
pub fn load_snapshots(dir: &Path) -> Result<LoadResult> {
    if !dir.exists() {
        anyhow::bail!("Snapshot directory does not exist: {}", dir.display());
    }

    let mut snapshots = Vec::new();
    let mut skipped = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process files that match the expected naming pattern: ac-status-*.json
        if path.extension().is_some_and(|ext| ext == "json") {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            // Only attempt to parse files that match the ac-status-<sha>.json pattern
            if extract_commit_from_filename(&path).is_none() {
                // Not an ac-status file, silently ignore
                continue;
            }

            match parse_snapshot(&path) {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(e) => {
                    skipped.push(SkippedFile { filename, reason: e.to_string() });
                }
            }
        }
    }

    // Sort by timestamp
    snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(LoadResult { snapshots, skipped })
}

/// Build the history report from loaded snapshots.
pub fn build_report(
    snapshots: Vec<SnapshotMetric>,
    skipped_files: Vec<SkippedFile>,
) -> AcHistoryReport {
    let snapshot_count = snapshots.len();

    let date_range = if snapshot_count >= 2 {
        Some((snapshots[0].timestamp.clone(), snapshots[snapshot_count - 1].timestamp.clone()))
    } else if snapshot_count == 1 {
        Some((snapshots[0].timestamp.clone(), snapshots[0].timestamp.clone()))
    } else {
        None
    };

    let deltas = compute_deltas(&snapshots);

    AcHistoryReport {
        schema_version: AC_HISTORY_SCHEMA_VERSION.to_string(),
        snapshot_count,
        date_range,
        snapshots,
        deltas,
        skipped_files,
    }
}

// ===========================================================================
// Internal helpers
// ===========================================================================

/// Parse a single snapshot file.
fn parse_snapshot(path: &Path) -> Result<SnapshotMetric> {
    let commit = extract_commit_from_filename(path)
        .ok_or_else(|| anyhow::anyhow!("Cannot extract commit from filename: {:?}", path))?;

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snapshot: {}", path.display()))?;

    let snapshot: AcStatusSnapshot = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse snapshot JSON: {}", path.display()))?;

    if snapshot.schema_version != "2.0" {
        // Log warning but continue - we can still parse older schemas
        eprintln!(
            "[WARN] Snapshot {} has schema version '{}', expected '2.0'",
            path.display(),
            snapshot.schema_version
        );
    }

    // Extract kernel blockers (failing must_have_ac=true ACs)
    let kernel_blockers: Vec<String> = snapshot
        .acs
        .iter()
        .filter(|ac| ac.must_have_ac && ac.status == "fail")
        .map(|ac| ac.id.clone())
        .collect();

    Ok(SnapshotMetric {
        commit,
        timestamp: snapshot.timestamp,
        must_have_total: snapshot.must_have_acs.total,
        must_have_passing: snapshot.must_have_acs.passing,
        must_have_failing: snapshot.must_have_acs.failing,
        must_have_unknown: snapshot.must_have_acs.unknown,
        optional_total: snapshot.optional_acs.total,
        optional_passing: snapshot.optional_acs.passing,
        optional_failing: snapshot.optional_acs.failing,
        optional_unknown: snapshot.optional_acs.unknown,
        coverage_percent: snapshot.coverage_percent,
        kernel_blockers,
    })
}

/// Compute deltas between consecutive snapshots.
fn compute_deltas(snapshots: &[SnapshotMetric]) -> Vec<SnapshotDelta> {
    if snapshots.len() < 2 {
        return Vec::new();
    }

    let mut deltas = Vec::new();

    for window in snapshots.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        // Find new blockers
        let new_blockers: Vec<String> = curr
            .kernel_blockers
            .iter()
            .filter(|b| !prev.kernel_blockers.contains(b))
            .cloned()
            .collect();

        // Find resolved blockers
        let resolved_blockers: Vec<String> = prev
            .kernel_blockers
            .iter()
            .filter(|b| !curr.kernel_blockers.contains(b))
            .cloned()
            .collect();

        let coverage_delta = curr.coverage_percent - prev.coverage_percent;

        // Only include if there's something interesting
        if !new_blockers.is_empty() || !resolved_blockers.is_empty() || coverage_delta.abs() > 0.5 {
            deltas.push(SnapshotDelta {
                commit: curr.commit.clone(),
                new_blockers,
                resolved_blockers,
                coverage_delta,
            });
        }
    }

    deltas
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SAMPLE_SNAPSHOT_1: &str = r#"{
        "schema_version": "2.0",
        "timestamp": "2025-12-01T10:00:00Z",
        "must_have_acs": {"total": 10, "passing": 7, "failing": 2, "unknown": 1},
        "optional_acs": {"total": 5, "passing": 4, "failing": 0, "unknown": 1},
        "coverage_percent": 73.3,
        "acs": [
            {"id": "AC-KERN-001", "status": "pass", "must_have_ac": true},
            {"id": "AC-KERN-002", "status": "fail", "must_have_ac": true},
            {"id": "AC-KERN-003", "status": "fail", "must_have_ac": true},
            {"id": "AC-OPT-001", "status": "pass", "must_have_ac": false}
        ]
    }"#;

    const SAMPLE_SNAPSHOT_2: &str = r#"{
        "schema_version": "2.0",
        "timestamp": "2025-12-02T10:00:00Z",
        "must_have_acs": {"total": 10, "passing": 9, "failing": 0, "unknown": 1},
        "optional_acs": {"total": 5, "passing": 5, "failing": 0, "unknown": 0},
        "coverage_percent": 93.3,
        "acs": [
            {"id": "AC-KERN-001", "status": "pass", "must_have_ac": true},
            {"id": "AC-KERN-002", "status": "pass", "must_have_ac": true},
            {"id": "AC-KERN-003", "status": "pass", "must_have_ac": true},
            {"id": "AC-OPT-001", "status": "pass", "must_have_ac": false}
        ]
    }"#;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("ac-status-abc123.json"), SAMPLE_SNAPSHOT_1).unwrap();
        fs::write(dir.path().join("ac-status-def456.json"), SAMPLE_SNAPSHOT_2).unwrap();
        dir
    }

    #[test]
    fn extract_commit_from_valid_filename() {
        let path = std::path::PathBuf::from("ac-status-abc123.json");
        assert_eq!(extract_commit_from_filename(&path), Some("abc123".to_string()));
    }

    #[test]
    fn extract_commit_from_invalid_filename() {
        let path = std::path::PathBuf::from("not-a-snapshot.json");
        assert_eq!(extract_commit_from_filename(&path), None);
    }

    #[test]
    fn load_snapshots_from_directory() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();

        assert_eq!(result.snapshots.len(), 2);
        assert!(result.skipped.is_empty());
        // Should be sorted by timestamp
        assert_eq!(result.snapshots[0].timestamp, "2025-12-01T10:00:00Z");
        assert_eq!(result.snapshots[1].timestamp, "2025-12-02T10:00:00Z");
    }

    #[test]
    fn snapshot_extracts_kernel_blockers() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();

        // First snapshot has 2 failing kernel ACs
        assert_eq!(result.snapshots[0].kernel_blockers.len(), 2);
        assert!(result.snapshots[0].kernel_blockers.contains(&"AC-KERN-002".to_string()));
        assert!(result.snapshots[0].kernel_blockers.contains(&"AC-KERN-003".to_string()));

        // Second snapshot has no blockers
        assert!(result.snapshots[1].kernel_blockers.is_empty());
    }

    #[test]
    fn build_report_computes_date_range() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);

        assert_eq!(report.snapshot_count, 2);
        let (start, end) = report.date_range.unwrap();
        assert_eq!(start, "2025-12-01T10:00:00Z");
        assert_eq!(end, "2025-12-02T10:00:00Z");
    }

    #[test]
    fn build_report_computes_deltas() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);

        // Should have delta showing resolved blockers
        assert_eq!(report.deltas.len(), 1);
        let delta = &report.deltas[0];
        assert_eq!(delta.commit, "def456");
        assert!(delta.new_blockers.is_empty());
        assert_eq!(delta.resolved_blockers.len(), 2);
    }

    #[test]
    fn empty_directory_returns_empty_snapshots() {
        let dir = TempDir::new().unwrap();
        let result = load_snapshots(dir.path()).unwrap();
        assert!(result.snapshots.is_empty());
    }

    #[test]
    fn nonexistent_directory_returns_error() {
        let result = load_snapshots(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    /// Shape lock test for ac-history JSON output.
    #[test]
    fn ac_history_json_shape_is_stable() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);

        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Required top-level fields
        let required_top_level = ["schema_version", "snapshot_count", "date_range", "snapshots"];
        for field in required_top_level {
            assert!(parsed.get(field).is_some(), "Missing required top-level field: {}", field);
        }

        // Schema version must be "1.0"
        assert_eq!(parsed["schema_version"].as_str().unwrap(), "1.0");

        // Snapshot fields
        let snapshot = &parsed["snapshots"][0];
        let required_snapshot_fields =
            ["commit", "timestamp", "must_have_total", "must_have_passing", "coverage_percent"];
        for field in required_snapshot_fields {
            assert!(snapshot.get(field).is_some(), "Missing required snapshot field: {}", field);
        }
    }
}
