//! AC History: Time-series analysis of AC coverage snapshots
//!
//! This command aggregates AC status snapshots (from CI) to show governance trends over time.
//!
//! # Design
//!
//! Core types and logic live in `ac-kernel`:
//! - `SnapshotMetric`, `AcHistoryReport`, `SnapshotDelta` (data model)
//! - `load_snapshots()`, `build_report()` (parsing and aggregation)
//!
//! This module provides only CLI concerns:
//! - Argument parsing (`AcHistoryArgs`)
//! - Output formatting with colors (text, markdown, csv, json)
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
use std::io::Write;
use std::path::PathBuf;

// Import types and functions from ac-kernel
use ac_kernel::{AcHistoryReport, build_report, load_snapshots};

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
// Main entry point
// ===========================================================================

pub fn run(args: AcHistoryArgs) -> Result<()> {
    let load_result = load_snapshots(&args.dir)?;

    if load_result.snapshots.is_empty() {
        println!("{} No snapshots found in {}", "[INFO]".blue(), args.dir.display());
        println!("  Run CI to generate ac-status snapshots, then download artifacts.");
        if !load_result.skipped.is_empty() {
            println!("  {} files were skipped due to errors:", load_result.skipped.len());
            for sf in &load_result.skipped {
                println!("    - {}: {}", sf.filename, sf.reason);
            }
        }
        return Ok(());
    }

    let report = build_report(load_result.snapshots, load_result.skipped);

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
    println!("{}", "=".repeat(48));
    println!();

    println!("  {} snapshots analyzed", report.snapshot_count);
    if let Some((start, end)) = &report.date_range {
        println!("  Date range: {} -> {}", start, end);
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
    println!("  {}", "-".repeat(60));

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
                    "  {} {} - New blockers: {}",
                    "v".red(),
                    &delta.commit[..12.min(delta.commit.len())],
                    delta.new_blockers.join(", ")
                );
            }
            if !delta.resolved_blockers.is_empty() {
                println!(
                    "  {} {} - Resolved: {}",
                    "^".green(),
                    &delta.commit[..12.min(delta.commit.len())],
                    delta.resolved_blockers.join(", ")
                );
            }
            if delta.new_blockers.is_empty()
                && delta.resolved_blockers.is_empty()
                && delta.coverage_delta.abs() > 0.5
            {
                let arrow = if delta.coverage_delta > 0.0 { "^".green() } else { "v".red() };
                println!(
                    "  {} {} - Coverage: {:+.1}%",
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

    // Show skipped files if any
    if !report.skipped_files.is_empty() {
        println!();
        println!(
            "{} {} file(s) skipped due to errors:",
            "[WARN]".yellow(),
            report.skipped_files.len()
        );
        for sf in &report.skipped_files {
            println!("  - {}: {}", sf.filename, sf.reason);
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
        writeln!(out, "**Date range:** {} -> {}", start, end)?;
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
                    "- **{}** New blockers: {}",
                    short_commit,
                    delta.new_blockers.join(", ")
                )?;
            }
            if !delta.resolved_blockers.is_empty() {
                writeln!(
                    out,
                    "- **{}** Resolved: {}",
                    short_commit,
                    delta.resolved_blockers.join(", ")
                )?;
            }
        }
        writeln!(out)?;
    }

    // Show skipped files if any
    if !report.skipped_files.is_empty() {
        writeln!(out, "### Skipped Snapshots")?;
        writeln!(out)?;
        writeln!(out, "{} file(s) skipped due to errors:", report.skipped_files.len())?;
        writeln!(out)?;
        for sf in &report.skipped_files {
            writeln!(out, "- `{}`: {}", sf.filename, sf.reason)?;
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
    use ac_kernel::SkippedFile;
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
    fn csv_output_has_header_and_rows() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);
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
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);
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
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);
        let args = AcHistoryArgs::default();

        let mut buf = Vec::new();
        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("### Notable Changes"));
        assert!(output.contains("Resolved"));
        assert!(output.contains("AC-KERN-002"));
    }

    #[test]
    fn must_have_filter_affects_output() {
        let dir = create_test_dir();
        let result = load_snapshots(dir.path()).unwrap();
        let report = build_report(result.snapshots, result.skipped);

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
}
