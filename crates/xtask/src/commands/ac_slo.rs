//! AC SLO: Governance Service Level Objective checker
//!
//! Evaluates whether the latest AC coverage snapshot meets defined SLO thresholds.
//!
//! # Design
//!
//! This command consumes ac-status snapshots (same as ac-history) and checks if the
//! latest snapshot meets SLO criteria. Returns exit code 0 if SLO is met, non-zero otherwise.
//!
//! # Usage
//!
//! ```bash
//! # Basic usage - check against default thresholds
//! cargo xtask ac-slo --dir ./artifacts/ac-status
//!
//! # Custom thresholds
//! cargo xtask ac-slo --dir ./artifacts/ac-status --min-coverage 95 --max-blockers 0
//!
//! # Strict mode (any unknown counts as failure)
//! cargo xtask ac-slo --dir ./artifacts/ac-status --max-unknown 0
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use std::path::PathBuf;

use ac_kernel::{build_report, load_snapshots};

/// Arguments for ac-slo command
#[derive(Debug, Clone)]
pub struct AcSloArgs {
    /// Directory containing ac-status JSON snapshots
    pub dir: PathBuf,
    /// Minimum required coverage percentage (default: 80.0)
    pub min_coverage: f64,
    /// Maximum allowed kernel blockers (default: 0)
    pub max_blockers: usize,
    /// Maximum allowed unknown status ACs (-1 = no limit, default)
    pub max_unknown: Option<usize>,
    /// Output format (text, json)
    pub format: String,
}

impl Default for AcSloArgs {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("artifacts/ac-status"),
            min_coverage: 80.0,
            max_blockers: 0,
            max_unknown: None, // No limit by default
            format: "text".to_string(),
        }
    }
}

/// SLO check result
#[derive(Debug, Serialize)]
pub struct SloResult {
    /// Schema version for this output
    pub schema_version: String,
    /// Whether the SLO was met
    pub passed: bool,
    /// Commit SHA of the evaluated snapshot
    pub commit: String,
    /// Timestamp of the evaluated snapshot
    pub timestamp: String,
    /// Actual coverage percentage
    pub coverage_percent: f64,
    /// Minimum required coverage (SLO threshold)
    pub min_coverage: f64,
    /// Whether coverage threshold was met
    pub coverage_ok: bool,
    /// Number of kernel blockers
    pub kernel_blockers: usize,
    /// Maximum allowed blockers (SLO threshold)
    pub max_blockers: usize,
    /// Whether blockers threshold was met
    pub blockers_ok: bool,
    /// List of failing kernel AC IDs
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocker_ids: Vec<String>,
    /// Number of unknown status ACs
    pub unknown_count: usize,
    /// Maximum allowed unknowns (None = no limit)
    pub max_unknown: Option<usize>,
    /// Whether unknown threshold was met (always true if no limit)
    pub unknown_ok: bool,
    /// Human-readable summary
    pub summary: String,
}

/// Current schema version for ac-slo JSON output
pub const AC_SLO_SCHEMA_VERSION: &str = "1.0";

