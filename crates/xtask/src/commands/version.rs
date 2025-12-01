use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SPEC_LEDGER_PATH: &str = "specs/spec_ledger.yaml";
const SERVICE_METADATA_PATH: &str = "specs/service_metadata.yaml";
const TEMPLATE_DESCRIPTION: &str = "Rust-as-Spec Platform Cell";

#[derive(Debug, Deserialize)]
struct SpecLedger {
    metadata: LedgerMetadata,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    last_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ServiceMetadata {
    #[serde(default)]
    service_id: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
}

/// Machine-readable version output (stable contract for AI/IDP consumers)
#[derive(Debug, Serialize)]
struct VersionOutput {
    /// The canonical kernel version from spec_ledger (e.g., "3.3.4")
    kernel_version: String,
    /// The kernel tag for git (e.g., "v3.3.4-kernel")
    kernel_tag: String,
    /// Schema version for spec_ledger format
    schema_version: String,
    /// Path to the canonical spec_ledger
    spec_ledger_path: String,
    /// Human-readable description
    description: String,
    /// Service ID from service_metadata.yaml (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    service_id: Option<String>,
    /// Last updated date from spec_ledger metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    last_updated: Option<String>,
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

    let version = ledger.metadata.template_version.clone();
    let schema_version =
        ledger.metadata.schema_version.clone().unwrap_or_else(|| "1.0".to_string());
    let description =
        ledger.metadata.description.clone().unwrap_or_else(|| TEMPLATE_DESCRIPTION.to_string());
    let last_updated = ledger.metadata.last_updated.clone();

    // Try to load service metadata for service_id
    let service_id = load_service_id();

    // Build kernel_tag from version
    let kernel_tag = format!("v{}-kernel", version);

    if args.json {
        // Machine-readable JSON output
        let output = VersionOutput {
            kernel_version: version,
            kernel_tag,
            schema_version,
            spec_ledger_path: SPEC_LEDGER_PATH.to_string(),
            description,
            service_id,
            last_updated,
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
        println!("  {}: {}", "Kernel tag".bold(), kernel_tag.cyan());
        println!("  {}: {}", "Schema version".bold(), schema_version.dimmed());
        println!("  {}: {}", "Spec ledger".bold(), SPEC_LEDGER_PATH.dimmed());
        if let Some(id) = &service_id {
            println!("  {}: {}", "Service ID".bold(), id.yellow());
        }
        if let Some(date) = &last_updated {
            println!("  {}: {}", "Last updated".bold(), date.dimmed());
        }
        println!("  {}: {}", "Description".bold(), description.dimmed());
        println!();
    }

    Ok(())
}

/// Try to load service_id from service_metadata.yaml (returns None if unavailable)
fn load_service_id() -> Option<String> {
    let path = Path::new(SERVICE_METADATA_PATH);
    let content = fs::read_to_string(path).ok()?;
    let metadata: ServiceMetadata = serde_yaml::from_str(&content).ok()?;
    metadata.service_id.or(metadata.display_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_output_serialization_shape() {
        // Test that VersionOutput serializes to expected JSON structure
        let output = VersionOutput {
            kernel_version: "3.3.4".to_string(),
            kernel_tag: "v3.3.4-kernel".to_string(),
            schema_version: "1.0".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Rust-as-Spec Platform Cell".to_string(),
            service_id: Some("rust-template".to_string()),
            last_updated: Some("2025-11-30".to_string()),
        };

        let json = serde_json::to_value(&output).unwrap();

        // Verify required fields exist and have expected types
        assert!(json.get("kernel_version").is_some());
        assert!(json.get("kernel_tag").is_some());
        assert!(json.get("schema_version").is_some());
        assert!(json.get("spec_ledger_path").is_some());
        assert!(json.get("description").is_some());

        // Verify values
        assert_eq!(json["kernel_version"], "3.3.4");
        assert_eq!(json["kernel_tag"], "v3.3.4-kernel");
        assert_eq!(json["schema_version"], "1.0");
        assert_eq!(json["spec_ledger_path"], "specs/spec_ledger.yaml");
        assert_eq!(json["description"], "Rust-as-Spec Platform Cell");
    }

    #[test]
    fn version_json_shape_is_stable() {
        // This test documents the stable JSON contract for AI/IDP consumers
        // Changes to this test indicate a breaking change to the --json output
        let output = VersionOutput {
            kernel_version: "1.0.0".to_string(),
            kernel_tag: "v1.0.0-kernel".to_string(),
            schema_version: "1.0".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Test description".to_string(),
            service_id: None,
            last_updated: None,
        };

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Required fields that must always be present (stable contract)
        let required_fields =
            ["kernel_version", "kernel_tag", "schema_version", "spec_ledger_path", "description"];
        for field in &required_fields {
            assert!(
                parsed.get(*field).is_some(),
                "Missing required field '{}' in version --json output",
                field
            );
        }

        // Verify types are stable
        assert!(parsed["kernel_version"].is_string(), "kernel_version must be a string");
        assert!(parsed["kernel_tag"].is_string(), "kernel_tag must be a string");
        assert!(parsed["schema_version"].is_string(), "schema_version must be a string");
        assert!(parsed["spec_ledger_path"].is_string(), "spec_ledger_path must be a string");
        assert!(parsed["description"].is_string(), "description must be a string");
    }

    #[test]
    fn version_optional_fields_excluded_when_none() {
        // Verify that optional fields are not serialized when None
        let output = VersionOutput {
            kernel_version: "1.0.0".to_string(),
            kernel_tag: "v1.0.0-kernel".to_string(),
            schema_version: "1.0".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Test".to_string(),
            service_id: None,
            last_updated: None,
        };

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Optional fields should not be present when None
        assert!(parsed.get("service_id").is_none(), "service_id should not be present when None");
        assert!(
            parsed.get("last_updated").is_none(),
            "last_updated should not be present when None"
        );
    }
}
