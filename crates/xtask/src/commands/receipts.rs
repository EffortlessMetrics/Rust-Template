//! Generate receipts from gate execution.
//!
//! This module provides the `receipts gate` command which runs validation gates
//! (fmt, clippy, tests) and emits structured JSON receipts to `.runs/current/receipts/`.
//!
//! Receipts provide machine-readable evidence of gate execution for:
//! - CI pipelines
//! - IDP integrations
//! - Audit trails
//! - Agent workflows

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Gate execution receipt - machine-readable evidence of validation gate runs
#[derive(Debug, Serialize, Deserialize)]
pub struct GateReceipt {
    /// Schema version for forward compatibility
    pub schema_version: String,
    /// Unique run identifier (timestamp + PR number if available)
    pub run_id: String,
    /// Pull request number (if running in PR context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u32>,
    /// Git commit SHA at time of execution
    pub commit: String,
    /// ISO8601 timestamp when gate execution started
    pub started_at: String,
    /// ISO8601 timestamp when gate execution finished
    pub finished_at: String,
    /// Individual gate results
    pub gates: Vec<GateResult>,
    /// Overall pass/fail status
    pub overall_status: String,
    /// Repository/template version
    pub repo_version: String,
    /// Environment information
    pub environment: Environment,
}

/// Result of a single gate execution
#[derive(Debug, Serialize, Deserialize)]
pub struct GateResult {
    /// Gate name (e.g., "fmt", "clippy", "tests")
    pub name: String,
    /// Full command that was executed
    pub command: String,
    /// Status: "pass" or "fail"
    pub status: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Environment metadata captured during gate execution
#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    /// Operating system
    pub os: String,
    /// Rust toolchain version
    pub rust_version: String,
    /// Whether running inside nix shell
    pub nix_shell: bool,
}

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

/// Run gates and emit gate.json receipt
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

    // Run gates
    let mut gates = Vec::new();
    let mut all_pass = true;

    // Gate 1: fmt check
    let start = Instant::now();
    let fmt_status = Command::new("cargo").args(["fmt", "--all", "--check"]).status();
    let fmt_pass = fmt_status.map(|s| s.success()).unwrap_or(false);
    gates.push(GateResult {
        name: "fmt".to_string(),
        command: "cargo fmt --all --check".to_string(),
        status: if fmt_pass { "pass" } else { "fail" }.to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    });
    if !fmt_pass {
        all_pass = false;
    }
    println!(
        "  {} fmt: {}",
        if fmt_pass { "pass".green() } else { "fail".red() },
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
        status: if clippy_pass { "pass" } else { "fail" }.to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    });
    if !clippy_pass {
        all_pass = false;
    }
    println!(
        "  {} clippy: {}ms",
        if clippy_pass { "pass".green() } else { "fail".red() },
        gates.last().unwrap().duration_ms
    );

    // Gate 3: tests
    let start = Instant::now();
    let test_status = Command::new("cargo").args(["test", "--all"]).status();
    let test_pass = test_status.map(|s| s.success()).unwrap_or(false);
    gates.push(GateResult {
        name: "tests".to_string(),
        command: "cargo test --all".to_string(),
        status: if test_pass { "pass" } else { "fail" }.to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    });
    if !test_pass {
        all_pass = false;
    }
    println!(
        "  {} tests: {}ms",
        if test_pass { "pass".green() } else { "fail".red() },
        gates.last().unwrap().duration_ms
    );

    let finished_at = Utc::now();

    // Get repo version from spec_ledger
    let repo_version = get_repo_version().unwrap_or_else(|| "unknown".to_string());

    let receipt = GateReceipt {
        schema_version: "1.0".to_string(),
        run_id,
        pr: args.pr,
        commit,
        started_at: started_at.to_rfc3339(),
        finished_at: finished_at.to_rfc3339(),
        gates,
        overall_status: if all_pass { "pass" } else { "fail" }.to_string(),
        repo_version,
        environment: Environment { os: std::env::consts::OS.to_string(), rust_version, nix_shell },
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_receipt_serialization_shape() {
        // Test that GateReceipt serializes to expected JSON structure
        let receipt = GateReceipt {
            schema_version: "1.0".to_string(),
            run_id: "2025-01-07T12-00-00Z-pr123".to_string(),
            pr: Some(123),
            commit: "abc123def456".to_string(),
            started_at: "2025-01-07T12:00:00Z".to_string(),
            finished_at: "2025-01-07T12:05:00Z".to_string(),
            gates: vec![GateResult {
                name: "fmt".to_string(),
                command: "cargo fmt --all --check".to_string(),
                status: "pass".to_string(),
                duration_ms: 1234,
            }],
            overall_status: "pass".to_string(),
            repo_version: "3.3.14".to_string(),
            environment: Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: true,
            },
        };

        let json = serde_json::to_value(&receipt).unwrap();

        // Verify required fields exist
        assert!(json.get("schema_version").is_some());
        assert!(json.get("run_id").is_some());
        assert!(json.get("commit").is_some());
        assert!(json.get("started_at").is_some());
        assert!(json.get("finished_at").is_some());
        assert!(json.get("gates").is_some());
        assert!(json.get("overall_status").is_some());
        assert!(json.get("repo_version").is_some());
        assert!(json.get("environment").is_some());

        // Verify values
        assert_eq!(json["schema_version"], "1.0");
        assert_eq!(json["pr"], 123);
        assert_eq!(json["overall_status"], "pass");
    }

    #[test]
    fn gate_receipt_optional_pr_excluded_when_none() {
        let receipt = GateReceipt {
            schema_version: "1.0".to_string(),
            run_id: "test".to_string(),
            pr: None, // No PR
            commit: "abc123".to_string(),
            started_at: "2025-01-07T12:00:00Z".to_string(),
            finished_at: "2025-01-07T12:05:00Z".to_string(),
            gates: vec![],
            overall_status: "pass".to_string(),
            repo_version: "3.3.14".to_string(),
            environment: Environment {
                os: "linux".to_string(),
                rust_version: "1.83.0".to_string(),
                nix_shell: false,
            },
        };

        let json_str = serde_json::to_string_pretty(&receipt).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // pr should not be present when None
        assert!(parsed.get("pr").is_none(), "pr should not be present when None");
    }

    #[test]
    fn gate_result_structure() {
        let result = GateResult {
            name: "clippy".to_string(),
            command: "cargo clippy --all-targets".to_string(),
            status: "fail".to_string(),
            duration_ms: 5000,
        };

        let json = serde_json::to_value(&result).unwrap();

        assert_eq!(json["name"], "clippy");
        assert_eq!(json["command"], "cargo clippy --all-targets");
        assert_eq!(json["status"], "fail");
        assert_eq!(json["duration_ms"], 5000);
    }

    #[test]
    fn environment_structure() {
        let env = Environment {
            os: "linux".to_string(),
            rust_version: "1.83.0".to_string(),
            nix_shell: true,
        };

        let json = serde_json::to_value(&env).unwrap();

        assert_eq!(json["os"], "linux");
        assert_eq!(json["rust_version"], "1.83.0");
        assert_eq!(json["nix_shell"], true);
    }
}
