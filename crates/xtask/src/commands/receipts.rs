//! Generate receipts from gate execution and economics tracking.
//!
//! This module provides:
//! - `receipts gate` command which runs validation gates (fmt, clippy, tests)
//!   and emits structured JSON receipts to `.runs/current/receipts/`.
//! - `receipts economics` command which records DevLT and compute spend.
//!
//! Receipts provide machine-readable evidence of gate execution for:
//! - CI pipelines
//! - IDP integrations
//! - Audit trails
//! - Agent workflows
//!
//! NOTE: Many utility functions and types in this module are re-exported from
//! the `xtask-receipts` library. See that library for the canonical
//! implementations of friction zone analysis, oscillation detection, and
//! historian appendix parsing.

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use gov_receipts::{
    Boundaries, ChangeSurface, Contracts, Convergence, FrictionZone, GeigerSummary,
    LlmBoundaryAssessment, LlmTestDepthAssessment, MetaConfidence, Oscillation, OscillationAction,
    OscillationType, ProbeProfile, ProbeResult, ProbeStatus, Quality, QualityReceipt, ReceiptMeta,
    Risks, Safety, Session, SessionClassification, SkippedProbe, Structure, TelemetryReceipt,
    TelemetryVerification, TimelineConfidence, TimelineReceipt, Topology, UnsafeDelta,
    Verification, WallClock,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use xtask_receipts::{
    HistorianQualityAppendix, categorize_friction_zones, extract_historian_appendix_json,
    generate_run_id, get_current_commit_full, get_current_commit_short, get_ref_sha,
    parse_historian_appendix, should_exclude_path,
};

// Re-export types from xtask_receipts for use in main.rs
pub use xtask_receipts::{
    ReceiptsEconomicsArgs, ReceiptsGateArgs, ReceiptsValidateArgs, run_economics, run_gate,
    run_validate,
};

/// Convert probe profile to meta confidence level
fn profile_to_meta_confidence(profile: ProbeProfile) -> MetaConfidence {
    match profile {
        ProbeProfile::Fast => MetaConfidence::Low,
        ProbeProfile::Full => MetaConfidence::Medium,
        ProbeProfile::Exhibit => MetaConfidence::High,
    }
}

/// Get top N changed files by numstat
fn get_numstat_top_n(base_branch: &str, n: usize) -> Vec<serde_json::Value> {
    let output = Command::new("git").args(["diff", "--numstat", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut entries: Vec<_> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        let added: i64 = parts[0].parse().unwrap_or(0);
                        let removed: i64 = parts[1].parse().unwrap_or(0);
                        let file = parts[2].to_string();
                        Some((added + removed, added, removed, file))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by total churn descending
            entries.sort_by(|a, b| b.0.cmp(&a.0));

            entries
                .into_iter()
                .take(n)
                .map(|(_, added, removed, file)| {
                    serde_json::json!({
                        "file": file,
                        "added": added,
                        "removed": removed
                    })
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

// ============================================================================
// QUALITY RECEIPT
// ============================================================================

/// Arguments for the receipts-quality command
#[derive(Debug, Clone)]
pub struct ReceiptsQualityArgs {
    /// Pull request number (optional)
    pub pr: Option<u32>,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
    /// LLM-provided boundary rating (improved/neutral/degraded)
    pub boundary_rating: Option<String>,
    /// LLM-provided test depth rating (hardened/mixed/shallow)
    pub test_depth_rating: Option<String>,
    /// LLM-provided notes (repeatable)
    pub notes: Vec<String>,
    /// Shared run_id (for forensic orchestration); generated if None
    pub run_id: Option<String>,
    /// Enable LLM semantic analysis via Historian agent
    pub llm: bool,
    /// Path to existing historian output (for offline/testing use)
    pub historian_output: Option<PathBuf>,
    /// Command template for running historian (use {input} as placeholder)
    pub historian_cmd: Option<String>,
}

impl Default for ReceiptsQualityArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            base_branch: "origin/main".to_string(),
            boundary_rating: None,
            test_depth_rating: None,
            notes: Vec::new(),
            run_id: None,
            llm: false,
            historian_output: None,
            historian_cmd: None,
        }
    }
}

/// Result of historian analysis retrieval
#[derive(Debug)]
enum HistorianResult {
    Success(HistorianQualityAppendix, PathBuf),
    ParseFailed(String),
    RunnerNotConfigured,
    CommandFailed(String),
}

/// Obtain historian analysis from file or by running command.
fn obtain_historian_analysis(
    args: &ReceiptsQualityArgs,
    analysis_dir: &std::path::Path,
) -> HistorianResult {
    // Priority 1: Read from provided file
    if let Some(ref output_path) = args.historian_output {
        match std::fs::read_to_string::<&PathBuf>(output_path) {
            Ok(content) => {
                // Copy historian output into run directory for durable evidence
                // This keeps the receipt self-contained and portable
                let durable_path = analysis_dir.join("historian.md");
                if let Err(e) = std::fs::write(&durable_path, &content) {
                    return HistorianResult::ParseFailed(format!(
                        "Failed to copy historian output to run directory: {}",
                        e
                    ));
                }
                println!(
                    "    → {} (copied from {})",
                    durable_path.display(),
                    output_path.display()
                );

                match extract_historian_appendix_json(&content).and_then(parse_historian_appendix) {
                    // Return the durable path, not the original
                    Ok(appendix) => return HistorianResult::Success(appendix, durable_path),
                    Err(e) => return HistorianResult::ParseFailed(e.to_string()),
                }
            }
            Err(e) => {
                return HistorianResult::ParseFailed(format!(
                    "Failed to read historian output: {}",
                    e
                ));
            }
        }
    }

    // Priority 2: Run historian command
    let cmd_template = args.historian_cmd.clone().or_else(|| std::env::var("HISTORIAN_CMD").ok());

    if let Some(template) = cmd_template {
        // Write input file for historian
        let input_path = analysis_dir.join("historian_input.json");
        let output_path = analysis_dir.join("historian.md");

        // Build enriched input for historian
        // This provides enough context for consistent semantic analysis
        let receipts_dir = args.output_dir.join("receipts");

        // Find existing receipts (if forensic was run before quality)
        let telemetry_path = receipts_dir.join("telemetry.json");
        let timeline_path = receipts_dir.join("timeline.json");

        // Get diff summary for context
        let diff_stat = get_diff_stat(&args.base_branch);
        let hotspots = find_hotspots(&args.base_branch);
        let modules = get_modules_touched_names(&args.base_branch);
        let numstat_top = get_numstat_top_n(&args.base_branch, 10);

        let input = serde_json::json!({
            "run_dir": args.output_dir.display().to_string(),
            "base_branch": args.base_branch,
            "pr": args.pr,
            // Git context
            "head_sha": get_current_commit_full(),
            "base_sha": get_ref_sha(&args.base_branch),
            // Existing receipts (paths for reference)
            "existing_receipts": {
                "telemetry": if telemetry_path.exists() { Some(telemetry_path.display().to_string()) } else { None },
                "timeline": if timeline_path.exists() { Some(timeline_path.display().to_string()) } else { None },
            },
            // Diff summary for context
            "diff_summary": {
                "files_changed": diff_stat.files_changed,
                "insertions": diff_stat.insertions,
                "deletions": diff_stat.deletions,
                "modules_touched": modules,
                "hotspots": hotspots,
                "numstat_top_10": numstat_top,
            }
        });

        if let Err(e) = std::fs::write(&input_path, serde_json::to_string_pretty(&input).unwrap()) {
            return HistorianResult::CommandFailed(format!(
                "Failed to write historian input: {}",
                e
            ));
        }
        println!("    → {}", input_path.display());

        // Substitute {input} placeholder
        let cmd = template.replace("{input}", &input_path.display().to_string());

        // Execute command
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &cmd]).output()
        } else {
            Command::new("sh").args(["-lc", &cmd]).output()
        };

        match result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return HistorianResult::CommandFailed(format!(
                        "Historian command failed: {}",
                        stderr
                    ));
                }

                // Write stdout to historian.md
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Err(e) = std::fs::write(&output_path, stdout.as_ref()) {
                    return HistorianResult::CommandFailed(format!(
                        "Failed to write historian output: {}",
                        e
                    ));
                }
                println!("    → {}", output_path.display());

                // Parse the output
                match std::fs::read_to_string(&output_path) {
                    Ok(content) => match extract_historian_appendix_json(&content)
                        .and_then(parse_historian_appendix)
                    {
                        Ok(appendix) => return HistorianResult::Success(appendix, output_path),
                        Err(e) => return HistorianResult::ParseFailed(e.to_string()),
                    },
                    Err(e) => {
                        return HistorianResult::ParseFailed(format!(
                            "Failed to read historian output: {}",
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                return HistorianResult::CommandFailed(format!(
                    "Failed to execute historian command: {}",
                    e
                ));
            }
        }
    }

    // No runner configured
    HistorianResult::RunnerNotConfigured
}

/// Parse confidence string to LlmConfidence enum
fn parse_llm_confidence(s: &str) -> Option<gov_receipts::LlmConfidence> {
    match s.to_lowercase().as_str() {
        "high" => Some(gov_receipts::LlmConfidence::High),
        "medium" => Some(gov_receipts::LlmConfidence::Medium),
        "low" => Some(gov_receipts::LlmConfidence::Low),
        _ => None,
    }
}

/// Parse confidence string to MetaConfidence enum
fn parse_meta_confidence(s: &str) -> Option<MetaConfidence> {
    match s.to_lowercase().as_str() {
        "high" => Some(MetaConfidence::High),
        "medium" => Some(MetaConfidence::Medium),
        "low" => Some(MetaConfidence::Low),
        _ => None,
    }
}

/// Generate quality.json receipt with hard metrics from git diff
pub fn run_quality(args: ReceiptsQualityArgs) -> Result<()> {
    println!("{}", "Generating quality receipt...".blue().bold());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    // Create analysis directory for historian artifacts
    let analysis_dir = args.output_dir.join("analysis");
    std::fs::create_dir_all(&analysis_dir)?;

    // Use provided run_id (for forensic orchestration) or generate new one
    let run_id = args.run_id.clone().unwrap_or_else(|| generate_run_id(args.pr));

    // Get diff stats from git
    let diff_stat = get_diff_stat(&args.base_branch);
    let modules_touched = count_modules_touched(&args.base_branch);
    let hotspots = find_hotspots(&args.base_branch);
    let unsafe_delta = count_unsafe_delta(&args.base_branch);
    let (tests_loc, impl_loc) = count_loc_changes(&args.base_branch);

    // Calculate test density delta
    let test_density_delta = if impl_loc > 0 {
        tests_loc as f64 / impl_loc as f64
    } else if tests_loc > 0 {
        1.0 // Pure test addition
    } else {
        0.0
    };

    // =========================================================================
    // Obtain historian analysis if --llm is enabled
    // =========================================================================
    let mut historian_appendix: Option<HistorianQualityAppendix> = None;
    let mut historian_path: Option<PathBuf> = None;
    let mut historian_assumptions: Vec<String> = Vec::new();
    let mut historian_confidence: Option<MetaConfidence> = None;

    if args.llm {
        println!("  {} LLM semantic analysis...", "Running".cyan());

        match obtain_historian_analysis(&args, &analysis_dir) {
            HistorianResult::Success(appendix, path) => {
                println!("  {} Historian appendix loaded from {}", "OK".green(), path.display());
                historian_confidence =
                    appendix.confidence.as_ref().and_then(|c| parse_meta_confidence(c));
                historian_appendix = Some(appendix);
                historian_path = Some(path);
            }
            HistorianResult::ParseFailed(err) => {
                println!("  {} Historian appendix parse failed: {}", "WARN".yellow(), err);
                historian_assumptions.push(format!("historian_appendix_parse_failed: {}", err));
                historian_confidence = Some(MetaConfidence::Low);
            }
            HistorianResult::RunnerNotConfigured => {
                println!(
                    "  {} No historian runner configured (use --historian-output or HISTORIAN_CMD)",
                    "WARN".yellow()
                );
                historian_assumptions.push("historian_runner_not_configured".to_string());
                historian_confidence = Some(MetaConfidence::Low);
            }
            HistorianResult::CommandFailed(err) => {
                println!("  {} Historian command failed: {}", "WARN".yellow(), err);
                historian_assumptions.push(format!("historian_cmd_failed: {}", err));
                historian_confidence = Some(MetaConfidence::Low);
            }
        }
    }

    // =========================================================================
    // Build quality metrics with optional historian merge
    // =========================================================================
    let mut boundaries = Boundaries { modules_touched, hotspots, llm_assessment: None };

    // Add LLM boundary assessment from CLI args or historian appendix
    if let Some(rating_str) = &args.boundary_rating
        && let Some(rating) = parse_boundary_rating(rating_str)
    {
        boundaries.llm_assessment = Some(LlmBoundaryAssessment {
            rating,
            notes: args.notes.clone(),
            confidence: None,
            evidence: vec![],
        });
    }

    // Merge historian boundary assessment (semantic fields only)
    if let Some(ref appendix) = historian_appendix {
        if let Some(ref rating_str) = appendix.boundary_rating
            && let Some(rating) = parse_boundary_rating(rating_str)
        {
            let existing = boundaries.llm_assessment.take();
            let mut notes = existing.as_ref().map(|e| e.notes.clone()).unwrap_or_default();
            notes.extend(appendix.boundary_notes.clone());

            let confidence = appendix
                .confidence
                .as_ref()
                .and_then(|c| parse_llm_confidence(c))
                .or_else(|| existing.as_ref().and_then(|e| e.confidence));

            let mut evidence = existing.as_ref().map(|e| e.evidence.clone()).unwrap_or_default();
            evidence.extend(appendix.evidence_pointers.clone());

            boundaries.llm_assessment =
                Some(LlmBoundaryAssessment { rating, notes, confidence, evidence });
        } else if !appendix.boundary_notes.is_empty() {
            // No rating but notes exist - create assessment with neutral rating and low confidence
            // This preserves LLM signal while explicitly marking the assumption
            let existing = boundaries.llm_assessment.take();
            let mut notes = existing.as_ref().map(|e| e.notes.clone()).unwrap_or_default();
            notes.extend(appendix.boundary_notes.clone());

            let mut evidence = existing.as_ref().map(|e| e.evidence.clone()).unwrap_or_default();
            evidence.extend(appendix.evidence_pointers.clone());

            boundaries.llm_assessment = Some(LlmBoundaryAssessment {
                rating: existing
                    .as_ref()
                    .map(|e| e.rating)
                    .unwrap_or(gov_receipts::BoundaryRating::Neutral),
                notes,
                confidence: Some(gov_receipts::LlmConfidence::Low),
                evidence,
            });
            historian_assumptions.push("historian_appendix_missing_boundary_rating".to_string());
        }
    }

    let mut verification = Verification {
        tests_added_loc: tests_loc,
        impl_added_loc: impl_loc,
        test_density_delta,
        llm_test_depth: None,
    };

    // Add LLM test depth assessment from CLI args
    if let Some(rating_str) = &args.test_depth_rating
        && let Some(rating) = parse_test_depth_rating(rating_str)
    {
        verification.llm_test_depth = Some(LlmTestDepthAssessment {
            rating,
            notes: vec![],
            confidence: None,
            evidence: vec![],
        });
    }

    // Merge historian test depth assessment (semantic fields only)
    if let Some(ref appendix) = historian_appendix {
        if let Some(ref rating_str) = appendix.test_depth_rating
            && let Some(rating) = parse_test_depth_rating(rating_str)
        {
            let existing = verification.llm_test_depth.take();
            let mut notes = existing.as_ref().map(|e| e.notes.clone()).unwrap_or_default();
            notes.extend(appendix.test_depth_notes.clone());

            let confidence = appendix
                .confidence
                .as_ref()
                .and_then(|c| parse_llm_confidence(c))
                .or_else(|| existing.as_ref().and_then(|e| e.confidence));

            let mut evidence = existing.as_ref().map(|e| e.evidence.clone()).unwrap_or_default();
            evidence.extend(appendix.evidence_pointers.clone());

            verification.llm_test_depth =
                Some(LlmTestDepthAssessment { rating, notes, confidence, evidence });
        } else if !appendix.test_depth_notes.is_empty() {
            // No rating but notes exist - create assessment with mixed rating and low confidence
            // This preserves LLM signal while explicitly marking the assumption
            let existing = verification.llm_test_depth.take();
            let mut notes = existing.as_ref().map(|e| e.notes.clone()).unwrap_or_default();
            notes.extend(appendix.test_depth_notes.clone());

            let mut evidence = existing.as_ref().map(|e| e.evidence.clone()).unwrap_or_default();
            evidence.extend(appendix.evidence_pointers.clone());

            verification.llm_test_depth = Some(LlmTestDepthAssessment {
                rating: existing
                    .as_ref()
                    .map(|e| e.rating)
                    .unwrap_or(gov_receipts::TestDepthRating::Mixed),
                notes,
                confidence: Some(gov_receipts::LlmConfidence::Low),
                evidence,
            });
            historian_assumptions.push("historian_appendix_missing_test_depth_rating".to_string());
        }
    }

    // Build risks with optional historian notes
    let mut llm_risk_notes = Vec::new();
    if let Some(ref appendix) = historian_appendix {
        llm_risk_notes.extend(appendix.risk_notes.clone());
    }

    // Store values before moving unsafe_delta
    let unsafe_added = unsafe_delta.added;
    let unsafe_removed = unsafe_delta.removed;

    let mut builder = QualityReceipt::builder().run_id(&run_id);

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

    // =========================================================================
    // Build meta provenance
    // =========================================================================
    let base_confidence = if args.llm {
        // If LLM was requested, use historian confidence or default to medium
        historian_confidence.unwrap_or(MetaConfidence::Medium)
    } else {
        MetaConfidence::Medium
    };

    let method_id = if args.llm { "quality-v1+llm" } else { "quality-v1" };

    let mut meta_builder = ReceiptMeta::builder()
        .method_id(method_id)
        .method_version(1)
        .analysis_run_id(&run_id)
        .input("git_diff")
        .input("git_numstat")
        .assumption(format!("base branch: {}", args.base_branch))
        .confidence(base_confidence);

    // Add historian-specific inputs and assumptions
    if args.llm {
        meta_builder = meta_builder.input("historian_appendix");
        for assumption in &historian_assumptions {
            meta_builder = meta_builder.assumption(assumption.clone());
        }

        // Add assumptions from historian appendix
        if let Some(ref appendix) = historian_appendix {
            for assumption in &appendix.assumptions {
                meta_builder = meta_builder.assumption(assumption.clone());
            }
        }
    }

    if let Some(commit) = get_current_commit_short() {
        meta_builder = meta_builder.evidence(commit);
    }

    // Add evidence pointers for hotspots
    for hotspot in boundaries.hotspots.iter().take(5) {
        meta_builder = meta_builder.evidence(hotspot.clone());
    }

    // Add historian evidence pointers
    if let Some(ref appendix) = historian_appendix {
        for pointer in &appendix.evidence_pointers {
            meta_builder = meta_builder.evidence(pointer.clone());
        }
    }

    // Add pointer to historian narrative if available
    if let Some(ref path) = historian_path {
        meta_builder = meta_builder.evidence(format!("analysis:{}", path.display()));
    }

    let receipt = builder
        .quality(Quality {
            contract: gov_receipts::Contract::default(),
            boundaries,
            verification,
            risks: Risks {
                unsafe_delta: Some(unsafe_delta),
                deps_added: vec![],
                concurrency_primitives_added: vec![],
                llm_risk_notes,
            },
        })
        .meta(meta_builder.build())
        .build();

    // Write receipt
    let quality_path = receipts_dir.join("quality.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&quality_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), quality_path.display());
    println!("  Modules touched: {}", modules_touched);
    println!("  Test LOC added: {}, Impl LOC added: {}", tests_loc, impl_loc);
    println!("  Test density delta: {:.2}", test_density_delta);
    println!("  Unsafe: +{} / -{}", unsafe_added, unsafe_removed);
    println!("  Files changed: {}", diff_stat.files_changed);

    if args.llm {
        if historian_appendix.is_some() {
            println!("  LLM: {} (historian appendix merged)", "enabled".green());
        } else {
            println!("  LLM: {} (no appendix)", "enabled".yellow());
        }
    }

    Ok(())
}

/// Parse boundary rating string to enum
fn parse_boundary_rating(s: &str) -> Option<gov_receipts::BoundaryRating> {
    match s.to_lowercase().as_str() {
        "improved" => Some(gov_receipts::BoundaryRating::Improved),
        "neutral" => Some(gov_receipts::BoundaryRating::Neutral),
        "degraded" => Some(gov_receipts::BoundaryRating::Degraded),
        _ => None,
    }
}

/// Parse test depth rating string to enum
fn parse_test_depth_rating(s: &str) -> Option<gov_receipts::TestDepthRating> {
    match s.to_lowercase().as_str() {
        "hardened" => Some(gov_receipts::TestDepthRating::Hardened),
        "mixed" => Some(gov_receipts::TestDepthRating::Mixed),
        "shallow" => Some(gov_receipts::TestDepthRating::Shallow),
        _ => None,
    }
}

// ============================================================================
// TELEMETRY RECEIPT
// ============================================================================

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

// ============================================================================
// TIMELINE RECEIPT
// ============================================================================

/// Arguments for the receipts-timeline command
#[derive(Debug, Clone)]
pub struct ReceiptsTimelineArgs {
    /// Pull request number (optional)
    pub pr: Option<u32>,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
    /// Session gap threshold in minutes (default: 30)
    pub session_gap_minutes: u32,
    /// Shared run_id (for forensic orchestration); generated if None
    pub run_id: Option<String>,
    /// Additional path prefixes to exclude from friction/oscillation analysis
    pub exclude_prefixes: Vec<String>,
    /// Include ephemeral directories in analysis (for debugging)
    pub include_ephemeral: bool,
}

impl Default for ReceiptsTimelineArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            base_branch: "origin/main".to_string(),
            session_gap_minutes: 30,
            run_id: None,
            exclude_prefixes: Vec::new(),
            include_ephemeral: false,
        }
    }
}

