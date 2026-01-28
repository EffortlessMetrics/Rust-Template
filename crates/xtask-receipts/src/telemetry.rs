//! Telemetry receipt generation.

use anyhow::{Context, Result};
use colored::Colorize;
use gov_receipts::{
    ChangeSurface, GeigerSummary, MetaConfidence, ProbeProfile, ProbeResult, ProbeStatus,
    ReceiptMeta, Safety, SkippedProbe, Structure, TelemetryReceipt, TelemetryVerification,
};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crate::timeline::{
    count_unsafe_delta, detect_contract_changes, get_crates_touched, get_diff_stat,
    get_modules_touched_names,
};
use crate::{generate_run_id, get_current_commit_short};

/// Arguments for the receipts-telemetry command
#[derive(Debug, Clone)]
pub struct ReceiptsTelemetryArgs {
    /// Pull request number (optional)
    pub pr: Option<u32>,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Probe profile: fast/full/exhibit
    pub profile: String,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
    /// Shared run_id (for forensic orchestration); generated if None
    pub run_id: Option<String>,
}

impl Default for ReceiptsTelemetryArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            profile: "fast".to_string(),
            base_branch: "origin/main".to_string(),
            run_id: None,
        }
    }
}

/// Convert probe profile to meta confidence level
fn profile_to_meta_confidence(profile: ProbeProfile) -> MetaConfidence {
    match profile {
        ProbeProfile::Fast => MetaConfidence::Low,
        ProbeProfile::Full => MetaConfidence::Medium,
        ProbeProfile::Exhibit => MetaConfidence::High,
    }
}

