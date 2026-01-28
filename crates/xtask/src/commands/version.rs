use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use serde_yaml;
use std::fs;
use xtask_lib::RepoContext;
use xtask_versioning::{Version, VersionInfo};

const TEMPLATE_DESCRIPTION: &str = "Rust-as-Spec Platform Cell";

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
    let repo = RepoContext::from_current_dir().context("Failed to get repository context")?;

    let ledger_path = repo.specs_dir().join("spec_ledger.yaml");

    // Read and parse spec_ledger.yaml
    let content = fs::read_to_string(&ledger_path)
        .with_context(|| format!("Failed to read spec ledger: {}", ledger_path.display()))?;

    let ledger: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse spec ledger YAML: {}", ledger_path.display()))?;

    let version = ledger
        .get("metadata")
        .and_then(|m| m.get("template_version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing template_version in spec_ledger.yaml"))?
        .to_string();

    let schema_version = ledger
        .get("metadata")
        .and_then(|m| m.get("schema_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("1.0")
        .to_string();

    let description = ledger
        .get("metadata")
        .and_then(|m| m.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or(TEMPLATE_DESCRIPTION)
        .to_string();

    let last_updated = ledger
        .get("metadata")
        .and_then(|m| m.get("last_updated"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Try to load service metadata for service_id
    let service_id = load_service_id(&repo);

    // Build kernel_tag from version using library
    let parsed_version = Version::parse(&version)?;
    let kernel_tag = parsed_version.to_kernel_tag();

    if args.json {
        // Machine-readable JSON output using library's VersionInfo
        let _version_info =
            VersionInfo::with_date(&version, last_updated.as_deref().unwrap_or("unknown"))?;
        let output = VersionOutput {
            kernel_version: version.clone(),
            kernel_tag: kernel_tag.clone(),
            schema_version,
            spec_ledger_path: ledger_path.display().to_string(),
            description,
            service_id,
            last_updated: last_updated.clone(),
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
        println!("  {}: {}", "Spec ledger".bold(), ledger_path.display().to_string().dimmed());
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
fn load_service_id(repo: &RepoContext) -> Option<String> {
    let path = repo.specs_dir().join("service_metadata.yaml");
    let content = fs::read_to_string(&path).ok()?;
    let metadata: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    metadata
        .get("service_id")
        .and_then(|v| v.as_str())
        .or_else(|| metadata.get("display_name").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
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
    fn library_version_parse() {
        // Test that the library's Version::parse works correctly
        let v = Version::parse("3.3.4").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 3);
        assert_eq!(v.patch, 4);
        assert_eq!(v.to_string(), "3.3.4");
        assert_eq!(v.to_tag(), "v3.3.4");
        assert_eq!(v.to_kernel_tag(), "v3.3.4-kernel");
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

    /// @AC-TPL-CLI-JSON-OUTPUT: version --json produces valid JSON with stable contract
    #[test]
    fn test_cli_json_output_contract() {
        // This test validates AC-TPL-CLI-JSON-OUTPUT for the version command
        // The JSON output must:
        // 1. Be valid JSON (parseable)
        // 2. Have stable top-level shape (required fields always present)
        // 3. Follow consistent naming (snake_case)
        let output = VersionOutput {
            kernel_version: "3.3.8".to_string(),
            kernel_tag: "v3.3.8-kernel".to_string(),
            schema_version: "1.0".to_string(),
            spec_ledger_path: "specs/spec_ledger.yaml".to_string(),
            description: "Rust-as-Spec Platform Cell".to_string(),
            service_id: Some("test-service".to_string()),
            last_updated: Some("2025-12-11".to_string()),
        };

        // Must serialize to valid JSON
        let json_result = serde_json::to_string_pretty(&output);
        assert!(json_result.is_ok(), "Version output must serialize to valid JSON");

        // Must parse back without errors
        let json_str = json_result.unwrap();
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(parse_result.is_ok(), "JSON output must be parseable");

        // Must be an object at top level
        let parsed = parse_result.unwrap();
        assert!(parsed.is_object(), "JSON output must be an object");
    }
}
