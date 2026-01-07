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

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use gov_receipts::{
    Boundaries, ChangeSurface, ComputeSpend, Confidence, Contracts, DevLtMinutes, EconomicsReceipt,
    Environment, FrictionZone, GateReceipt, GateResult, GateStatus, Iterations,
    LlmBoundaryAssessment, LlmTestDepthAssessment, ProbeProfile, ProbeResult, ProbeStatus, Quality,
    QualityReceipt, Risks, Session, SessionClassification, SkippedProbe, TelemetryReceipt,
    TimelineConfidence, TimelineReceipt, Topology, UnsafeDelta, ValueDelivered, Verification,
    WallClock,
};
use jsonschema::Validator;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Arguments for the receipts gate command
#[derive(Debug, Clone)]
pub struct ReceiptsGateArgs {
    /// Pull request number (optional)
    pub pr: Option<u32>,
    /// Output directory for receipts
    pub output_dir: PathBuf,
}

impl Default for ReceiptsGateArgs {
    fn default() -> Self {
        Self { pr: None, output_dir: PathBuf::from(".runs/current") }
    }
}

/// Run gates and emit gate.json receipt using gov-receipts types
pub fn run_gate(args: ReceiptsGateArgs) -> Result<()> {
    println!("{}", "Generating gate receipt...".blue().bold());

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create {}", args.output_dir.display()))?;

    let receipts_dir = args.output_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    let started_at = Utc::now();
    let run_id = format!(
        "{}-pr{}",
        started_at.format("%Y-%m-%dT%H-%M-%SZ"),
        args.pr.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())
    );

    // Get git commit
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get rust version
    let rust_version = Command::new("rustc")
        .args(["--version"])
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.split_whitespace().nth(1).unwrap_or("unknown").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());

    // Check if in nix shell
    let nix_shell = std::env::var("IN_NIX_SHELL").is_ok();

    // Run gates and collect results
    let mut gates = Vec::new();
    let mut all_pass = true;

    // Gate 1: fmt check
    let start = Instant::now();
    let fmt_status = Command::new("cargo").args(["fmt", "--all", "--check"]).status();
    let fmt_pass = fmt_status.map(|s| s.success()).unwrap_or(false);
    gates.push(GateResult {
        name: "fmt".to_string(),
        command: "cargo fmt --all --check".to_string(),
        status: if fmt_pass { GateStatus::Pass } else { GateStatus::Fail },
        duration_ms: start.elapsed().as_millis() as u64,
        details: None,
    });
    if !fmt_pass {
        all_pass = false;
    }
    println!(
        "  {} fmt: {}ms",
        if fmt_pass { "PASS".green() } else { "FAIL".red() },
        gates.last().unwrap().duration_ms
    );

    // Gate 2: clippy
    let start = Instant::now();
    let clippy_status =
        Command::new("cargo").args(["clippy", "--all-targets", "--", "-D", "warnings"]).status();
    let clippy_pass = clippy_status.map(|s| s.success()).unwrap_or(false);
    gates.push(GateResult {
        name: "clippy".to_string(),
        command: "cargo clippy --all-targets -- -D warnings".to_string(),
        status: if clippy_pass { GateStatus::Pass } else { GateStatus::Fail },
        duration_ms: start.elapsed().as_millis() as u64,
        details: None,
    });
    if !clippy_pass {
        all_pass = false;
    }
    println!(
        "  {} clippy: {}ms",
        if clippy_pass { "PASS".green() } else { "FAIL".red() },
        gates.last().unwrap().duration_ms
    );

    // Gate 3: tests
    let start = Instant::now();
    let test_status = Command::new("cargo").args(["test", "--all"]).status();
    let test_pass = test_status.map(|s| s.success()).unwrap_or(false);
    gates.push(GateResult {
        name: "tests".to_string(),
        command: "cargo test --all".to_string(),
        status: if test_pass { GateStatus::Pass } else { GateStatus::Fail },
        duration_ms: start.elapsed().as_millis() as u64,
        details: None,
    });
    if !test_pass {
        all_pass = false;
    }
    println!(
        "  {} tests: {}ms",
        if test_pass { "PASS".green() } else { "FAIL".red() },
        gates.last().unwrap().duration_ms
    );

    let finished_at = Utc::now();

    // Get repo version from spec_ledger
    let repo_version = get_repo_version().unwrap_or_else(|| "unknown".to_string());

    // Build receipt using gov-receipts types
    let mut builder = GateReceipt::builder()
        .run_id(run_id)
        .commit(commit)
        .started_at(started_at)
        .finished_at(finished_at)
        .gates(gates)
        .overall_status(if all_pass { GateStatus::Pass } else { GateStatus::Fail })
        .repo_version(repo_version)
        .environment(Environment { os: std::env::consts::OS.to_string(), rust_version, nix_shell });

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

    let receipt = builder.build();

    // Write receipt
    let gate_path = receipts_dir.join("gate.json");
    let json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(&gate_path, &json)?;

    println!();
    println!("{} Receipt written to {}", "OK".green(), gate_path.display());
    println!("  Overall: {}", if all_pass { "PASS".green().bold() } else { "FAIL".red().bold() });

    if all_pass { Ok(()) } else { anyhow::bail!("One or more gates failed") }
}