/// Arguments for the receipts-forensic command
#[derive(Debug, Clone)]
pub struct ReceiptsForensicArgs {
    /// Pull request number (required for forensic analysis)
    pub pr: u32,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
    /// Probe profile (fast/full/exhibit)
    pub profile: String,
    /// Session gap threshold in minutes (default: 30)
    pub session_gap_minutes: u32,
    /// Additional path prefixes to exclude from friction/oscillation analysis
    pub exclude_prefixes: Vec<String>,
    /// Include ephemeral directories in analysis (for debugging)
    pub include_ephemeral: bool,
    /// Enable LLM semantic analysis via Historian agent for quality receipt
    pub llm: bool,
    /// Path to existing historian output (for offline/testing use)
    pub historian_output: Option<PathBuf>,
    /// Command template for running historian (use {input} as placeholder)
    pub historian_cmd: Option<String>,
}

impl Default for ReceiptsForensicArgs {
    fn default() -> Self {
        Self {
            pr: 0,
            output_dir: PathBuf::from(".runs/current"),
            base_branch: "origin/main".to_string(),
            profile: "fast".to_string(),
            session_gap_minutes: 30,
            exclude_prefixes: Vec::new(),
            include_ephemeral: false,
            llm: false,
            historian_output: None,
            historian_cmd: None,
        }
    }
}