pub fn run(args: AcSloArgs) -> Result<()> {
    let load_result = load_snapshots(&args.dir)?;

    if load_result.snapshots.is_empty() {
        eprintln!("{} No snapshots found in {}", "[ERROR]".red(), args.dir.display());
        eprintln!("  Cannot evaluate SLO without AC status data.");
        eprintln!("  Run CI to generate ac-status snapshots, then download artifacts.");
        std::process::exit(1);
    }

    let report = build_report(load_result.snapshots, load_result.skipped);

    // Get the latest snapshot
    let latest = report.snapshots.last().context("No snapshots in report")?;

    // Evaluate SLO conditions
    let coverage_ok = latest.coverage_percent >= args.min_coverage;
    let blockers_ok = latest.kernel_blockers.len() <= args.max_blockers;
    let unknown_count = latest.must_have_unknown + latest.optional_unknown;
    let unknown_ok = args.max_unknown.map_or(true, |max| unknown_count <= max);

    let passed = coverage_ok && blockers_ok && unknown_ok;

    // Build summary
    let summary = if passed {
        format!(
            "AC SLO OK: coverage {:.1}% (>={:.0}%), {} kernel blockers (<={})",
            latest.coverage_percent,
            args.min_coverage,
            latest.kernel_blockers.len(),
            args.max_blockers
        )
    } else {
        let mut issues = Vec::new();
        if !coverage_ok {
            issues.push(format!(
                "coverage {:.1}% (<{:.0}%)",
                latest.coverage_percent, args.min_coverage
            ));
        }
        if !blockers_ok {
            issues.push(format!(
                "{} kernel blockers (>{}): {}",
                latest.kernel_blockers.len(),
                args.max_blockers,
                latest.kernel_blockers.join(", ")
            ));
        }
        if !unknown_ok {
            issues.push(format!(
                "{} unknown ACs (>{})",
                unknown_count,
                args.max_unknown.unwrap_or(0)
            ));
        }
        format!("AC SLO VIOLATED: {}", issues.join(", "))
    };

    let result = SloResult {
        schema_version: AC_SLO_SCHEMA_VERSION.to_string(),
        passed,
        commit: latest.commit.clone(),
        timestamp: latest.timestamp.clone(),
        coverage_percent: latest.coverage_percent,
        min_coverage: args.min_coverage,
        coverage_ok,
        kernel_blockers: latest.kernel_blockers.len(),
        max_blockers: args.max_blockers,
        blockers_ok,
        blocker_ids: latest.kernel_blockers.clone(),
        unknown_count,
        max_unknown: args.max_unknown,
        unknown_ok,
        summary: summary.clone(),
    };

    match args.format.as_str() {
        "json" => {
            let json =
                serde_json::to_string_pretty(&result).context("Failed to serialize SLO result")?;
            println!("{}", json);
        }
        _ => {
            // Text output
            if passed {
                println!("{} {}", "[SLO OK]".green().bold(), summary);
            } else {
                println!("{} {}", "[SLO VIOLATED]".red().bold(), summary);
            }
            println!();
            println!("  Commit:    {}", latest.commit);
            println!("  Timestamp: {}", latest.timestamp);
            println!(
                "  Coverage:  {:.1}% (threshold: {:.0}%)",
                latest.coverage_percent, args.min_coverage
            );
            println!(
                "  Blockers:  {} (threshold: {})",
                latest.kernel_blockers.len(),
                args.max_blockers
            );
            if !latest.kernel_blockers.is_empty() {
                println!("             {}", latest.kernel_blockers.join(", "));
            }
            if args.max_unknown.is_some() {
                println!(
                    "  Unknown:   {} (threshold: {})",
                    unknown_count,
                    args.max_unknown.unwrap()
                );
            }
        }
    }

    if !passed {
        std::process::exit(1);
    }

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

    const SAMPLE_PASSING_SNAPSHOT: &str = r#"{
        "schema_version": "2.0",
        "timestamp": "2025-12-01T10:00:00Z",
        "must_have_acs": {"total": 10, "passing": 10, "failing": 0, "unknown": 0},
        "optional_acs": {"total": 5, "passing": 5, "failing": 0, "unknown": 0},
        "coverage_percent": 100.0,
        "acs": [
            {"id": "AC-KERN-001", "status": "pass", "must_have_ac": true}
        ]
    }"#;

    const SAMPLE_FAILING_SNAPSHOT: &str = r#"{
        "schema_version": "2.0",
        "timestamp": "2025-12-01T10:00:00Z",
        "must_have_acs": {"total": 10, "passing": 5, "failing": 3, "unknown": 2},
        "optional_acs": {"total": 5, "passing": 3, "failing": 1, "unknown": 1},
        "coverage_percent": 53.3,
        "acs": [
            {"id": "AC-KERN-001", "status": "pass", "must_have_ac": true},
            {"id": "AC-KERN-002", "status": "fail", "must_have_ac": true},
            {"id": "AC-KERN-003", "status": "fail", "must_have_ac": true}
        ]
    }"#;

    fn create_passing_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("ac-status-abc123.json"), SAMPLE_PASSING_SNAPSHOT).unwrap();
        dir
    }

    fn create_failing_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("ac-status-def456.json"), SAMPLE_FAILING_SNAPSHOT).unwrap();
        dir
    }

    #[test]
    fn slo_result_schema_version_is_stable() {
        assert_eq!(AC_SLO_SCHEMA_VERSION, "1.0");
    }

    #[test]
    fn passing_snapshot_meets_default_slo() {
        let dir = create_passing_dir();
        let load_result = load_snapshots(dir.path()).unwrap();
        let report = build_report(load_result.snapshots, load_result.skipped);
        let latest = report.snapshots.last().unwrap();

        let args = AcSloArgs::default();
        let coverage_ok = latest.coverage_percent >= args.min_coverage;
        let blockers_ok = latest.kernel_blockers.len() <= args.max_blockers;

        assert!(coverage_ok, "Coverage should meet 80% threshold");
        assert!(blockers_ok, "Should have 0 blockers");
    }

    #[test]
    fn failing_snapshot_violates_slo() {
        let dir = create_failing_dir();
        let load_result = load_snapshots(dir.path()).unwrap();
        let report = build_report(load_result.snapshots, load_result.skipped);
        let latest = report.snapshots.last().unwrap();

        let args = AcSloArgs::default();
        let coverage_ok = latest.coverage_percent >= args.min_coverage;
        let blockers_ok = latest.kernel_blockers.len() <= args.max_blockers;

        assert!(!coverage_ok, "Coverage 53.3% should fail 80% threshold");
        assert!(!blockers_ok, "2 blockers should fail 0 threshold");
    }

    #[test]
    fn custom_thresholds_work() {
        let dir = create_failing_dir();
        let load_result = load_snapshots(dir.path()).unwrap();
        let report = build_report(load_result.snapshots, load_result.skipped);
        let latest = report.snapshots.last().unwrap();

        // With relaxed thresholds, this should pass
        let args = AcSloArgs { min_coverage: 50.0, max_blockers: 5, ..Default::default() };

        let coverage_ok = latest.coverage_percent >= args.min_coverage;
        let blockers_ok = latest.kernel_blockers.len() <= args.max_blockers;

        assert!(coverage_ok, "53.3% should meet 50% threshold");
        assert!(blockers_ok, "2 blockers should meet 5 threshold");
    }

    #[test]
    fn unknown_threshold_works() {
        let dir = create_failing_dir();
        let load_result = load_snapshots(dir.path()).unwrap();
        let report = build_report(load_result.snapshots, load_result.skipped);
        let latest = report.snapshots.last().unwrap();

        let unknown_count = latest.must_have_unknown + latest.optional_unknown;

        // With no limit, should pass
        let no_limit_ok: bool = None::<usize>.map_or(true, |max| unknown_count <= max);
        assert!(no_limit_ok);

        // With strict limit of 0, should fail (has 3 unknown)
        let strict_ok = Some(0_usize).map_or(true, |max| unknown_count <= max);
        assert!(!strict_ok);

        // With reasonable limit, should pass
        let reasonable_ok = Some(5_usize).map_or(true, |max| unknown_count <= max);
        assert!(reasonable_ok);
    }

    /// Shape lock test for ac-slo JSON output
    #[test]
    fn ac_slo_json_shape_is_stable() {
        let result = SloResult {
            schema_version: AC_SLO_SCHEMA_VERSION.to_string(),
            passed: true,
            commit: "abc123".to_string(),
            timestamp: "2025-12-01T10:00:00Z".to_string(),
            coverage_percent: 95.0,
            min_coverage: 80.0,
            coverage_ok: true,
            kernel_blockers: 0,
            max_blockers: 0,
            blockers_ok: true,
            blocker_ids: vec![],
            unknown_count: 0,
            max_unknown: None,
            unknown_ok: true,
            summary: "AC SLO OK".to_string(),
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Required top-level fields
        let required_fields = [
            "schema_version",
            "passed",
            "commit",
            "timestamp",
            "coverage_percent",
            "min_coverage",
            "coverage_ok",
            "kernel_blockers",
            "max_blockers",
            "blockers_ok",
            "unknown_count",
            "unknown_ok",
            "summary",
        ];

        for field in required_fields {
            assert!(parsed.get(field).is_some(), "Missing required field: {}", field);
        }

        assert_eq!(parsed["schema_version"].as_str().unwrap(), "1.0");
        assert!(parsed["passed"].is_boolean());
        assert!(parsed["coverage_percent"].is_f64());
        assert!(parsed["kernel_blockers"].is_u64());
    }
}
