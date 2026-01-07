//! Generate PR cover sheet from receipts.
//!
//! This command generates a markdown cover sheet for PRs based on governance
//! receipts from a run directory. The cover sheet includes:
//! - What changed section
//! - Review map (where to look)
//! - Proof (receipts from selftest, gate checks)
//! - Errata section
//! - Unified budget (DevLT, compute spend)
//! - Reproduce locally instructions

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Arguments for pr-cover command
#[derive(Debug, Clone, Default)]
pub struct PrCoverArgs {
    /// PR number
    pub pr: u32,
    /// Directory containing receipts (default: .runs/pr/{pr}/latest/)
    pub run_dir: Option<PathBuf>,
    /// Output file (default: stdout)
    pub output: Option<PathBuf>,
}

/// Gate receipt structure (subset of fields we care about)
#[derive(Debug, Deserialize)]
struct GateReceipt {
    #[serde(default)]
    passed: bool,
    #[serde(default)]
    steps: Option<Vec<GateStep>>,
}

#[derive(Debug, Deserialize)]
struct GateStep {
    name: String,
    #[serde(default)]
    passed: bool,
}

/// Economics data structure
#[derive(Debug, Deserialize)]
struct Economics {
    #[serde(default)]
    dev_lt_minutes: Option<f64>,
    #[serde(default)]
    compute_spend_usd: Option<f64>,
    #[serde(default)]
    notes: Option<String>,
}

pub fn run(args: PrCoverArgs) -> Result<()> {
    println!("{}", "Generating PR cover sheet...".blue().bold());

    // Determine run directory
    let run_dir =
        args.run_dir.unwrap_or_else(|| PathBuf::from(format!(".runs/pr/{}/latest", args.pr)));

    // Check if receipts exist
    let gate_path = run_dir.join("receipts/gate.json");
    let economics_path = run_dir.join("economics.json");

    // Try to load gate receipt
    let gate_receipt = if gate_path.exists() {
        match fs::read_to_string(&gate_path) {
            Ok(content) => serde_json::from_str::<GateReceipt>(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Try to load economics
    let economics = if economics_path.exists() {
        match fs::read_to_string(&economics_path) {
            Ok(content) => serde_json::from_str::<Economics>(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Generate cover sheet markdown
    let mut content = String::new();
    content.push_str("## Cover Sheet\n\n");

    // What changed section
    content.push_str("### What changed\n");
    content.push_str("- <TODO: describe the change>\n\n");

    // Review map section
    content.push_str("### Where to look (review map)\n");
    content.push_str("| Area | Files | Why |\n");
    content.push_str("|------|-------|-----|\n");
    content.push_str("| <domain> | `path/to/files` | <what changed> |\n\n");

    // Proof/receipts section
    content.push_str("### Proof (receipts)\n");
    content.push_str("| Check | Status | Receipt |\n");
    content.push_str("|-------|--------|--------|\n");

    // Check for gate.json and report status
    if let Some(ref gate) = gate_receipt {
        let status = if gate.passed { "PASS" } else { "FAIL" };
        content.push_str(&format!("| Selftest | {} | `{}` |\n", status, gate_path.display()));

        // Add individual step results if available
        if let Some(ref steps) = gate.steps {
            for step in steps {
                let step_status = if step.passed { "PASS" } else { "FAIL" };
                content.push_str(&format!("| - {} | {} | |\n", step.name, step_status));
            }
        }
    } else if gate_path.exists() {
        content.push_str(&format!("| Selftest | ? | `{}` (parse error) |\n", gate_path.display()));
    } else {
        content.push_str("| Selftest | N/A | (no receipt found) |\n");
    }

    content.push('\n');

    // Errata section
    content.push_str("### Errata (what we got wrong)\n");
    content.push_str("- Nothing identified in this PR's scope.\n");
    content.push_str("- (If you find something later, add an addendum here.)\n\n");

    // Unified budget section
    content.push_str("### Unified budget (DevLT dominates)\n");
    content.push_str("| Metric | Value | Notes |\n");
    content.push_str("|--------|-------|-------|\n");

    if let Some(ref econ) = economics {
        let dev_lt = econ
            .dev_lt_minutes
            .map(|m| format!("~{:.0} min", m))
            .unwrap_or_else(|| "unknown".to_string());
        let compute = econ
            .compute_spend_usd
            .map(|c| format!("~${:.2}", c))
            .unwrap_or_else(|| "unknown".to_string());
        let notes = econ.notes.as_deref().unwrap_or("");

        content.push_str(&format!("| DevLT (author) | {} | {} |\n", dev_lt, notes));
        content.push_str(&format!("| Compute spend | {} | |\n", compute));
    } else if economics_path.exists() {
        content.push_str("| DevLT | ? | (economics.json parse error) |\n");
        content.push_str("| Compute spend | ? | |\n");
    } else {
        content.push_str("| DevLT | unknown | (no economics.json) |\n");
        content.push_str("| Compute spend | unknown | |\n");
    }

    content.push('\n');

    // Reproduce locally section
    content.push_str("### Reproduce locally\n");
    content.push_str("```bash\n");
    content.push_str("nix develop\n");
    content.push_str("cargo xtask selftest\n");
    content.push_str("```\n");

    // Output
    match args.output {
        Some(path) => {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(&path, &content)
                .with_context(|| format!("Failed to write to {}", path.display()))?;
            println!("{} Cover sheet written to {}", "OK".green(), path.display());
        }
        None => {
            println!("\n{}", content);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_run_dir() {
        let args = PrCoverArgs { pr: 123, run_dir: None, output: None };

        let expected = PathBuf::from(".runs/pr/123/latest");
        let actual =
            args.run_dir.unwrap_or_else(|| PathBuf::from(format!(".runs/pr/{}/latest", args.pr)));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_custom_run_dir() {
        let args =
            PrCoverArgs { pr: 456, run_dir: Some(PathBuf::from("/custom/path")), output: None };

        assert_eq!(args.run_dir, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_gate_receipt_deserialization() {
        let json = r#"{
            "passed": true,
            "timestamp": "2025-01-07T12:00:00Z",
            "steps": [
                {"name": "fmt", "passed": true},
                {"name": "clippy", "passed": true}
            ]
        }"#;

        let receipt: GateReceipt = serde_json::from_str(json).unwrap();
        assert!(receipt.passed);
        assert_eq!(receipt.steps.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_economics_deserialization() {
        let json = r#"{
            "dev_lt_minutes": 45.5,
            "compute_spend_usd": 0.12,
            "notes": "Includes CI run"
        }"#;

        let econ: Economics = serde_json::from_str(json).unwrap();
        assert_eq!(econ.dev_lt_minutes, Some(45.5));
        assert_eq!(econ.compute_spend_usd, Some(0.12));
        assert_eq!(econ.notes, Some("Includes CI run".to_string()));
    }

    #[test]
    fn test_economics_partial_deserialization() {
        // Test that missing fields don't cause errors
        let json = r#"{"dev_lt_minutes": 30}"#;

        let econ: Economics = serde_json::from_str(json).unwrap();
        assert_eq!(econ.dev_lt_minutes, Some(30.0));
        assert_eq!(econ.compute_spend_usd, None);
        assert_eq!(econ.notes, None);
    }
}