/// Generate timeline.json receipt from commit history
pub fn run_timeline(args: ReceiptsTimelineArgs) -> Result<()> {
    println!("{}", "Generating timeline receipt...".blue().bold());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    // Use provided run_id (for forensic orchestration) or generate new one
    let run_id = args.run_id.clone().unwrap_or_else(|| generate_run_id(args.pr));

    // Get commit history
    let commits = get_commit_history(&args.base_branch);

    if commits.is_empty() {
        anyhow::bail!("No commits found between HEAD and {}", args.base_branch);
    }

    // Build wall clock from commits
    let first_commit = commits.last().unwrap(); // oldest
    let last_commit = commits.first().unwrap(); // newest

    let wall_clock = WallClock {
        first_commit: first_commit.timestamp,
        last_commit: last_commit.timestamp,
        pr_created: None,
        pr_merged: None,
        total_duration_minutes: Some(
            ((last_commit.timestamp - first_commit.timestamp).num_minutes().max(0)) as u64,
        ),
    };

    // Identify sessions (clusters of commits within gap threshold)
    let sessions = identify_sessions(&commits, args.session_gap_minutes);

    // Find friction zones (files touched multiple times)
    let friction_zones =
        find_friction_zones(&args.base_branch, &args.exclude_prefixes, args.include_ephemeral);

    // Detect oscillations (add/remove/add patterns indicating uncertainty)
    let oscillations =
        detect_oscillations(&args.base_branch, &args.exclude_prefixes, args.include_ephemeral);

    // Detect convergence (how the PR stabilized toward completion)
    let convergence = detect_convergence(&args.base_branch, &commits);

    // Classify topology
    let (topology, confidence, reasons) = classify_topology(&commits, &friction_zones, &sessions);

    let mut builder = TimelineReceipt::builder()
        .run_id(&run_id)
        .wall_clock(wall_clock)
        .topology(topology)
        .topology_confidence(confidence)
        .sessions(sessions);

    for zone in friction_zones {
        builder = builder.friction_zone(zone);
    }

    for osc in oscillations.iter().cloned() {
        builder = builder.oscillation(osc);
    }

    for reason in reasons {
        builder = builder.topology_reason(reason);
    }

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

    if let Some(conv) = convergence {
        builder = builder.convergence(conv);
    }

    // Build meta provenance - timeline uses medium confidence by default
    // as it relies on git history analysis
    let mut meta_builder = ReceiptMeta::builder()
        .method_id("timeline-v1")
        .method_version(1)
        .analysis_run_id(&run_id)
        .input("git_log")
        .input("commit_history")
        .assumption(format!("session gap threshold: {} minutes", args.session_gap_minutes))
        .confidence(MetaConfidence::Medium);

    if let Some(commit) = get_current_commit_short() {
        meta_builder = meta_builder.evidence(commit);
    }

    // Add evidence pointers for commits analyzed
    for commit in commits.iter().take(5) {
        meta_builder = meta_builder.evidence(&commit.sha[..7.min(commit.sha.len())]);
    }

    builder = builder.meta(meta_builder.build());

    let receipt = builder.build();

    // Write receipt
    let timeline_path = receipts_dir.join("timeline.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&timeline_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), timeline_path.display());
    println!("  Commits: {}", commits.len());
    println!("  Sessions: {}", receipt.sessions.len());
    println!("  Friction zones: {}", receipt.friction_zones.len());

    // Show category rollup for friction zones
    if !receipt.friction_zones.is_empty() {
        let by_category = categorize_friction_zones(&receipt.friction_zones);
        let mut categories: Vec<_> = by_category.iter().collect();
        categories.sort_by_key(|(_, zones)| std::cmp::Reverse(zones.len()));
        print!("    Categories: ");
        for (i, (cat, zones)) in categories.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}={}", cat, zones.len());
        }
        println!();
    }

    println!("  Oscillations: {}", receipt.oscillations.len());
    println!("  Topology: {:?} ({:?} confidence)", receipt.topology, receipt.topology_confidence);
    if let Some(ref conv) = receipt.convergence {
        println!(
            "  Convergence: stable={}, categories={:?}",
            conv.last_n_commits_stable, conv.stable_categories
        );
    }

    Ok(())
}