/// Get repository version from spec_ledger.yaml
fn get_repo_version() -> Option<String> {
    use serde::Deserialize;
    use std::fs;

    #[derive(Deserialize)]
    struct SpecLedger {
        metadata: Metadata,
    }

    #[derive(Deserialize)]
    struct Metadata {
        template_version: String,
    }

    let content = fs::read_to_string("specs/spec_ledger.yaml").ok()?;
    let ledger: SpecLedger = serde_yaml::from_str(&content).ok()?;
    Some(ledger.metadata.template_version)
}

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
        }
    }
}

/// Parse confidence string to Confidence enum
fn parse_confidence(s: &str) -> Confidence {
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

    let run_id = format!("{}-pr{}", Utc::now().format("%Y-%m-%dT%H-%M-%SZ"), args.pr);

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

/// Arguments for the receipts-validate command
#[derive(Debug, Clone)]
pub struct ReceiptsValidateArgs {
    /// Run directory containing receipts/ subdirectory
    pub run_dir: PathBuf,
    /// Schema directory (default: specs/schemas/)
    pub schema_dir: PathBuf,
}

impl Default for ReceiptsValidateArgs {
    fn default() -> Self {
        Self { run_dir: PathBuf::from(".runs/current"), schema_dir: PathBuf::from("specs/schemas") }
    }
}

/// Result of validating a single receipt
#[derive(Debug)]
struct ValidationResult {
    receipt_name: String,
    schema_name: String,
    passed: bool,
    errors: Vec<String>,
}

/// Validate receipt JSON files against their schemas
///
/// Finds all `receipts/*.json` files in the run directory, matches each
/// to its corresponding schema (gate.json -> gate.schema.json), and validates.
pub fn run_validate(args: ReceiptsValidateArgs) -> Result<()> {
    println!("{}", "Validating receipts against schemas...".blue().bold());
    println!();

    // Check that run_dir exists
    if !args.run_dir.exists() {
        anyhow::bail!("Run directory does not exist: {}", args.run_dir.display());
    }

    // Check that schema_dir exists
    if !args.schema_dir.exists() {
        anyhow::bail!("Schema directory does not exist: {}", args.schema_dir.display());
    }

    let receipts_dir = args.run_dir.join("receipts");
    if !receipts_dir.exists() {
        anyhow::bail!(
            "Receipts directory does not exist: {}\n\
             Expected to find receipts/*.json files here.",
            receipts_dir.display()
        );
    }

    // Load all available schemas into a map: base_name -> (schema_path, compiled_validator)
    let mut schemas: HashMap<String, (PathBuf, Validator)> = HashMap::new();
    for entry in std::fs::read_dir(&args.schema_dir).with_context(|| {
        format!("Failed to read schema directory: {}", args.schema_dir.display())
    })? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            // Extract base name: "gate.schema.json" -> "gate"
            if let Some(base_name) = file_name.strip_suffix(".schema.json") {
                let schema_content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read schema: {}", path.display()))?;
                let schema_json: serde_json::Value = serde_json::from_str(&schema_content)
                    .with_context(|| format!("Failed to parse schema JSON: {}", path.display()))?;
                let validator = Validator::new(&schema_json).map_err(|e| {
                    anyhow::anyhow!("Failed to compile schema {}: {}", path.display(), e)
                })?;
                schemas.insert(base_name.to_string(), (path.clone(), validator));
            }
        }
    }

    if schemas.is_empty() {
        anyhow::bail!(
            "No schemas found in {}. Expected files like gate.schema.json, economics.schema.json",
            args.schema_dir.display()
        );
    }

    println!(
        "  Loaded {} schema(s): {}",
        schemas.len(),
        schemas.keys().cloned().collect::<Vec<_>>().join(", ")
    );
    println!();

    // Find all receipt JSON files
    let mut results: Vec<ValidationResult> = Vec::new();
    let mut receipt_count = 0;

    for entry in std::fs::read_dir(&receipts_dir)
        .with_context(|| format!("Failed to read receipts directory: {}", receipts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        receipt_count += 1;
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        // Extract base name: "gate.json" -> "gate"
        let base_name = file_name.strip_suffix(".json").unwrap_or(&file_name);

        // Find matching schema
        let Some((schema_path, validator)) = schemas.get(base_name) else {
            results.push(ValidationResult {
                receipt_name: file_name.clone(),
                schema_name: format!("{}.schema.json", base_name),
                passed: false,
                errors: vec![format!(
                    "No matching schema found. Expected {} in {}",
                    format!("{}.schema.json", base_name),
                    args.schema_dir.display()
                )],
            });
            continue;
        };

        // Read and parse receipt
        let receipt_content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read receipt: {}", path.display()))?;
        let receipt_json: serde_json::Value = match serde_json::from_str(&receipt_content) {
            Ok(v) => v,
            Err(e) => {
                results.push(ValidationResult {
                    receipt_name: file_name.clone(),
                    schema_name: schema_path.file_name().unwrap().to_string_lossy().to_string(),
                    passed: false,
                    errors: vec![format!("Invalid JSON: {}", e)],
                });
                continue;
            }
        };

        // Validate against schema
        let errors: Vec<String> =
            validator.iter_errors(&receipt_json).map(|e| format!("{}", e)).collect();

        results.push(ValidationResult {
            receipt_name: file_name,
            schema_name: schema_path.file_name().unwrap().to_string_lossy().to_string(),
            passed: errors.is_empty(),
            errors,
        });
    }

    if receipt_count == 0 {
        anyhow::bail!(
            "No receipt JSON files found in {}\n\
             Run `cargo xtask receipts-gate` or `cargo xtask receipts-economics` to generate receipts.",
            receipts_dir.display()
        );
    }

    // Print results
    let mut passed = 0;
    let mut failed = 0;

    for result in &results {
        if result.passed {
            passed += 1;
            println!(
                "  {} {} (validated against {})",
                "PASS".green(),
                result.receipt_name,
                result.schema_name
            );
        } else {
            failed += 1;
            println!("  {} {} (schema: {})", "FAIL".red(), result.receipt_name, result.schema_name);
            for error in &result.errors {
                println!("       {} {}", "-".dimmed(), error);
            }
        }
    }

    println!();
    println!(
        "Summary: {} passed, {} failed out of {} receipt(s)",
        passed.to_string().green(),
        if failed > 0 { failed.to_string().red() } else { failed.to_string().normal() },
        receipt_count
    );

    if failed > 0 {
        anyhow::bail!("{} receipt(s) failed schema validation", failed);
    }

    println!();
    println!("{} All receipts valid", "OK".green());
    Ok(())
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
    /// LLM-provided notes
    pub notes: Vec<String>,
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
        }
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

    let run_id = format!(
        "{}-pr{}",
        Utc::now().format("%Y-%m-%dT%H-%M-%SZ"),
        args.pr.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())
    );

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

    // Build quality metrics
    let mut boundaries = Boundaries { modules_touched, hotspots, llm_assessment: None };

    // Add LLM boundary assessment if provided
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

    let mut verification = Verification {
        tests_added_loc: tests_loc,
        impl_added_loc: impl_loc,
        test_density_delta,
        llm_test_depth: None,
    };

    // Add LLM test depth assessment if provided
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

    // Store values before moving unsafe_delta
    let unsafe_added = unsafe_delta.added;
    let unsafe_removed = unsafe_delta.removed;

    let mut builder = QualityReceipt::builder().run_id(&run_id);

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
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
                llm_risk_notes: vec![],
            },
        })
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
    /// Probe profile (fast/full/exhibit)
    pub profile: String,
    /// Base branch for comparison (default: origin/main)
    pub base_branch: String,
}

