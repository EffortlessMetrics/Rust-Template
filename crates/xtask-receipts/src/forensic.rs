//! Forensic receipt orchestration.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;

use crate::generate_run_id;
use crate::quality::{ReceiptsQualityArgs, run_quality};
use crate::telemetry::{ReceiptsTelemetryArgs, run_telemetry};
use crate::timeline::{ReceiptsTimelineArgs, run_timeline};
use crate::validate::{ReceiptsValidateArgs, run_validate};

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