// ============================================================================
// FORENSIC RECEIPT (orchestrates all emitters)
// ============================================================================

/// Run all receipt emitters in sequence for comprehensive PR forensics
///
/// This is the main entry point for generating a complete forensic receipt set.
/// It runs:
/// 1. telemetry - provides facts about the change surface
/// 2. timeline - analyzes git history for development patterns
/// 3. quality - computes code quality metrics
/// 4. validate - validates all receipts against schemas
pub fn run_forensic(args: ReceiptsForensicArgs) -> Result<()> {
    println!("{}", "Running forensic receipt generation...".blue().bold());
    println!();
    println!("  PR: #{}", args.pr);
    println!("  Profile: {}", args.profile);
    println!("  Base branch: {}", args.base_branch);
    println!("  Output: {}", args.output_dir.display());
    println!();

    // Create output directory structure
    std::fs::create_dir_all(&args.output_dir).with_context(|| {
        format!("Failed to create output directory: {}", args.output_dir.display())
    })?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir).with_context(|| {
        format!("Failed to create receipts directory: {}", receipts_dir.display())
    })?;

    // Generate a SHARED run_id for all receipts in this forensic run
    // This ensures all receipts from the same analysis can be correlated
    let shared_run_id = generate_run_id(Some(args.pr));
    println!("  Run ID: {}", shared_run_id);
    println!();

    // Track results for summary
    let mut results: Vec<(&str, bool, String)> = Vec::new();
    let start_time = Instant::now();

    // 1. Telemetry receipt (first, provides facts)
    println!("{}", "Step 1/4: Generating telemetry receipt...".cyan());
    let telemetry_result = run_telemetry(ReceiptsTelemetryArgs {
        pr: Some(args.pr),
        output_dir: args.output_dir.clone(),
        profile: args.profile.clone(),
        base_branch: args.base_branch.clone(),
        run_id: Some(shared_run_id.clone()),
    });
    match &telemetry_result {
        Ok(()) => results.push(("telemetry", true, "generated".to_string())),
        Err(e) => results.push(("telemetry", false, e.to_string())),
    }
    println!();

    // 2. Timeline receipt (uses git history)
    println!("{}", "Step 2/4: Generating timeline receipt...".cyan());
    let timeline_result = run_timeline(ReceiptsTimelineArgs {
        pr: Some(args.pr),
        output_dir: args.output_dir.clone(),
        base_branch: args.base_branch.clone(),
        session_gap_minutes: args.session_gap_minutes,
        run_id: Some(shared_run_id.clone()),
        exclude_prefixes: args.exclude_prefixes.clone(),
        include_ephemeral: args.include_ephemeral,
    });
    match &timeline_result {
        Ok(()) => results.push(("timeline", true, "generated".to_string())),
        Err(e) => results.push(("timeline", false, e.to_string())),
    }
    println!();

    // 3. Quality receipt (can use telemetry data, optionally with LLM analysis)
    let llm_note = if args.llm { " (with LLM)" } else { "" };
    println!("{}", format!("Step 3/4: Generating quality receipt{}...", llm_note).cyan());
    let quality_result = run_quality(ReceiptsQualityArgs {
        pr: Some(args.pr),
        output_dir: args.output_dir.clone(),
        base_branch: args.base_branch.clone(),
        boundary_rating: None,
        test_depth_rating: None,
        notes: vec![],
        run_id: Some(shared_run_id.clone()),
        llm: args.llm,
        historian_output: args.historian_output.clone(),
        historian_cmd: args.historian_cmd.clone(),
    });
    match &quality_result {
        Ok(()) => results.push(("quality", true, "generated".to_string())),
        Err(e) => results.push(("quality", false, e.to_string())),
    }
    println!();

    // 4. Validate all receipts
    println!("{}", "Step 4/4: Validating receipts against schemas...".cyan());
    let validate_result = run_validate(ReceiptsValidateArgs {
        run_dir: args.output_dir.clone(),
        schema_dir: PathBuf::from("specs/schemas"),
    });
    match &validate_result {
        Ok(()) => results.push(("validate", true, "all valid".to_string())),
        Err(e) => results.push(("validate", false, e.to_string())),
    }

    let elapsed = start_time.elapsed();

    // Print summary
    println!();
    println!("{}", "=".repeat(60));
    println!("{}", "Forensic Receipt Summary".blue().bold());
    println!("{}", "=".repeat(60));
    println!();

    let mut pass_count = 0;
    let mut fail_count = 0;

    for (name, passed, detail) in &results {
        if *passed {
            pass_count += 1;
            println!("  {} {}: {}", "PASS".green(), name, detail.dimmed());
        } else {
            fail_count += 1;
            println!("  {} {}: {}", "FAIL".red(), name, detail);
        }
    }

    println!();
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  Output directory: {}", args.output_dir.display());
    println!();

    // List generated receipts
    if receipts_dir.exists() {
        let mut receipts: Vec<_> = std::fs::read_dir(&receipts_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        receipts.sort();

        if !receipts.is_empty() {
            println!("  Generated receipts:");
            for receipt in &receipts {
                println!("    - {}", receipt);
            }
            println!();
        }
    }

    if fail_count > 0 {
        println!(
            "{} {} passed, {} failed",
            "Summary:".bold(),
            pass_count.to_string().green(),
            fail_count.to_string().red()
        );
        anyhow::bail!("{} step(s) failed during forensic receipt generation", fail_count);
    }

    println!("{} All {} steps completed successfully", "OK".green().bold(), pass_count);

    Ok(())
}

// ============================================================================
// GIT HELPERS
// ============================================================================

/// Diff statistics
struct DiffStat {
    files_changed: u32,
    insertions: u32,
    deletions: u32,
}

/// Get diff statistics from git
fn get_diff_stat(base_branch: &str) -> DiffStat {
    let output = Command::new("git").args(["diff", "--stat", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_diff_stat(&stdout)
        }
        Err(_) => DiffStat { files_changed: 0, insertions: 0, deletions: 0 },
    }
}

/// Parse git diff --stat output
fn parse_diff_stat(output: &str) -> DiffStat {
    let mut files_changed = 0u32;
    let mut insertions = 0u32;
    let mut deletions = 0u32;

    // Look for the summary line like "5 files changed, 100 insertions(+), 50 deletions(-)"
    for line in output.lines() {
        if line.contains("changed") && (line.contains("insertion") || line.contains("deletion")) {
            let parts: Vec<&str> = line.split(',').collect();
            for part in parts {
                let part = part.trim();
                if part.contains("file")
                    && let Some(num) = part.split_whitespace().next()
                {
                    files_changed = num.parse().unwrap_or(0);
                } else if part.contains("insertion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    insertions = num.parse().unwrap_or(0);
                } else if part.contains("deletion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    deletions = num.parse().unwrap_or(0);
                }
            }
        }
    }

    DiffStat { files_changed, insertions, deletions }
}

/// Count distinct modules (top-level directories) touched
fn count_modules_touched(base_branch: &str) -> u32 {
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let modules: std::collections::HashSet<_> =
                stdout.lines().filter_map(|line| line.split('/').next()).collect();
            modules.len() as u32
        }
        Err(_) => 0,
    }
}

/// Get module names touched
fn get_modules_touched_names(base_branch: &str) -> Vec<String> {
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let modules: std::collections::HashSet<_> = stdout
                .lines()
                .filter_map(|line| line.split('/').next())
                .map(|s| s.to_string())
                .collect();
            modules.into_iter().collect()
        }
        Err(_) => vec![],
    }
}

/// Get crate names touched
fn get_crates_touched(base_branch: &str) -> Vec<String> {
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let crates: std::collections::HashSet<_> = stdout
                .lines()
                .filter(|line| line.starts_with("crates/"))
                .filter_map(|line| line.split('/').nth(1))
                .map(|s| s.to_string())
                .collect();
            crates.into_iter().collect()
        }
        Err(_) => vec![],
    }
}

/// Find files with high churn (touched multiple times)
fn find_hotspots(base_branch: &str) -> Vec<String> {
    // For v0, we just return files changed - hotspot detection needs commit history analysis
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|line| line.ends_with(".rs"))
                .take(5) // Top 5 files
                .map(|s| s.to_string())
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Count unsafe blocks added/removed
fn count_unsafe_delta(base_branch: &str) -> UnsafeDelta {
    let output = Command::new("git").args(["diff", "-U0", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut added = 0u32;
            let mut removed = 0u32;

            for line in stdout.lines() {
                if line.starts_with('+') && !line.starts_with("+++") && line.contains("unsafe") {
                    added += 1;
                } else if line.starts_with('-')
                    && !line.starts_with("---")
                    && line.contains("unsafe")
                {
                    removed += 1;
                }
            }

            UnsafeDelta { added, removed }
        }
        Err(_) => UnsafeDelta::default(),
    }
}

