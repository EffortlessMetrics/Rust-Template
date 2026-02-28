//! Quality receipt generation.

use anyhow::{Context, Result};
use colored::Colorize;
use gov_receipts::{
    Boundaries, LlmBoundaryAssessment, LlmConfidence, LlmTestDepthAssessment, MetaConfidence,
    Quality, QualityReceipt, ReceiptMeta, Risks, Verification,
};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::timeline::{
    count_loc_changes, count_modules_touched, count_unsafe_delta, find_hotspots, get_diff_stat,
    get_modules_touched_names,
};
use crate::{generate_run_id, get_current_commit_full, get_current_commit_short, get_ref_sha};
use xtask_historian::{
    HistorianQualityAppendix, extract_historian_appendix_json, parse_historian_appendix,
};

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

/// Obtain historian analysis from file or by running command.
fn obtain_historian_analysis(args: &ReceiptsQualityArgs, analysis_dir: &Path) -> HistorianResult {
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
fn parse_llm_confidence(s: &str) -> Option<LlmConfidence> {
    match s.to_lowercase().as_str() {
        "high" => Some(LlmConfidence::High),
        "medium" => Some(LlmConfidence::Medium),
        "low" => Some(LlmConfidence::Low),
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
                confidence: Some(LlmConfidence::Low),
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
                confidence: Some(LlmConfidence::Low),
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
