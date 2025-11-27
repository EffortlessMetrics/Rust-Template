use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SPEC_LEDGER_PATH: &str = "specs/spec_ledger.yaml";
const TEMPLATE_DESCRIPTION: &str = "Rust-as-Spec Platform Cell";

#[derive(Debug, Deserialize)]
struct SpecLedger {
    metadata: LedgerMetadata,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct VersionOutput {
    kernel_version: String,
    spec_ledger_path: String,
    description: String,
}

#[derive(Default)]
pub struct VersionArgs {
    pub json: bool,
}

pub fn run(args: VersionArgs) -> Result<()> {
    let ledger_path = Path::new(SPEC_LEDGER_PATH);

    // Read and parse spec_ledger.yaml
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read spec ledger: {}", ledger_path.display()))?;

    let ledger: SpecLedger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse spec ledger YAML: {}", ledger_path.display()))?;

    let version = ledger.metadata.template_version;
    let description =
        ledger.metadata.description.unwrap_or_else(|| TEMPLATE_DESCRIPTION.to_string());

    if args.json {
        // Machine-readable JSON output
        let output = VersionOutput {
            kernel_version: version,
            spec_ledger_path: SPEC_LEDGER_PATH.to_string(),
            description,
        };
        let json = serde_json::to_string_pretty(&output)
            .context("Failed to serialize version output to JSON")?;
        println!("{}", json);
    } else {
        // Human-readable output
        println!();
        println!("{}", "Rust-as-Spec Platform Cell".bold());
        println!("{}", "=========================".blue());
        println!();
        println!("  {}: {}", "Kernel version".bold(), version.green());
        println!("  {}: {}", "Spec ledger".bold(), SPEC_LEDGER_PATH.dimmed());
        println!("  {}: {}", "Description".bold(), description.dimmed());
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_output_serialization_shape() {
        // Test that VersionOutput serializes to expected JSON structure
        let output = VersionOutput {
            kernel_version: "v3.3.3".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Rust-as-Spec Platform Cell".to_string(),
        };

        let json = serde_json::to_value(&output).unwrap();

        // Verify required fields exist and have expected types
        assert!(json.get("kernel_version").is_some());
        assert!(json.get("spec_ledger_path").is_some());
        assert!(json.get("description").is_some());

        // Verify values
        assert_eq!(json["kernel_version"], "v3.3.3");
        assert_eq!(json["spec_ledger_path"], "specs/spec_ledger.yaml");
        assert_eq!(json["description"], "Rust-as-Spec Platform Cell");
    }

    #[test]
    fn version_json_shape_is_stable() {
        // This test documents the stable JSON contract for AI/IDP consumers
        // Changes to this test indicate a breaking change to the --json output
        let output = VersionOutput {
            kernel_version: "v1.0.0".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Test description".to_string(),
        };

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Required fields that must always be present (stable contract)
        let required_fields = ["kernel_version", "spec_ledger_path", "description"];
        for field in &required_fields {
            assert!(
                parsed.get(*field).is_some(),
                "Missing required field '{}' in version --json output",
                field
            );
        }

        // Verify types are stable
        assert!(parsed["kernel_version"].is_string(), "kernel_version must be a string");
        assert!(parsed["spec_ledger_path"].is_string(), "spec_ledger_path must be a string");
        assert!(parsed["description"].is_string(), "description must be a string");
    }
}