/// Count lines of code changes split by test vs impl
fn count_loc_changes(base_branch: &str) -> (u32, u32) {
    let output = Command::new("git").args(["diff", "--numstat", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut tests_loc = 0u32;
            let mut impl_loc = 0u32;

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let added: u32 = parts[0].parse().unwrap_or(0);
                    let file = parts[2];

                    // Categorize by file path
                    if file.contains("/tests/")
                        || file.contains("_test.rs")
                        || file.ends_with("/mod.rs") && file.contains("tests")
                    {
                        tests_loc += added;
                    } else if file.ends_with(".rs") {
                        // Check if the file has #[cfg(test)] or #[test] markers
                        // For simplicity, assume files in src/ are impl, tests/ are tests
                        if file.contains("src/") {
                            impl_loc += added;
                        } else {
                            tests_loc += added;
                        }
                    }
                }
            }

            (tests_loc, impl_loc)
        }
        Err(_) => (0, 0),
    }
}

/// Detect contract changes from git diff
fn detect_contract_changes(base_branch: &str) -> Contracts {
    let output = Command::new("git").args(["diff", "--name-only", base_branch]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let files: Vec<&str> = stdout.lines().collect();

            let schema_changed =
                files.iter().any(|f| f.ends_with(".schema.json") || f.contains("specs/schemas/"));

            let cli_changed =
                files.iter().any(|f| f.contains("xtask/src/main.rs") || f.contains("cli/"));

            // Public API detection is complex - for v0, check if lib.rs changed
            let public_api_changed =
                files.iter().any(|f| f.ends_with("lib.rs") && f.contains("crates/"));

            Contracts {
                schema_changed,
                public_api_changed,
                cli_changed,
                breaking: false, // Would need semver analysis
                diff_pointers: vec![],
            }
        }
        Err(_) => Contracts::default(),
    }
}

/// Commit information
struct CommitInfo {
    sha: String,
    timestamp: chrono::DateTime<Utc>,
    #[allow(dead_code)]
    author: String,
}

/// File category for convergence detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileCategory {
    /// Test files: **/tests/**, *_test.rs
    Tests,
    /// Documentation: docs/**, *.md
    Docs,
    /// Receipt files: .runs/**
    Receipts,
    /// Config files: *.toml, *.yaml, *.json in root or specs/
    Config,
    /// Implementation files
    Impl,
}

impl FileCategory {
    /// Categorize a file path
    fn from_path(path: &str) -> Self {
        // Tests
        if path.contains("/tests/")
            || path.ends_with("_test.rs")
            || path.contains("tests.rs")
            || (path.contains("/mod.rs") && path.contains("tests"))
        {
            return FileCategory::Tests;
        }

        // Docs
        if path.starts_with("docs/") || path.ends_with(".md") {
            return FileCategory::Docs;
        }

        // Receipts
        if path.starts_with(".runs/") {
            return FileCategory::Receipts;
        }

        // Config
        if (path.ends_with(".toml") || path.ends_with(".yaml") || path.ends_with(".json"))
            && (path.starts_with("specs/")
                || !path.contains('/')
                || path.starts_with(".claude/")
                || path.starts_with(".llm/"))
        {
            return FileCategory::Config;
        }

        FileCategory::Impl
    }

    /// Check if this category is "stable" (non-implementation)
    fn is_stable(&self) -> bool {
        matches!(
            self,
            FileCategory::Tests
                | FileCategory::Docs
                | FileCategory::Receipts
                | FileCategory::Config
        )
    }

    /// Convert to string representation for receipt output
    fn as_str(&self) -> &'static str {
        match self {
            FileCategory::Tests => "tests",
            FileCategory::Docs => "docs",
            FileCategory::Receipts => "receipts",
            FileCategory::Config => "config",
            FileCategory::Impl => "impl",
        }
    }
}

