//! Economics receipt generation.

use anyhow::{Context, Result};
use colored::Colorize;
use gov_receipts::{
    ComputeSpend, Confidence, DevLtMinutes, EconomicsReceipt, Iterations, ValueDelivered,
};
use std::path::PathBuf;

use crate::generate_run_id;

/// Arguments for the receipts economics command
#[derive(Debug, Clone)]
pub struct ReceiptsEconomicsArgs {
    /// PR number
    pub pr: u32,
    /// Output directory for receipts
    pub output_dir: PathBuf,
    /// Author time in minutes (optional)
    pub author_minutes: Option<u32>,
    /// Author time confidence: measured, estimated, or unknown
    pub author_confidence: String,
    /// Review time in minutes (optional)
    pub review_minutes: Option<u32>,
    /// Review time confidence: measured, estimated, or unknown
    pub review_confidence: String,
    /// Number of human interventions
    pub interventions: u32,
    /// Compute cost in USD (optional)
    pub compute_usd: Option<f64>,
    /// Compute confidence: measured, estimated, or unknown
    pub compute_confidence: String,
    /// Number of CI/gate runs
    pub runs: u32,
    /// Number of failed gates before success
    pub failed_gates: u32,
    /// Number of fix-and-retry loops
    pub fix_loops: u32,
    /// Description of uncertainty reduced (optional)
    pub uncertainty_reduced: Option<String>,
    /// Description of rework prevented (optional)
    pub rework_prevented: Option<String>,
    /// DevLT notes (optional)
    pub devlt_notes: Option<String>,
    /// Compute notes (optional)
    pub compute_notes: Option<String>,
    /// Iteration notes (optional)
    pub iteration_notes: Option<String>,
    /// Shared run_id (for forensic orchestration); generated if None
    pub run_id: Option<String>,
}

impl Default for ReceiptsEconomicsArgs {
    fn default() -> Self {
        Self {
            pr: 0,
            output_dir: PathBuf::from(".runs/current"),
            author_minutes: None,
            author_confidence: "unknown".to_string(),
            review_minutes: None,
            review_confidence: "unknown".to_string(),
            interventions: 0,
            compute_usd: None,
            compute_confidence: "unknown".to_string(),
            runs: 0,
            failed_gates: 0,
            fix_loops: 0,
            uncertainty_reduced: None,
            rework_prevented: None,
            devlt_notes: None,
            compute_notes: None,
            iteration_notes: None,
            run_id: None,
        }
    }
}

/// Parse confidence string to Confidence enum
pub fn parse_confidence(s: &str) -> Confidence {
    match s.to_lowercase().as_str() {
        "measured" => Confidence::Measured,
        "estimated" => Confidence::Estimated,
        _ => Confidence::Unknown,
    }
}

/// Generate economics.json receipt from provided values
pub fn run_economics(args: ReceiptsEconomicsArgs) -> Result<()> {
    println!("{}", "Generating economics receipt...".blue().bold());

    // Create output directory and receipts subdirectory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    // Use provided run_id (for forensic orchestration) or generate new one
    let run_id = args.run_id.clone().unwrap_or_else(|| generate_run_id(Some(args.pr)));

    let receipt = EconomicsReceipt::builder()
        .schema_version("1.0")
        .pr(args.pr as u64)
        .run_id(&run_id)
        .devlt_minutes(DevLtMinutes {
            author: args.author_minutes,
            author_confidence: parse_confidence(&args.author_confidence),
            review: args.review_minutes,
            review_confidence: parse_confidence(&args.review_confidence),
            interventions: args.interventions,
            notes: args.devlt_notes,
        })
        .compute(ComputeSpend {
            tokens_usd: args.compute_usd,
            confidence: parse_confidence(&args.compute_confidence),
            runs: args.runs,
            notes: args.compute_notes,
        })
        .iterations(Iterations {
            failed_gates: args.failed_gates,
            fix_loops: args.fix_loops,
            notes: args.iteration_notes,
        })
        .value_delivered(ValueDelivered {
            uncertainty_reduced: args.uncertainty_reduced,
            rework_prevented: args.rework_prevented,
        })
        .build();

    // Write receipt to receipts/ subdirectory (consistent with gate.json)
    let economics_path = receipts_dir.join("economics.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&economics_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), economics_path.display());
    println!("  PR: #{}", args.pr);
    println!(
        "  DevLT: {} min (author: {}, review: {})",
        args.author_minutes.unwrap_or(0) + args.review_minutes.unwrap_or(0),
        args.author_confidence,
        args.review_confidence
    );
    if let Some(usd) = args.compute_usd {
        println!("  Compute: ${:.2} ({}, {} runs)", usd, args.compute_confidence, args.runs);
    } else {
        println!("  Compute: {} ({} runs)", args.compute_confidence, args.runs);
    }
    if args.failed_gates > 0 || args.fix_loops > 0 {
        println!("  Iterations: {} failed gates, {} fix loops", args.failed_gates, args.fix_loops);
    }

    Ok(())
}
