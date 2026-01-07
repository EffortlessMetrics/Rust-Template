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
    ComputeSpend, Confidence, DevLtMinutes, EconomicsReceipt, Environment, GateReceipt, GateResult,
    GateStatus, Iterations, ValueDelivered,
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