/// Get commit history between HEAD and base branch
fn get_commit_history(base_branch: &str) -> Vec<CommitInfo> {
    let output = Command::new("git")
        .args(["log", "--format=%H|%aI|%an", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        let timestamp = chrono::DateTime::parse_from_rfc3339(parts[1])
                            .ok()?
                            .with_timezone(&Utc);
                        Some(CommitInfo {
                            sha: parts[0].to_string(),
                            timestamp,
                            author: parts[2].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Get files changed in a specific commit
fn get_files_changed_in_commit(sha: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", sha])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().map(|s| s.to_string()).collect()
        }
        Err(_) => vec![],
    }
}

/// Commit change summary for convergence analysis
struct CommitChangeSummary {
    sha: String,
    file_count: usize,
    categories: std::collections::HashSet<FileCategory>,
    is_stable: bool,
}

/// Detect convergence in the commit history
///
/// Convergence detection looks for:
/// 1. An inflection point where churn collapsed (last commit touching many impl files)
/// 2. Whether final N commits only touch stable categories (tests, docs, receipts, config)
/// 3. What stable categories the final commits touched
fn detect_convergence(_base_branch: &str, commits: &[CommitInfo]) -> Option<Convergence> {
    if commits.is_empty() {
        return None;
    }

    // Number of final commits to check for stability
    const STABLE_WINDOW: usize = 3;

    // Analyze each commit's file changes
    // Commits are ordered newest-first from get_commit_history
    let mut summaries: Vec<CommitChangeSummary> = Vec::with_capacity(commits.len());

    for commit in commits {
        let files = get_files_changed_in_commit(&commit.sha);
        let categories: std::collections::HashSet<FileCategory> =
            files.iter().map(|f| FileCategory::from_path(f)).collect();

        let is_stable = !categories.is_empty() && categories.iter().all(|c| c.is_stable());

        summaries.push(CommitChangeSummary {
            sha: commit.sha.clone(),
            file_count: files.len(),
            categories,
            is_stable,
        });
    }

    if summaries.is_empty() {
        return None;
    }

    // Check if last N commits are stable
    let window_size = std::cmp::min(STABLE_WINDOW, summaries.len());
    let final_commits = &summaries[..window_size];
    let last_n_commits_stable = final_commits.iter().all(|s| s.is_stable);

    // Collect stable categories from final commits
    let mut stable_categories_set: std::collections::HashSet<&str> =
        std::collections::HashSet::new();
    if last_n_commits_stable {
        for summary in final_commits {
            for cat in &summary.categories {
                if cat.is_stable() {
                    stable_categories_set.insert(cat.as_str());
                }
            }
        }
    }

    let stable_categories: Vec<String> =
        stable_categories_set.into_iter().map(String::from).collect();

    // Find inflection point: last commit that touched impl files before stabilization
    // Walk from newest to oldest, find where impl work stopped
    let mut inflection_commit: Option<String> = None;

    // If all commits are stable, no inflection point
    if !summaries.iter().all(|s| s.is_stable) {
        // Find the newest commit that touched impl
        // Then the inflection is the commit right before the stable tail started

        // First, find where the stable tail begins (from newest)
        let stable_tail_start =
            summaries.iter().position(|s| !s.is_stable).unwrap_or(summaries.len());

        // If there's a stable tail and impl commits before it, the inflection is the first impl
        // commit
        if stable_tail_start > 0 && stable_tail_start < summaries.len() {
            // The inflection is the commit at stable_tail_start (first non-stable from newest)
            inflection_commit = Some(summaries[stable_tail_start].sha.clone());
        } else if stable_tail_start == 0 {
            // No stable tail at all - look for where file count dropped
            // Find the commit with the most files as a potential inflection
            if let Some((idx, _)) = summaries
                .iter()
                .enumerate()
                .filter(|(_, s)| s.file_count > 0)
                .max_by_key(|(_, s)| s.file_count)
            {
                // The inflection is where exploration peaked
                inflection_commit = Some(summaries[idx].sha.clone());
            }
        }
    }

    // Only return convergence if there's meaningful data
    if inflection_commit.is_some() || last_n_commits_stable || !stable_categories.is_empty() {
        Some(Convergence { inflection_commit, last_n_commits_stable, stable_categories })
    } else {
        None
    }
}

/// Identify sessions from commit history
fn identify_sessions(commits: &[CommitInfo], gap_minutes: u32) -> Vec<Session> {
    if commits.is_empty() {
        return vec![];
    }

    let mut sessions = Vec::new();
    let mut session_start = commits.last().unwrap().timestamp; // oldest first
    let mut session_end = session_start;
    let mut commit_count = 0u32;

    // Process commits from oldest to newest
    for commit in commits.iter().rev() {
        let gap = (commit.timestamp - session_end).num_minutes();

        if gap > gap_minutes as i64 && commit_count > 0 {
            // End current session, start new one
            sessions.push(Session {
                start: session_start,
                end: session_end,
                commit_count,
                classification: classify_session(
                    commit_count,
                    (session_end - session_start).num_minutes(),
                ),
            });
            session_start = commit.timestamp;
            commit_count = 0;
        }

        session_end = commit.timestamp;
        commit_count += 1;
    }

    // Don't forget the last session
    if commit_count > 0 {
        sessions.push(Session {
            start: session_start,
            end: session_end,
            commit_count,
            classification: classify_session(
                commit_count,
                (session_end - session_start).num_minutes(),
            ),
        });
    }

    sessions
}

/// Classify a session based on commit frequency
fn classify_session(commit_count: u32, duration_minutes: i64) -> Option<SessionClassification> {
    if duration_minutes <= 0 {
        return Some(SessionClassification::Mixed);
    }

    let commits_per_hour = (commit_count as f64 / duration_minutes as f64) * 60.0;

    if commits_per_hour > 10.0 {
        Some(SessionClassification::MachineGrind)
    } else if commits_per_hour < 2.0 {
        Some(SessionClassification::HumanWork)
    } else {
        Some(SessionClassification::Mixed)
    }
}

/// Find friction zones (files touched in multiple commits)
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `extra_excludes` - Additional path prefixes to exclude
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
fn find_friction_zones(
    base_branch: &str,
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<FrictionZone> {
    let output = Command::new("git")
        .args(["log", "--name-only", "--pretty=format:%H", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut file_commits: HashMap<String, Vec<String>> = HashMap::new();
            let mut current_commit = String::new();

            for line in stdout.lines() {
                if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
                    current_commit = line.to_string();
                } else if !line.is_empty() && !current_commit.is_empty() {
                    file_commits.entry(line.to_string()).or_default().push(current_commit.clone());
                }
            }

            // Filter to files touched 2+ times, excluding ephemeral directories
            file_commits
                .into_iter()
                .filter(|(path, commits)| {
                    !should_exclude_path(path, extra_excludes, include_ephemeral)
                        && commits.len() >= 2
                })
                .map(|(path, commits)| FrictionZone {
                    path,
                    touch_count: commits.len() as u32,
                    commits: commits.into_iter().take(5).collect(), // Limit to 5 commits
                })
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Classify overall development topology
fn classify_topology(
    commits: &[CommitInfo],
    friction_zones: &[FrictionZone],
    sessions: &[Session],
) -> (Topology, TimelineConfidence, Vec<String>) {
    let mut reasons = Vec::new();
    let commit_count = commits.len();
    let friction_count = friction_zones.len();
    let high_friction_files = friction_zones.iter().filter(|z| z.touch_count >= 3).count();

    // Heuristics for topology classification
    let has_machine_sessions =
        sessions.iter().any(|s| s.classification == Some(SessionClassification::MachineGrind));

    if friction_count == 0 && commit_count <= 5 {
        reasons.push("Clean progression with minimal commits".to_string());
        return (Topology::Linear, TimelineConfidence::High, reasons);
    }

    if high_friction_files >= 3 || friction_count > commit_count / 2 {
        reasons.push(format!("{} high-friction files detected", high_friction_files));
        reasons.push("Multiple files touched repeatedly across commits".to_string());
        return (Topology::Chaotic, TimelineConfidence::Medium, reasons);
    }

    if friction_count > 0 || has_machine_sessions {
        reasons.push(format!(
            "{} friction zones, {} machine sessions",
            friction_count,
            sessions
                .iter()
                .filter(|s| s.classification == Some(SessionClassification::MachineGrind))
                .count()
        ));
        return (Topology::Cyclical, TimelineConfidence::Medium, reasons);
    }

    reasons.push("Steady commit progression".to_string());
    (Topology::Linear, TimelineConfidence::Low, reasons)
}

// ============================================================================
// OSCILLATION DETECTION
// ============================================================================

/// Represents an action on a subject (file or dependency) in a commit
#[derive(Debug, Clone, PartialEq, Eq)]
enum SubjectAction {
    Add,
    Remove,
}

/// Track per-commit changes for a subject
#[derive(Debug, Clone)]
struct SubjectHistory {
    actions: Vec<(String, SubjectAction)>, // (commit_sha, action)
}

impl SubjectHistory {
    fn new() -> Self {
        Self { actions: Vec::new() }
    }

    fn push(&mut self, commit: String, action: SubjectAction) {
        self.actions.push((commit, action));
    }

    /// Check if this history shows an oscillation pattern (Add -> Remove or Remove -> Add -> Remove, etc.)
    /// Returns the sequence of actions if it's an oscillation (2+ alternating actions)
    fn to_oscillation_sequence(&self) -> Option<Vec<OscillationAction>> {
        if self.actions.len() < 2 {
            return None;
        }

        // Convert to oscillation actions and check for alternation
        let mut sequence = Vec::new();
        let mut prev_action: Option<&SubjectAction> = None;
        let mut has_alternation = false;

        for (_, action) in &self.actions {
            let osc_action = match action {
                SubjectAction::Add => OscillationAction::Add,
                SubjectAction::Remove => OscillationAction::Remove,
            };

            // Check if this alternates from previous
            if let Some(prev) = prev_action
                && prev != action
            {
                has_alternation = true;
            }

            sequence.push(osc_action);
            prev_action = Some(action);
        }

        if has_alternation { Some(sequence) } else { None }
    }
}

/// Detect oscillations in the commit history between base_branch and HEAD
///
/// This function detects:
/// - Dependency oscillations: deps added then removed (or vice versa)
/// - File oscillations: files created then deleted (or vice versa)
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `extra_excludes` - Additional path prefixes to exclude from file oscillations
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
pub fn detect_oscillations(
    base_branch: &str,
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<Oscillation> {
    let mut oscillations = Vec::new();

    // Get commit list from oldest to newest
    let commits = get_commit_shas_oldest_first(base_branch);
    if commits.len() < 2 {
        return oscillations;
    }

    // Detect dependency oscillations
    let dep_oscillations = detect_dependency_oscillations(base_branch, &commits);
    oscillations.extend(dep_oscillations);

    // Detect file oscillations
    let file_oscillations =
        detect_file_oscillations(base_branch, &commits, extra_excludes, include_ephemeral);
    oscillations.extend(file_oscillations);

    oscillations
}

/// Get commit SHAs from oldest to newest
fn get_commit_shas_oldest_first(base_branch: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--format=%H", "--reverse", &format!("{}..HEAD", base_branch)])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().map(|s| s.to_string()).collect()
        }
        Err(_) => vec![],
    }
}

/// Detect dependency oscillations by parsing Cargo.toml changes across commits
fn detect_dependency_oscillations(base_branch: &str, commits: &[String]) -> Vec<Oscillation> {
    let mut dep_history: HashMap<String, SubjectHistory> = HashMap::new();

    // For each commit, get the diff of Cargo.toml files and track dependency changes
    for (i, commit) in commits.iter().enumerate() {
        let parent = if i == 0 { base_branch.to_string() } else { commits[i - 1].clone() };

        // Get diff for Cargo.toml files in this commit
        let output = Command::new("git")
            .args(["diff", "-U0", &parent, commit, "--", "**/Cargo.toml", "Cargo.toml"])
            .output();

        if let Ok(out) = output {
            let diff = String::from_utf8_lossy(&out.stdout);
            parse_cargo_toml_diff(&diff, commit, &mut dep_history);
        }
    }

    // Convert histories to oscillations
    dep_history
        .into_iter()
        .filter_map(|(dep_name, history)| {
            history.to_oscillation_sequence().map(|sequence| Oscillation {
                oscillation_type: OscillationType::Dependency,
                subject: dep_name,
                sequence,
            })
        })
        .collect()
}

/// Parse a Cargo.toml diff to extract dependency additions/removals
fn parse_cargo_toml_diff(diff: &str, commit: &str, history: &mut HashMap<String, SubjectHistory>) {
    // We're looking for lines like:
    // +serde = "1.0"
    // -serde = "1.0"
    // +serde = { version = "1.0", features = ["derive"] }
    // -serde = { version = "1.0", features = ["derive"] }

    let mut in_dependencies_section = false;

    for line in diff.lines() {
        // Track if we're entering a dependencies section
        if line.contains("[dependencies]")
            || line.contains("[dev-dependencies]")
            || line.contains("[build-dependencies]")
            || line.contains(".dependencies]")
        {
            in_dependencies_section = true;
            continue;
        }

        // Exit dependency section on new section
        if line.starts_with('+') || line.starts_with('-') {
            let content = &line[1..];
            if content.starts_with('[') && !content.contains("dependencies") {
                in_dependencies_section = false;
                continue;
            }
        }

        // Skip non-diff lines and header lines
        if !line.starts_with('+') && !line.starts_with('-') {
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }

        // Parse dependency line
        let is_add = line.starts_with('+');
        let content = &line[1..].trim();

        // Skip empty lines and section headers
        if content.is_empty() || content.starts_with('[') {
            continue;
        }

        // Extract dependency name - it's the part before '=' or '.'
        // e.g., "serde = ..." or "serde.workspace = true"
        if let Some(dep_name) = extract_dependency_name(content) {
            // Only track if we think we're in a dependencies section
            // or the line looks like a dependency declaration
            if in_dependencies_section || looks_like_dependency_line(content) {
                let entry = history.entry(dep_name).or_insert_with(SubjectHistory::new);
                let action = if is_add { SubjectAction::Add } else { SubjectAction::Remove };
                entry.push(commit.to_string(), action);
            }
        }
    }
}

/// Extract dependency name from a Cargo.toml dependency line
fn extract_dependency_name(line: &str) -> Option<String> {
    // Handle "dep = ..." or "dep.feature = ..."
    let line = line.trim();

    // Skip if this doesn't look like a dep line
    if !line.contains('=') {
        return None;
    }

    // Get the part before '='
    let before_eq = line.split('=').next()?.trim();

    // Handle "dep.workspace" -> "dep"
    let name = before_eq.split('.').next()?.trim();

    // Validate it looks like a crate name (alphanumeric, _, -)
    if name.is_empty() {
        return None;
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return None;
    }

    // Skip common non-dependency keys
    let skip_keys = [
        "version",
        "edition",
        "name",
        "authors",
        "description",
        "license",
        "repository",
        "readme",
        "keywords",
        "categories",
        "workspace",
        "package",
        "path",
        "features",
        "default-features",
        "optional",
        "default",
        "members",
        "exclude",
        "include",
        "resolver",
        "rust-version",
    ];
    if skip_keys.contains(&name) {
        return None;
    }

    Some(name.to_string())
}

/// Check if a line looks like a dependency declaration
fn looks_like_dependency_line(line: &str) -> bool {
    let line = line.trim();
    // Dependency lines typically have format: name = "version" or name = { ... }
    if let Some(after_eq) = line.split('=').nth(1) {
        let after_eq = after_eq.trim();
        // Version string or table
        after_eq.starts_with('"') || after_eq.starts_with('{') || after_eq == "true"
    } else {
        false
    }
}

/// Detect file oscillations by tracking file additions/deletions across commits
///
/// # Arguments
/// * `base_branch` - Base branch for comparison
/// * `commits` - List of commit SHAs
/// * `extra_excludes` - Additional path prefixes to exclude
/// * `include_ephemeral` - If true, include ephemeral directories (for debugging)
fn detect_file_oscillations(
    base_branch: &str,
    commits: &[String],
    extra_excludes: &[String],
    include_ephemeral: bool,
) -> Vec<Oscillation> {
    let mut file_history: HashMap<String, SubjectHistory> = HashMap::new();

    for (i, commit) in commits.iter().enumerate() {
        let parent = if i == 0 { base_branch.to_string() } else { commits[i - 1].clone() };

        // Get list of added and deleted files in this commit
        let output = Command::new("git")
            .args(["diff", "--name-status", "--diff-filter=AD", &parent, commit])
            .output();

        if let Ok(out) = output {
            let diff = String::from_utf8_lossy(&out.stdout);
            for line in diff.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let status = parts[0];
                    let file_path = parts[1];

                    // Skip ephemeral directories
                    if should_exclude_path(file_path, extra_excludes, include_ephemeral) {
                        continue;
                    }

                    let action = match status {
                        "A" => SubjectAction::Add,
                        "D" => SubjectAction::Remove,
                        _ => continue,
                    };

                    let entry = file_history
                        .entry(file_path.to_string())
                        .or_insert_with(SubjectHistory::new);
                    entry.push(commit.to_string(), action);
                }
            }
        }
    }

    // Convert histories to oscillations (ephemeral paths already filtered above)
    file_history
        .into_iter()
        .filter_map(|(file_path, history)| {
            history.to_oscillation_sequence().map(|sequence| Oscillation {
                oscillation_type: OscillationType::File,
                subject: file_path,
                sequence,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gov_receipts::{Environment, GateReceipt, GateResult, GateStatus};
    use xtask_receipts::normalize_path_separators;

    #[test]
    fn gate_receipt_uses_gov_receipts_types() {
        // Verify we're using the gov-receipts crate types
        let receipt = GateReceipt::builder()
            .run_id("test-run")
            .commit("abc123")
            .started_at(Utc::now())
            .finished_at(Utc::now())
            .gate(GateResult {
                name: "fmt".to_string(),
                command: "cargo fmt --all --check".to_string(),
                status: GateStatus::Pass,
                duration_ms: 1234,
                details: None,
            })
            .overall_status(GateStatus::Pass)
            .repo_version("3.3.14")
            .environment(Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: true,
            })
            .build();

        assert!(receipt.all_passed());
        assert_eq!(receipt.run_id, "test-run");
    }

    #[test]
    fn gate_receipt_optional_pr() {
        let receipt = GateReceipt::builder()
            .run_id("test-run")
            .pr(123)
            .commit("abc123")
            .started_at(Utc::now())
            .finished_at(Utc::now())
            .overall_status(GateStatus::Pass)
            .repo_version("3.3.14")
            .environment(Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: false,
            })
            .build();

        assert_eq!(receipt.pr, Some(123));

        // Verify JSON serialization includes pr
        let json = serde_json::to_string(&receipt).unwrap();
        assert!(json.contains(r#""pr":123"#));
    }

    #[test]
    fn gate_status_serialization() {
        assert_eq!(serde_json::to_string(&GateStatus::Pass).unwrap(), r#""pass""#);
        assert_eq!(serde_json::to_string(&GateStatus::Fail).unwrap(), r#""fail""#);
        assert_eq!(serde_json::to_string(&GateStatus::Skipped).unwrap(), r#""skipped""#);
    }

    // ========================================================================
    // OSCILLATION DETECTION TESTS
    // ========================================================================

    #[test]
    fn subject_history_no_oscillation_single_action() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);

        assert!(history.to_oscillation_sequence().is_none());
    }

    #[test]
    fn subject_history_no_oscillation_same_actions() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Add);

        assert!(history.to_oscillation_sequence().is_none());
    }

    #[test]
    fn subject_history_detects_add_remove_oscillation() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Remove);

        let sequence = history.to_oscillation_sequence().expect("should detect oscillation");
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0], OscillationAction::Add);
        assert_eq!(sequence[1], OscillationAction::Remove);
    }

    #[test]
    fn subject_history_detects_add_remove_add_oscillation() {
        let mut history = SubjectHistory::new();
        history.push("commit1".to_string(), SubjectAction::Add);
        history.push("commit2".to_string(), SubjectAction::Remove);
        history.push("commit3".to_string(), SubjectAction::Add);

        let sequence = history.to_oscillation_sequence().expect("should detect oscillation");
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence[0], OscillationAction::Add);
        assert_eq!(sequence[1], OscillationAction::Remove);
        assert_eq!(sequence[2], OscillationAction::Add);
    }

    #[test]
    fn extract_dependency_name_simple() {
        assert_eq!(extract_dependency_name("serde = \"1.0\""), Some("serde".to_string()));
        assert_eq!(
            extract_dependency_name("tokio = { version = \"1.0\" }"),
            Some("tokio".to_string())
        );
    }

    #[test]
    fn extract_dependency_name_workspace() {
        assert_eq!(extract_dependency_name("serde.workspace = true"), Some("serde".to_string()));
    }

    #[test]
    fn extract_dependency_name_skips_metadata() {
        assert_eq!(extract_dependency_name("version = \"0.1.0\""), None);
        assert_eq!(extract_dependency_name("edition = \"2021\""), None);
        assert_eq!(extract_dependency_name("name = \"my-crate\""), None);
    }

    #[test]
    fn extract_dependency_name_invalid() {
        assert_eq!(extract_dependency_name(""), None);
        assert_eq!(extract_dependency_name("no-equals-sign"), None);
        assert_eq!(extract_dependency_name("[dependencies]"), None);
    }

    #[test]
    fn looks_like_dependency_line_valid() {
        assert!(looks_like_dependency_line("serde = \"1.0\""));
        assert!(looks_like_dependency_line("tokio = { version = \"1.0\" }"));
        assert!(looks_like_dependency_line("serde.workspace = true"));
    }

    #[test]
    fn looks_like_dependency_line_invalid() {
        assert!(!looks_like_dependency_line("[dependencies]"));
        assert!(!looks_like_dependency_line(""));
        assert!(!looks_like_dependency_line("no-equals"));
    }

    #[test]
    fn parse_cargo_toml_diff_extracts_deps() {
        let diff = r#"
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -10,0 +11,2 @@
+[dependencies]
+serde = "1.0"
"#;
        let mut history = HashMap::new();
        parse_cargo_toml_diff(diff, "abc123", &mut history);

        assert!(history.contains_key("serde"));
        assert_eq!(history["serde"].actions.len(), 1);
        assert_eq!(history["serde"].actions[0].1, SubjectAction::Add);
    }

    #[test]
    fn parse_cargo_toml_diff_tracks_removal() {
        let diff = r#"
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -10,2 +10,0 @@
-[dependencies]
-serde = "1.0"
"#;
        let mut history = HashMap::new();
        parse_cargo_toml_diff(diff, "abc123", &mut history);

        assert!(history.contains_key("serde"));
        assert_eq!(history["serde"].actions.len(), 1);
        assert_eq!(history["serde"].actions[0].1, SubjectAction::Remove);
    }

    // ========================================================================
    // FILE CATEGORY TESTS
    // ========================================================================

    #[test]
    fn file_category_tests_detection() {
        assert_eq!(FileCategory::from_path("crates/foo/tests/integration.rs"), FileCategory::Tests);
        assert_eq!(FileCategory::from_path("src/lib_test.rs"), FileCategory::Tests);
        assert_eq!(FileCategory::from_path("crates/x/src/tests.rs"), FileCategory::Tests);
    }

    #[test]
    fn file_category_docs_detection() {
        assert_eq!(FileCategory::from_path("docs/README.md"), FileCategory::Docs);
        assert_eq!(FileCategory::from_path("CHANGELOG.md"), FileCategory::Docs);
        assert_eq!(FileCategory::from_path("docs/api/overview.md"), FileCategory::Docs);
    }

    #[test]
    fn file_category_receipts_detection() {
        assert_eq!(
            FileCategory::from_path(".runs/current/receipts/gate.json"),
            FileCategory::Receipts
        );
        assert_eq!(
            FileCategory::from_path(".runs/pr123/receipts/timeline.json"),
            FileCategory::Receipts
        );
    }

    #[test]
    fn file_category_config_detection() {
        assert_eq!(FileCategory::from_path("Cargo.toml"), FileCategory::Config);
        assert_eq!(FileCategory::from_path("specs/config.yaml"), FileCategory::Config);
        assert_eq!(FileCategory::from_path(".claude/settings.json"), FileCategory::Config);
    }

    #[test]
    fn file_category_impl_detection() {
        assert_eq!(FileCategory::from_path("crates/foo/src/lib.rs"), FileCategory::Impl);
        assert_eq!(FileCategory::from_path("src/main.rs"), FileCategory::Impl);
    }

    #[test]
    fn file_category_is_stable() {
        assert!(FileCategory::Tests.is_stable());
        assert!(FileCategory::Docs.is_stable());
        assert!(FileCategory::Receipts.is_stable());
        assert!(FileCategory::Config.is_stable());
        assert!(!FileCategory::Impl.is_stable());
    }

    #[test]
    fn file_category_as_str() {
        assert_eq!(FileCategory::Tests.as_str(), "tests");
        assert_eq!(FileCategory::Docs.as_str(), "docs");
        assert_eq!(FileCategory::Receipts.as_str(), "receipts");
        assert_eq!(FileCategory::Config.as_str(), "config");
        assert_eq!(FileCategory::Impl.as_str(), "impl");
    }

    // ========================================================================
    // Path Normalization Tests
    // ========================================================================

    #[test]
    fn normalize_path_separators_converts_backslashes() {
        // Windows-style paths
        assert_eq!(normalize_path_separators(r"crates\xtask\src"), "crates/xtask/src");
        assert_eq!(normalize_path_separators(r".runs\pr\123"), ".runs/pr/123");
        assert_eq!(normalize_path_separators(r"target\debug\foo"), "target/debug/foo");
    }

    #[test]
    fn normalize_path_separators_preserves_forward_slashes() {
        // Unix-style paths remain unchanged
        assert_eq!(normalize_path_separators("crates/xtask/src"), "crates/xtask/src");
        assert_eq!(normalize_path_separators(".runs/pr/123"), ".runs/pr/123");
    }

    #[test]
    fn should_exclude_path_with_defaults() {
        // Default exclusions with empty extra excludes
        let empty_excludes: Vec<String> = vec![];

        assert!(should_exclude_path(".runs/current/foo.json", &empty_excludes, false));
        assert!(should_exclude_path("target/debug/foo", &empty_excludes, false));
        assert!(should_exclude_path(".git/objects/abc", &empty_excludes, false));

        // Non-excluded paths
        assert!(!should_exclude_path("src/main.rs", &empty_excludes, false));
        assert!(!should_exclude_path("crates/foo/src/lib.rs", &empty_excludes, false));
    }

    #[test]
    fn should_exclude_path_with_windows_separators() {
        let empty_excludes: Vec<String> = vec![];

        // Windows-style .runs path should be excluded
        assert!(should_exclude_path(r".runs\current\foo.json", &empty_excludes, false));
        assert!(should_exclude_path(r"target\debug\foo", &empty_excludes, false));
    }

    #[test]
    fn should_exclude_path_with_extra_excludes() {
        let extra_excludes = vec!["vendor/".to_string(), "custom_dir/".to_string()];

        // Extra exclusions
        assert!(should_exclude_path("vendor/crate/src", &extra_excludes, false));
        assert!(should_exclude_path("custom_dir/file.txt", &extra_excludes, false));

        // Non-excluded
        assert!(!should_exclude_path("src/main.rs", &extra_excludes, false));
    }

    #[test]
    fn should_exclude_path_include_ephemeral_flag() {
        let empty_excludes: Vec<String> = vec![];

        // With include_ephemeral=true, default exclusions are skipped
        assert!(!should_exclude_path(".runs/current/foo.json", &empty_excludes, true));
        assert!(!should_exclude_path("target/debug/foo", &empty_excludes, true));

        // But extra exclusions still apply
        let extra = vec!["always_exclude/".to_string()];
        assert!(should_exclude_path("always_exclude/file.txt", &extra, true));
    }

    // ========================================================================
    // HISTORIAN APPENDIX TESTS
    // ========================================================================

    #[test]
    fn extract_historian_appendix_valid() {
        let markdown = r#"
# Quality Assessment

This is narrative content.

<!-- historian:appendix:start -->
{
  "boundary_rating": "improved",
  "boundary_notes": ["Note 1", "Note 2"],
  "confidence": "high"
}
<!-- historian:appendix:end -->
"#;

        let json = extract_historian_appendix_json(markdown).unwrap();
        assert!(json.contains("boundary_rating"));
        assert!(json.contains("improved"));
    }

    #[test]
    fn extract_historian_appendix_missing_start_marker() {
        let markdown = r#"
# Quality Assessment
No markers here.
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("start marker not found"));
    }

    #[test]
    fn extract_historian_appendix_missing_end_marker() {
        let markdown = r#"
<!-- historian:appendix:start -->
{ "boundary_rating": "improved" }
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("end marker not found"));
    }

    #[test]
    fn extract_historian_appendix_rejects_fenced_json() {
        let markdown = r#"
<!-- historian:appendix:start -->
```json
{ "boundary_rating": "improved" }
```
<!-- historian:appendix:end -->
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("code fence"));
    }

    #[test]
    fn parse_historian_appendix_full() {
        let json = r#"{
            "boundary_rating": "improved",
            "boundary_notes": ["Note 1", "Note 2"],
            "test_depth_rating": "hardened",
            "test_depth_notes": ["Test note"],
            "risk_notes": ["Risk 1"],
            "assumptions": ["Assumption 1"],
            "evidence_pointers": ["path:lib.rs:42"],
            "confidence": "high"
        }"#;

        let appendix = parse_historian_appendix(json).unwrap();
        assert_eq!(appendix.boundary_rating, Some("improved".to_string()));
        assert_eq!(appendix.boundary_notes.len(), 2);
        assert_eq!(appendix.test_depth_rating, Some("hardened".to_string()));
        assert_eq!(appendix.test_depth_notes.len(), 1);
        assert_eq!(appendix.risk_notes.len(), 1);
        assert_eq!(appendix.assumptions.len(), 1);
        assert_eq!(appendix.evidence_pointers.len(), 1);
        assert_eq!(appendix.confidence, Some("high".to_string()));
    }

    #[test]
    fn parse_historian_appendix_partial() {
        let json = r#"{
            "boundary_rating": "neutral",
            "confidence": "low"
        }"#;

        let appendix = parse_historian_appendix(json).unwrap();
        assert_eq!(appendix.boundary_rating, Some("neutral".to_string()));
        assert!(appendix.boundary_notes.is_empty());
        assert!(appendix.test_depth_rating.is_none());
        assert_eq!(appendix.confidence, Some("low".to_string()));
    }

    #[test]
    fn parse_historian_appendix_empty() {
        let json = "{}";

        let appendix = parse_historian_appendix(json).unwrap();
        assert!(appendix.boundary_rating.is_none());
        assert!(appendix.boundary_notes.is_empty());
        assert!(appendix.confidence.is_none());
    }

    #[test]
    fn parse_llm_confidence_valid() {
        assert_eq!(parse_llm_confidence("high"), Some(gov_receipts::LlmConfidence::High));
        assert_eq!(parse_llm_confidence("medium"), Some(gov_receipts::LlmConfidence::Medium));
        assert_eq!(parse_llm_confidence("low"), Some(gov_receipts::LlmConfidence::Low));
        assert_eq!(parse_llm_confidence("HIGH"), Some(gov_receipts::LlmConfidence::High)); // Case insensitive
    }

    #[test]
    fn parse_llm_confidence_invalid() {
        assert!(parse_llm_confidence("invalid").is_none());
        assert!(parse_llm_confidence("").is_none());
    }

    #[test]
    fn parse_meta_confidence_valid() {
        assert_eq!(parse_meta_confidence("high"), Some(MetaConfidence::High));
        assert_eq!(parse_meta_confidence("medium"), Some(MetaConfidence::Medium));
        assert_eq!(parse_meta_confidence("low"), Some(MetaConfidence::Low));
    }

    #[test]
    fn parse_historian_appendix_notes_only() {
        // Tests that appendices with notes but no ratings are valid
        // The merge logic should default to neutral/mixed with low confidence
        let json = r#"{
            "boundary_notes": ["[INF] Boundary work detected but impact unclear"],
            "test_depth_notes": ["[INF] Tests added but coverage unclear"],
            "risk_notes": ["[INF] Large change surface"],
            "assumptions": ["Assumed incremental refactor"],
            "evidence_pointers": ["path:crates/core/src/lib.rs:100"],
            "confidence": "low"
        }"#;

        let appendix = parse_historian_appendix(json).unwrap();
        // Ratings should be None - the merge logic will default them
        assert!(appendix.boundary_rating.is_none());
        assert!(appendix.test_depth_rating.is_none());
        // But notes should be present
        assert_eq!(appendix.boundary_notes.len(), 1);
        assert_eq!(appendix.test_depth_notes.len(), 1);
        assert_eq!(appendix.risk_notes.len(), 1);
        assert_eq!(appendix.confidence, Some("low".to_string()));
    }
}