impl Default for ReceiptsTelemetryArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            profile: "fast".to_string(),
            base_branch: "origin/main".to_string(),
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

    let run_id = format!(
        "{}-pr{}",
        Utc::now().format("%Y-%m-%dT%H-%M-%SZ"),
        args.pr.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())
    );

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

    // For v0, we mark most probes as "not_run" since tooling isn't integrated yet
    match profile {
        ProbeProfile::Fast => {
            not_run.push(SkippedProbe {
                probe: "cargo-geiger".to_string(),
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
        }
        ProbeProfile::Full | ProbeProfile::Exhibit => {
            // Mark as not_run since tooling isn't yet integrated
            not_run.push(SkippedProbe {
                probe: "cargo-geiger".to_string(),
                reason: "tooling not yet integrated".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "cargo-deny".to_string(),
                reason: "tooling not yet integrated".to_string(),
            });
            not_run.push(SkippedProbe {
                probe: "coverage".to_string(),
                reason: "tooling not yet integrated".to_string(),
            });
        }
    }

    // Add a basic probe result for git-diff (always runs)
    probes.push(ProbeResult {
        name: "git-diff".to_string(),
        version: None,
        status: ProbeStatus::Run,
        reason: None,
        duration_ms: Some(0),
        artifact_path: None,
    });

    let mut builder = TelemetryReceipt::builder()
        .run_id(&run_id)
        .profile(profile)
        .change_surface(change_surface)
        .contracts(contracts)
        .probes(probes);

    for skip in not_run {
        builder = builder.skipped(skip);
    }

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

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
}

impl Default for ReceiptsTimelineArgs {
    fn default() -> Self {
        Self {
            pr: None,
            output_dir: PathBuf::from(".runs/current"),
            base_branch: "origin/main".to_string(),
            session_gap_minutes: 30,
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

    let run_id = format!(
        "{}-pr{}",
        Utc::now().format("%Y-%m-%dT%H-%M-%SZ"),
        args.pr.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())
    );

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
    let friction_zones = find_friction_zones(&args.base_branch);

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

    for reason in reasons {
        builder = builder.topology_reason(reason);
    }

    if let Some(pr_num) = args.pr {
        builder = builder.pr(pr_num as u64);
    }

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
    println!("  Topology: {:?} ({:?} confidence)", receipt.topology, receipt.topology_confidence);

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
    #[allow(dead_code)]
    sha: String,
    timestamp: chrono::DateTime<Utc>,
    #[allow(dead_code)]
    author: String,
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
fn find_friction_zones(base_branch: &str) -> Vec<FrictionZone> {
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

            // Filter to files touched 2+ times
            file_commits
                .into_iter()
                .filter(|(_, commits)| commits.len() >= 2)
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

#[cfg(test)]
mod tests {
    use super::*;
    use gov_receipts::GateStatus;

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
}