/// Generate telemetry.json receipt with probe execution results
pub fn run_telemetry(args: ReceiptsTelemetryArgs) -> Result<()> {
    println!("{}", "Generating telemetry receipt...".blue().bold());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    // Use provided run_id (for forensic orchestration) or generate new one
    let run_id = args.run_id.clone().unwrap_or_else(|| generate_run_id(args.pr));

    // Parse profile
    let profile = match args.profile.to_lowercase().as_str() {
        "fast" => ProbeProfile::Fast,
        "full" => ProbeProfile::Full,
        "exhibit" => ProbeProfile::Exhibit,
        _ => ProbeProfile::Fast,
    };

    // Get change surface from git diff
    let diff_stat = get_diff_stat(&args.base_branch);
    let crates_touched = get_crates_touched(&args.base_branch);
    let modules_touched_names = get_modules_touched_names(&args.base_branch);

    let change_surface = ChangeSurface {
        files_changed: diff_stat.files_changed,
        insertions: diff_stat.insertions,
        deletions: diff_stat.deletions,
        hotspots: vec![],
        modules_touched: modules_touched_names,
        crates_touched,
    };

    // Detect contract changes
    let contracts = detect_contract_changes(&args.base_branch);

    // Build probe results based on profile
    let mut probes = Vec::new();
    let mut not_run = Vec::new();
    let mut safety: Option<Safety> = None;
    let mut structure: Option<Structure> = None;
    let mut verification: Option<TelemetryVerification> = None;

    // Add a basic probe result for git-diff (always runs)
    probes.push(ProbeResult {
        name: "git-diff".to_string(),
        version: None,
        status: ProbeStatus::Run,
        reason: None,
        duration_ms: Some(0),
        artifact_path: None,
    });

    match profile {
        ProbeProfile::Fast => {
            // Fast profile skips heavy analysis tools
            not_run.push(SkippedProbe {
                probe: "cargo-geiger".to_string(),
                reason: "fast profile".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "cargo-modules".to_string(),
                reason: "fast profile".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "cargo-deny".to_string(),
                reason: "fast profile".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "coverage".to_string(),
                reason: "fast profile".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "rust-code-analysis".to_string(),
                reason: "fast profile".to_string(),
            });
        }
        ProbeProfile::Full | ProbeProfile::Exhibit => {
            // Run cargo-geiger for safety analysis
            let (geiger_probe, geiger_safety) = run_geiger_probe();
            probes.push(geiger_probe);
            if let Some(s) = geiger_safety {
                // Merge with unsafe delta from git diff
                let unsafe_delta = count_unsafe_delta(&args.base_branch);
                safety = Some(Safety {
                    unsafe_added: unsafe_delta.added,
                    unsafe_removed: unsafe_delta.removed,
                    geiger_summary: s.geiger_summary,
                    pointers: s.pointers,
                });
            } else {
                // Just use git diff unsafe counts
                let unsafe_delta = count_unsafe_delta(&args.base_branch);
                if unsafe_delta.added > 0 || unsafe_delta.removed > 0 {
                    safety = Some(Safety {
                        unsafe_added: unsafe_delta.added,
                        unsafe_removed: unsafe_delta.removed,
                        geiger_summary: None,
                        pointers: vec![],
                    });
                }
            }

            // Run cargo-modules for structural analysis
            let (modules_probe, modules_structure) = run_modules_probe();
            probes.push(modules_probe);
            if let Some(s) = modules_structure {
                structure = Some(s);
            }

            // cargo-deny is still not integrated
            not_run.push(SkippedProbe {
                probe: "cargo-deny".to_string(),
                reason: "tooling not yet integrated".to_string(),
            });

            // coverage requires special setup
            not_run.push(SkippedProbe {
                probe: "coverage".to_string(),
                reason: "requires llvm-cov setup".to_string(),
            });

            // rust-code-analysis only for exhibit profile
            if profile == ProbeProfile::Exhibit {
                let (rca_probe, rca_verification) =
                    run_rust_code_analysis_probe(&args.base_branch, &receipts_dir);
                probes.push(rca_probe);
                if let Some(v) = rca_verification {
                    verification = Some(v);
                }
            } else {
                not_run.push(SkippedProbe {
                    probe: "rust-code-analysis".to_string(),
                    reason: "full profile (exhibit required)".to_string(),
                });
            }
        }
    }

    let mut builder = TelemetryReceipt::builder()
        .run_id(&run_id)
        .profile(profile)
        .change_surface(change_surface.clone())
        .contracts(contracts)
        .probes(probes);

    if let Some(s) = safety {
        builder = builder.safety(s);
    }

    if let Some(s) = structure {
        builder = builder.structure(s);
    }

    if let Some(v) = verification {
        builder = builder.verification(v);
    }

    for skip in not_run {
        builder = builder.skipped(skip);
    }

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

    // Build meta provenance
    let mut meta_builder = ReceiptMeta::builder()
        .method_id("telemetry-v1")
        .method_version(1)
        .analysis_run_id(&run_id)
        .input("git_diff")
        .input("git_log")
        .confidence(profile_to_meta_confidence(profile));

    if let Some(commit) = get_current_commit_short() {
        meta_builder = meta_builder.evidence(commit);
    }

    // Add evidence pointers for key files analyzed
    for crate_name in &change_surface.crates_touched {
        meta_builder = meta_builder.evidence(format!("crates/{}", crate_name));
    }

    builder = builder.meta(meta_builder.build());

    let receipt = builder.build();

    // Write receipt
    let telemetry_path = receipts_dir.join("telemetry.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&telemetry_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), telemetry_path.display());
    println!("  Profile: {}", args.profile);
    println!(
        "  Files changed: {}, +{} / -{}",
        diff_stat.files_changed, diff_stat.insertions, diff_stat.deletions
    );
    println!("  Probes ran: {}, skipped: {}", receipt.probes.len(), receipt.not_run.len());

    Ok(())
}

// ============================================================================
// PROBE IMPLEMENTATIONS
// ============================================================================

/// Check if a tool is available in PATH
fn is_tool_available(tool: &str) -> bool {
    Command::new("which").arg(tool).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Get version of a tool
fn get_tool_version(tool: &str) -> Option<String> {
    Command::new(tool).arg("--version").output().ok().and_then(|o| {
        let stdout = String::from_utf8_lossy(&o.stdout);
        // Extract first line and try to find version number
        stdout.lines().next().map(|s| s.to_string())
    })
}

/// Run cargo-geiger probe for unsafe code analysis
///
/// Returns (ProbeResult, Option<Safety>)
fn run_geiger_probe() -> (ProbeResult, Option<Safety>) {
    let start = Instant::now();

    // Check if cargo-geiger is installed
    if !is_tool_available("cargo-geiger") {
        return (
            ProbeResult {
                name: "cargo-geiger".to_string(),
                version: None,
                status: ProbeStatus::NotRun,
                reason: Some("tool not installed".to_string()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        );
    }

    let version = get_tool_version("cargo-geiger");
    println!("  Running cargo-geiger...");

    // Run cargo geiger with JSON output
    let output = Command::new("cargo").args(["geiger", "--output-format", "Json"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let safety = parse_geiger_output(&stdout);

            (
                ProbeResult {
                    name: "cargo-geiger".to_string(),
                    version,
                    status: ProbeStatus::Run,
                    reason: None,
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    artifact_path: None,
                },
                safety,
            )
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            (
                ProbeResult {
                    name: "cargo-geiger".to_string(),
                    version,
                    status: ProbeStatus::Error,
                    reason: Some(format!("exit code: {}", o.status.code().unwrap_or(-1))),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    artifact_path: None,
                },
                // Try to parse output even on error (partial output)
                parse_geiger_output(&stderr),
            )
        }
        Err(e) => (
            ProbeResult {
                name: "cargo-geiger".to_string(),
                version,
                status: ProbeStatus::Error,
                reason: Some(format!("failed to run: {}", e)),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        ),
    }
}

/// Parse cargo-geiger JSON output into Safety struct
fn parse_geiger_output(output: &str) -> Option<Safety> {
    // cargo-geiger JSON format has a "packages" array with unsafe counts
    // We'll extract the totals for our workspace
    #[derive(serde::Deserialize)]
    struct GeigerReport {
        #[serde(default)]
        packages: Vec<GeigerPackage>,
    }

    #[derive(serde::Deserialize)]
    struct GeigerPackage {
        #[serde(default)]
        unsafety: GeigerUnsafety,
    }

    #[derive(serde::Deserialize, Default)]
    struct GeigerUnsafety {
        #[serde(default)]
        used: GeigerCounts,
        #[serde(default)]
        unused: GeigerCounts,
        #[serde(default)]
        forbids_unsafe: bool,
    }

    #[derive(serde::Deserialize, Default)]
    struct GeigerCounts {
        #[serde(default)]
        functions: GeigerCount,
        #[serde(default)]
        exprs: GeigerCount,
        #[serde(default)]
        item_impls: GeigerCount,
        #[serde(default)]
        item_traits: GeigerCount,
        #[serde(default)]
        methods: GeigerCount,
    }

    #[derive(serde::Deserialize, Default)]
    struct GeigerCount {
        #[allow(dead_code)]
        #[serde(default)]
        safe: u32,
        #[serde(default)]
        unsafe_: u32,
    }

    // Try to parse the JSON - it might be mixed with other output
    for line in output.lines() {
        if let Ok(report) = serde_json::from_str::<GeigerReport>(line) {
            let mut used_total = 0u32;
            let mut unused_total = 0u32;
            let mut any_forbid = false;

            for pkg in &report.packages {
                used_total += pkg.unsafety.used.functions.unsafe_
                    + pkg.unsafety.used.exprs.unsafe_
                    + pkg.unsafety.used.item_impls.unsafe_
                    + pkg.unsafety.used.item_traits.unsafe_
                    + pkg.unsafety.used.methods.unsafe_;

                unused_total += pkg.unsafety.unused.functions.unsafe_
                    + pkg.unsafety.unused.exprs.unsafe_
                    + pkg.unsafety.unused.item_impls.unsafe_
                    + pkg.unsafety.unused.item_traits.unsafe_
                    + pkg.unsafety.unused.methods.unsafe_;

                if pkg.unsafety.forbids_unsafe {
                    any_forbid = true;
                }
            }

            return Some(Safety {
                unsafe_added: 0,
                unsafe_removed: 0,
                geiger_summary: Some(GeigerSummary {
                    used_unsafe: used_total,
                    unused_unsafe: unused_total,
                    forbid_unsafe: any_forbid,
                }),
                pointers: vec![],
            });
        }
    }

    None
}

/// Run cargo-modules probe for dependency cycle detection
///
/// Returns (ProbeResult, Option<Structure>)
fn run_modules_probe() -> (ProbeResult, Option<Structure>) {
    let start = Instant::now();

    // Check if cargo-modules is installed
    if !is_tool_available("cargo-modules") {
        return (
            ProbeResult {
                name: "cargo-modules".to_string(),
                version: None,
                status: ProbeStatus::NotRun,
                reason: Some("tool not installed".to_string()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        );
    }

    let version = get_tool_version("cargo-modules");
    println!("  Running cargo-modules dependencies --acyclic...");

    // Run cargo modules dependencies --acyclic
    // This command fails (non-zero exit) if cycles are detected
    let output = Command::new("cargo").args(["modules", "dependencies", "--acyclic"]).output();

    match output {
        Ok(o) => {
            let cycles_detected = !o.status.success();
            let stdout = String::from_utf8_lossy(&o.stdout);

            // Try to count dependency edges from output
            let edges_count = count_dependency_edges(&stdout);

            (
                ProbeResult {
                    name: "cargo-modules".to_string(),
                    version,
                    status: ProbeStatus::Run,
                    reason: if cycles_detected {
                        Some("cycles detected".to_string())
                    } else {
                        None
                    },
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    artifact_path: None,
                },
                Some(Structure {
                    cycles_detected,
                    dependency_edges_delta: edges_count.unwrap_or(0),
                    pointers: vec![],
                }),
            )
        }
        Err(e) => (
            ProbeResult {
                name: "cargo-modules".to_string(),
                version,
                status: ProbeStatus::Error,
                reason: Some(format!("failed to run: {}", e)),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        ),
    }
}

/// Count dependency edges from cargo-modules output
fn count_dependency_edges(output: &str) -> Option<i32> {
    // cargo-modules outputs dependency lines like "mod_a -> mod_b"
    let edge_count = output.lines().filter(|line| line.contains("->")).count();
    if edge_count > 0 { Some(edge_count as i32) } else { None }
}

/// Run rust-code-analysis probe for complexity metrics
///
/// Returns (ProbeResult, Option<TelemetryVerification>)
fn run_rust_code_analysis_probe(
    base_branch: &str,
    receipts_dir: &std::path::Path,
) -> (ProbeResult, Option<TelemetryVerification>) {
    let start = Instant::now();

    // Check if rust-code-analysis-cli is installed
    if !is_tool_available("rust-code-analysis-cli") {
        return (
            ProbeResult {
                name: "rust-code-analysis".to_string(),
                version: None,
                status: ProbeStatus::NotRun,
                reason: Some("tool not installed".to_string()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        );
    }

    let version = get_tool_version("rust-code-analysis-cli");
    println!("  Running rust-code-analysis on changed files...");

    // Get list of changed Rust files
    let changed_files = get_changed_rust_files(base_branch);
    if changed_files.is_empty() {
        return (
            ProbeResult {
                name: "rust-code-analysis".to_string(),
                version,
                status: ProbeStatus::Run,
                reason: Some("no Rust files changed".to_string()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                artifact_path: None,
            },
            None,
        );
    }

    // Run rust-code-analysis-cli on each file and collect metrics
    let artifact_path = receipts_dir.join("rca-metrics.json");
    let mut all_metrics = Vec::new();

    for file in &changed_files {
        let output = Command::new("rust-code-analysis-cli")
            .args(["--metrics", "-O", "json", "-p", file])
            .output();

        if let Ok(o) = output
            && o.status.success()
        {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&stdout) {
                all_metrics.push(serde_json::json!({
                    "file": file,
                    "metrics": metrics
                }));
            }
        }
    }

    // Write aggregated metrics to artifact file
    if !all_metrics.is_empty() {
        let _ = std::fs::write(&artifact_path, serde_json::to_string_pretty(&all_metrics).unwrap());
    }

    (
        ProbeResult {
            name: "rust-code-analysis".to_string(),
            version,
            status: ProbeStatus::Run,
            reason: None,
            duration_ms: Some(start.elapsed().as_millis() as u64),
            artifact_path: if !all_metrics.is_empty() {
                Some(artifact_path.to_string_lossy().to_string())
            } else {
                None
            },
        },
        Some(TelemetryVerification {
            tests_added: 0,
            tests_modified: 0,
            coverage_report_path: None,
            mutation_outcomes_path: None,
        }),
    )
}

/// Get list of changed Rust files
fn get_changed_rust_files(base_branch: &str) -> Vec<String> {
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|line| line.ends_with(".rs"))
            .map(|s| s.to_string())
            .collect(),
        Err(_) => vec![],
    }
}
