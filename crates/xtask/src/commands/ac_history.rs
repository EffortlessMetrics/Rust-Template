//! AC History: Time-series analysis of AC coverage snapshots
//!
//! This command aggregates AC status snapshots (from CI) to show governance trends over time.
//!
//! # Design
//!
//! Each CI run generates `ac-status --json` and stores it as an artifact with the commit SHA
//! in the filename. This command reads a directory of those snapshots and produces trend analysis.
//!
//! # Usage
//!
//! ```bash
//! # Summarize history from downloaded CI artifacts
//! cargo xtask ac-history --dir ./artifacts/ac-status
//!
//! # Export as CSV for charting
//! cargo xtask ac-history --dir ./artifacts/ac-status --format csv
//!
//! # Focus on kernel ACs only
//! cargo xtask ac-history --dir ./artifacts/ac-status --must-have
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Arguments for ac-history command
#[derive(Debug, Clone)]
pub struct AcHistoryArgs {
    /// Directory containing ac-status JSON snapshots
    pub dir: PathBuf,
    /// Output format (text, markdown, csv, json)
    pub format: String,
    /// Only show must_have_ac=true ACs
    pub must_have: bool,
}

impl Default for AcHistoryArgs {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("artifacts/ac-status"),
            format: "text".to_string(),
            must_have: false,
        }
    }
}

// ===========================================================================
// Data types - mirror ac_status.rs JSON output (schema v2.0)
// ===========================================================================

/// Deserialized AC status snapshot (from ac-status --json)
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
// Aggregated snapshot metric
// ===========================================================================

/// Aggregated metrics from a single snapshot
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotMetric {
    /// Commit SHA (extracted from filename)
    pub commit: String,
    /// Timestamp from JSON
    pub timestamp: String,

    // Must-have AC stats
    pub must_have_total: usize,
    pub must_have_passing: usize,
    pub must_have_failing: usize,
    pub must_have_unknown: usize,

    // Optional AC stats
    pub optional_total: usize,
    pub optional_passing: usize,
    pub optional_failing: usize,
    pub optional_unknown: usize,

    /// Overall coverage percentage
    pub coverage_percent: f64,

    /// List of failing must-have AC IDs (blockers)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kernel_blockers: Vec<String>,
}

/// History report containing all snapshots
#[derive(Debug, Serialize)]
pub struct AcHistoryReport {
    /// Number of snapshots analyzed
    pub snapshot_count: usize,
    /// Date range (first timestamp to last)
    pub date_range: Option<(String, String)>,
    /// All snapshots sorted by timestamp
    pub snapshots: Vec<SnapshotMetric>,
    /// Delta analysis (new blockers between snapshots)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deltas: Vec<SnapshotDelta>,
}

/// Delta between consecutive snapshots
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

// ===========================================================================
// Parsing helpers
// ===========================================================================

/// Extract commit SHA from filename like "ac-status-abcd1234.json"
pub fn extract_commit_from_filename(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    // Expected format: ac-status-<sha>
    if let Some(sha) = stem.strip_prefix("ac-status-") { Some(sha.to_string()) } else { None }
}

/// Parse a single snapshot file
fn parse_snapshot(path: &Path) -> Result<SnapshotMetric> {
    let commit = extract_commit_from_filename(path)
        .ok_or_else(|| anyhow::anyhow!("Cannot extract commit from filename: {:?}", path))?;

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snapshot: {}", path.display()))?;

    let snapshot: AcStatusSnapshot = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse snapshot JSON: {}", path.display()))?;

    if snapshot.schema_version != "2.0" {
        eprintln!(
            "{} Snapshot {} has schema version '{}', expected '2.0'",
            "[WARN]".yellow(),
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

/// Load all snapshots from a directory
pub fn load_snapshots(dir: &Path) -> Result<Vec<SnapshotMetric>> {
    if !dir.exists() {
        anyhow::bail!("Snapshot directory does not exist: {}", dir.display());
    }

    let mut snapshots = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            match parse_snapshot(&path) {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(e) => {
                    eprintln!("{} Skipping {}: {}", "[WARN]".yellow(), path.display(), e);
                }
            }
        }
    }

    // Sort by timestamp
    snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(snapshots)
}

/// Compute deltas between consecutive snapshots
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

/// Build the history report
pub fn build_report(snapshots: Vec<SnapshotMetric>) -> AcHistoryReport {
    let snapshot_count = snapshots.len();

    let date_range = if snapshot_count >= 2 {
        Some((snapshots[0].timestamp.clone(), snapshots[snapshot_count - 1].timestamp.clone()))
    } else if snapshot_count == 1 {
        Some((snapshots[0].timestamp.clone(), snapshots[0].timestamp.clone()))
    } else {
        None
    };

    let deltas = compute_deltas(&snapshots);

    AcHistoryReport { snapshot_count, date_range, snapshots, deltas }
}

// ===========================================================================
// Main entry point
// ===========================================================================

pub fn run(args: AcHistoryArgs) -> Result<()> {
    let snapshots = load_snapshots(&args.dir)?;

    if snapshots.is_empty() {
        println!("{} No snapshots found in {}", "[INFO]".blue(), args.dir.display());
        println!("  Run CI to generate ac-status snapshots, then download artifacts.");
        return Ok(());
    }

    let report = build_report(snapshots);

    match args.format.as_str() {
        "text" => render_text(&report, &args),
        "markdown" => render_markdown(&report, &args),
        "csv" => render_csv(&report, &args),
        "json" => render_json(&report),
        _ => anyhow::bail!("Unknown format: {}. Use text, markdown, csv, or json.", args.format),
    }
}

// ===========================================================================
// Text output (default)
// ===========================================================================

fn render_text(report: &AcHistoryReport, args: &AcHistoryArgs) -> Result<()> {
    println!("{}", "AC Coverage History".cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("  {} snapshots analyzed", report.snapshot_count);
    if let Some((start, end)) = &report.date_range {
        println!("  Date range: {} → {}", start, end);
    }
    println!();

    if args.must_have {
        println!("{}", "Must-have ACs (kernel):".bold());
    } else {
        println!("{}", "All ACs:".bold());
    }
    println!();

    // Table header
    println!(
        "  {:12} {:24} {:>6} {:>5} {:>5} {:>5}",
        "Commit", "Date", "Cov%", "Pass", "Fail", "Unk"
    );
    println!("  {}", "─".repeat(60));

    for snapshot in &report.snapshots {
        let (pass, fail, unk, total) = if args.must_have {
            (
                snapshot.must_have_passing,
                snapshot.must_have_failing,
                snapshot.must_have_unknown,
                snapshot.must_have_total,
            )
        } else {
            (
                snapshot.must_have_passing + snapshot.optional_passing,
                snapshot.must_have_failing + snapshot.optional_failing,
                snapshot.must_have_unknown + snapshot.optional_unknown,
                snapshot.must_have_total + snapshot.optional_total,
            )
        };

        let cov = if total > 0 { (pass as f64 / total as f64) * 100.0 } else { 0.0 };

        // Truncate commit to 12 chars
        let short_commit =
            if snapshot.commit.len() > 12 { &snapshot.commit[..12] } else { &snapshot.commit };

        // Truncate timestamp to date portion
        let short_date = if snapshot.timestamp.len() >= 10 {
            &snapshot.timestamp[..10]
        } else {
            &snapshot.timestamp
        };

        let fail_str = if fail > 0 { fail.to_string().red().to_string() } else { "0".to_string() };

        println!(
            "  {:12} {:24} {:>5.1}% {:>5} {:>5} {:>5}",
            short_commit, short_date, cov, pass, fail_str, unk
        );
    }
    println!();

    // Show deltas if any
    if !report.deltas.is_empty() {
        println!("{}", "Notable Changes:".bold());
        for delta in &report.deltas {
            if !delta.new_blockers.is_empty() {
                println!(
                    "  {} {} – New blockers: {}",
                    "↓".red(),
                    &delta.commit[..12.min(delta.commit.len())],
                    delta.new_blockers.join(", ")
                );
            }
            if !delta.resolved_blockers.is_empty() {
                println!(
                    "  {} {} – Resolved: {}",
                    "↑".green(),
                    &delta.commit[..12.min(delta.commit.len())],
                    delta.resolved_blockers.join(", ")
                );
            }
            if delta.new_blockers.is_empty()
                && delta.resolved_blockers.is_empty()
                && delta.coverage_delta.abs() > 0.5
            {
                let arrow = if delta.coverage_delta > 0.0 { "↑".green() } else { "↓".red() };
                println!(
                    "  {} {} – Coverage: {:+.1}%",
                    arrow,
                    &delta.commit[..12.min(delta.commit.len())],
                    delta.coverage_delta
                );
            }
        }
        println!();
    }

    // Latest snapshot summary
    if let Some(latest) = report.snapshots.last() {
        println!("{}", "Latest Snapshot:".bold());
        println!("  Commit: {}", latest.commit);
        println!("  Coverage: {:.1}%", latest.coverage_percent);
        if !latest.kernel_blockers.is_empty() {
            println!(
                "  {} Kernel blockers: {}",
                "[WARN]".yellow(),
                latest.kernel_blockers.join(", ")
            );
        } else {
            println!("  {} No kernel blockers", "[OK]".green());
        }
    }

    Ok(())
}

// ===========================================================================
// Markdown output
// ===========================================================================

fn render_markdown(report: &AcHistoryReport, args: &AcHistoryArgs) -> Result<()> {
    let mut out = std::io::stdout();
    render_markdown_to(&mut out, report, args)
}

fn render_markdown_to<W: Write>(
    out: &mut W,
    report: &AcHistoryReport,
    args: &AcHistoryArgs,
) -> Result<()> {
    writeln!(out, "## AC Coverage History")?;
    writeln!(out)?;

    writeln!(out, "**Snapshots:** {}", report.snapshot_count)?;
    if let Some((start, end)) = &report.date_range {
        writeln!(out, "**Date range:** {} → {}", start, end)?;
    }
    writeln!(out)?;

    // Table
    let header = if args.must_have {
        "| Commit | Date | Cov% | Pass | Fail | Unknown |"
    } else {
        "| Commit | Date | Cov% | Pass | Fail | Unknown |"
    };
    writeln!(out, "{}", header)?;
    writeln!(out, "|--------|------|------|------|------|---------|")?;

    for snapshot in &report.snapshots {
        let (pass, fail, unk, total) = if args.must_have {
            (
                snapshot.must_have_passing,
                snapshot.must_have_failing,
                snapshot.must_have_unknown,
                snapshot.must_have_total,
            )
        } else {
            (
                snapshot.must_have_passing + snapshot.optional_passing,
                snapshot.must_have_failing + snapshot.optional_failing,
                snapshot.must_have_unknown + snapshot.optional_unknown,
                snapshot.must_have_total + snapshot.optional_total,
            )
        };

        let cov = if total > 0 { (pass as f64 / total as f64) * 100.0 } else { 0.0 };

        let short_commit =
            if snapshot.commit.len() > 8 { &snapshot.commit[..8] } else { &snapshot.commit };

        let short_date = if snapshot.timestamp.len() >= 10 {
            &snapshot.timestamp[..10]
        } else {
            &snapshot.timestamp
        };

        writeln!(
            out,
            "| {} | {} | {:.1}% | {} | {} | {} |",
            short_commit, short_date, cov, pass, fail, unk
        )?;
    }
    writeln!(out)?;

    // Notable changes
    if !report.deltas.is_empty() {
        writeln!(out, "### Notable Changes")?;
        writeln!(out)?;
        for delta in &report.deltas {
            let short_commit =
                if delta.commit.len() > 8 { &delta.commit[..8] } else { &delta.commit };

            if !delta.new_blockers.is_empty() {
                writeln!(
                    out,
                    "- **{}** ⚠️ New blockers: {}",
                    short_commit,
                    delta.new_blockers.join(", ")
                )?;
            }
            if !delta.resolved_blockers.is_empty() {
                writeln!(
                    out,
                    "- **{}** ✅ Resolved: {}",
                    short_commit,
                    delta.resolved_blockers.join(", ")
                )?;
            }
        }
        writeln!(out)?;
    }

    Ok(())
}

// ===========================================================================
// CSV output
// ===========================================================================

fn render_csv(report: &AcHistoryReport, args: &AcHistoryArgs) -> Result<()> {
    let mut out = std::io::stdout();
    render_csv_to(&mut out, report, args)
}

fn render_csv_to<W: Write>(
    out: &mut W,
    report: &AcHistoryReport,
    args: &AcHistoryArgs,
) -> Result<()> {
    writeln!(
        out,
        "commit,timestamp,coverage_percent,total,passing,failing,unknown,kernel_blockers"
    )?;

    for snapshot in &report.snapshots {
        let (pass, fail, unk, total) = if args.must_have {
            (
                snapshot.must_have_passing,
                snapshot.must_have_failing,
                snapshot.must_have_unknown,
                snapshot.must_have_total,
            )
        } else {
            (
                snapshot.must_have_passing + snapshot.optional_passing,
                snapshot.must_have_failing + snapshot.optional_failing,
                snapshot.must_have_unknown + snapshot.optional_unknown,
                snapshot.must_have_total + snapshot.optional_total,
            )
        };

        let cov = if total > 0 { (pass as f64 / total as f64) * 100.0 } else { 0.0 };
        let blockers = snapshot.kernel_blockers.join(";");

        writeln!(
            out,
            "{},{},{:.2},{},{},{},{},\"{}\"",
            snapshot.commit, snapshot.timestamp, cov, total, pass, fail, unk, blockers
        )?;
    }

    Ok(())
}

// ===========================================================================
// JSON output
// ===========================================================================

fn render_json(report: &AcHistoryReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report).context("Failed to serialize report")?;
    println!("{}", json);
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        let path = PathBuf::from("ac-status-abc123.json");
        assert_eq!(extract_commit_from_filename(&path), Some("abc123".to_string()));
    }

    #[test]
    fn extract_commit_from_long_sha() {
        let path = PathBuf::from("ac-status-abcdef1234567890.json");
        assert_eq!(extract_commit_from_filename(&path), Some("abcdef1234567890".to_string()));
    }

    #[test]
    fn extract_commit_from_invalid_filename() {
        let path = PathBuf::from("not-a-snapshot.json");
        assert_eq!(extract_commit_from_filename(&path), None);
    }

    #[test]
    fn load_snapshots_from_directory() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();

        assert_eq!(snapshots.len(), 2);
        // Should be sorted by timestamp
        assert_eq!(snapshots[0].timestamp, "2025-12-01T10:00:00Z");
        assert_eq!(snapshots[1].timestamp, "2025-12-02T10:00:00Z");
    }

    #[test]
    fn snapshot_extracts_kernel_blockers() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();

        // First snapshot has 2 failing kernel ACs
        assert_eq!(snapshots[0].kernel_blockers.len(), 2);
        assert!(snapshots[0].kernel_blockers.contains(&"AC-KERN-002".to_string()));
        assert!(snapshots[0].kernel_blockers.contains(&"AC-KERN-003".to_string()));

        // Second snapshot has no blockers
        assert!(snapshots[1].kernel_blockers.is_empty());
    }

    #[test]
    fn build_report_computes_date_range() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);

        assert_eq!(report.snapshot_count, 2);
        let (start, end) = report.date_range.unwrap();
        assert_eq!(start, "2025-12-01T10:00:00Z");
        assert_eq!(end, "2025-12-02T10:00:00Z");
    }

    #[test]
    fn build_report_computes_deltas() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);

        // Should have delta showing resolved blockers
        assert_eq!(report.deltas.len(), 1);
        let delta = &report.deltas[0];
        assert_eq!(delta.commit, "def456");
        assert!(delta.new_blockers.is_empty());
        assert_eq!(delta.resolved_blockers.len(), 2);
        assert!(delta.resolved_blockers.contains(&"AC-KERN-002".to_string()));
        assert!(delta.resolved_blockers.contains(&"AC-KERN-003".to_string()));
        assert!(delta.coverage_delta > 15.0); // ~20% improvement
    }

    #[test]
    fn csv_output_has_header_and_rows() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);
        let args = AcHistoryArgs::default();

        let mut buf = Vec::new();
        render_csv_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Check header
        assert!(output.contains("commit,timestamp,coverage_percent"));

        // Check data rows exist
        assert!(output.contains("abc123"));
        assert!(output.contains("def456"));
        assert!(output.contains("2025-12-01"));
        assert!(output.contains("2025-12-02"));
    }

    #[test]
    fn markdown_output_has_table() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);
        let args = AcHistoryArgs::default();

        let mut buf = Vec::new();
        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("## AC Coverage History"));
        assert!(output.contains("| Commit | Date | Cov%"));
        assert!(output.contains("|--------|------|------"));
    }

    #[test]
    fn markdown_output_shows_resolved_blockers() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);
        let args = AcHistoryArgs::default();

        let mut buf = Vec::new();
        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("### Notable Changes"));
        assert!(output.contains("✅ Resolved"));
        assert!(output.contains("AC-KERN-002"));
    }

    #[test]
    fn json_output_parses_back() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);

        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["snapshot_count"], 2);
        assert!(parsed["snapshots"].is_array());
        assert!(parsed["deltas"].is_array());
    }

    #[test]
    fn must_have_filter_affects_output() {
        let dir = create_test_dir();
        let snapshots = load_snapshots(dir.path()).unwrap();
        let report = build_report(snapshots);

        // With must_have=true
        let args_kernel = AcHistoryArgs { must_have: true, ..Default::default() };
        let mut buf_kernel = Vec::new();
        render_csv_to(&mut buf_kernel, &report, &args_kernel).unwrap();
        let output_kernel = String::from_utf8(buf_kernel).unwrap();

        // With must_have=false (all ACs)
        let args_all = AcHistoryArgs { must_have: false, ..Default::default() };
        let mut buf_all = Vec::new();
        render_csv_to(&mut buf_all, &report, &args_all).unwrap();
        let output_all = String::from_utf8(buf_all).unwrap();

        // Numbers should be different (kernel has 10+5=15 total, kernel-only has 10)
        // We can verify by checking the total column differs
        assert_ne!(output_kernel, output_all);
    }

    #[test]
    fn empty_directory_returns_empty_snapshots() {
        let dir = TempDir::new().unwrap();
        let snapshots = load_snapshots(dir.path()).unwrap();
        assert!(snapshots.is_empty());
    }

    #[test]
    fn nonexistent_directory_returns_error() {
        let result = load_snapshots(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
