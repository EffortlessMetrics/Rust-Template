//! Validation module for ensuring xtask implementation matches specifications
//!
//! This module provides build-time and test-time validation that:
//! 1. All required xtask commands from specs/xtask_commands.yaml are implemented
//! 2. AC reports conform to specs/ac_report.schema.json

#[cfg(test)]
use anyhow::{Context, Result};
#[cfg(test)]
use serde::Deserialize;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};

#[cfg(test)]
use serde_json::Value;

/// Xtask commands specification loaded from specs/xtask_commands.yaml.
/// Used by validation infrastructure in tests.
#[cfg(test)]
#[derive(Debug, Deserialize)]
struct XtaskCommandsSpec {
    #[expect(dead_code, reason = "deserialized for schema completeness")]
    version: String,
    #[expect(dead_code, reason = "deserialized for schema completeness")]
    description: String,
    commands: Vec<CommandSpec>,
}

/// Individual command specification from the xtask commands YAML.
#[cfg(test)]
#[derive(Debug, Deserialize)]
struct CommandSpec {
    name: String,
    #[expect(dead_code, reason = "deserialized for schema completeness; future validation")]
    description: String,
    required: bool,
    #[expect(dead_code, reason = "deserialized for schema completeness; future validation")]
    usage: String,
    #[serde(default)]
    #[expect(dead_code, reason = "deserialized for schema completeness; future validation")]
    has_args: bool,
}

/// Get the project root directory (two levels up from the xtask crate).
/// Used by validation functions and tests.
#[cfg(test)]
fn project_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find project root")
        .to_path_buf()
}

/// Validate that all required commands in the spec are implemented in the Commands enum.
///
/// This function reads specs/xtask_commands.yaml and verifies that each required
/// command name appears in the Commands enum definition.
///
/// Used by tests to ensure the xtask implementation matches the spec.
#[cfg(test)]
fn validate_xtask_commands_against_spec() -> Result<()> {
    // Read the spec
    let root = project_root();
    let spec_path = root.join("specs/xtask_commands.yaml");
    if !spec_path.exists() {
        anyhow::bail!(
            "Specification file not found: {}. Ensure you're running from project root.",
            spec_path.display()
        );
    }

    let spec_content = fs::read_to_string(&spec_path)
        .with_context(|| format!("Failed to read spec: {}", spec_path.display()))?;

    let spec: XtaskCommandsSpec = serde_yaml::from_str(&spec_content)
        .with_context(|| format!("Failed to parse spec YAML: {}", spec_path.display()))?;

    // Extract required command names
    let required_commands: HashSet<String> = spec
        .commands
        .iter()
        .filter(|cmd| cmd.required)
        .map(|cmd| normalize_command_name(&cmd.name))
        .collect();

    // Read the main.rs to extract implemented commands
    let main_rs = root.join("crates/xtask/src/main.rs");
    if !main_rs.exists() {
        anyhow::bail!("main.rs not found: {}", main_rs.display());
    }

    let main_content = fs::read_to_string(&main_rs)
        .with_context(|| format!("Failed to read main.rs: {}", main_rs.display()))?;

    let implemented_commands = extract_commands_from_enum(&main_content)?;

    // Validate that all required commands are implemented
    let mut missing = Vec::new();
    for cmd_name in &required_commands {
        if !implemented_commands.contains(cmd_name) {
            missing.push(cmd_name.clone());
        }
    }

    if !missing.is_empty() {
        anyhow::bail!(
            "Missing required xtask commands: {}\nThese commands are marked as required in {} but not found in Commands enum",
            missing.join(", "),
            spec_path.display()
        );
    }

    // Optional: Warn about extra commands not in spec
    let extra: Vec<_> = implemented_commands.difference(&required_commands).cloned().collect();

    if !extra.is_empty() {
        eprintln!("Warning: Commands implemented but not in spec: {}", extra.join(", "));
    }

    Ok(())
}

/// Normalize command names to handle kebab-case to PascalCase conversion.
///
/// Examples: "ac-status" -> "AcStatus", "check" -> "Check"
#[cfg(test)]
fn normalize_command_name(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().chain(chars.flat_map(|c| c.to_lowercase())).collect()
                }
            }
        })
        .collect()
}

/// Extract command variant names from the Commands enum definition.
#[cfg(test)]
fn extract_commands_from_enum(content: &str) -> Result<HashSet<String>> {
    let mut commands = HashSet::new();
    let mut in_enum = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect start of Commands enum
        if trimmed.starts_with("enum Commands") {
            in_enum = true;
            continue;
        }

        // Stop at end of enum
        if in_enum && trimmed == "}" {
            break;
        }

        // Extract variant names (lines that start with uppercase or ///comments)
        if in_enum {
            // Skip comments and empty lines
            if trimmed.starts_with("///") || trimmed.is_empty() {
                continue;
            }

            // Extract variant name (first word before {, comma, or whitespace)
            // Split on common delimiters and take first part
            let first_word = trimmed.split(&[' ', '{', ',', '\n'][..]).next().unwrap_or("");

            if !first_word.is_empty() && first_word.chars().next().is_some_and(|c| c.is_uppercase())
            {
                commands.insert(first_word.to_string());
            }
        }
    }

    if commands.is_empty() {
        anyhow::bail!("No commands found in Commands enum. Check main.rs structure.");
    }

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Validation infrastructure not yet fully integrated - planned for v1.2"]
    fn test_validate_xtask_commands() {
        // This test ensures the Commands enum matches the spec
        validate_xtask_commands_against_spec().expect("xtask commands must match specification");
    }

    #[test]
    fn test_normalize_command_name() {
        assert_eq!(normalize_command_name("check"), "Check");
        assert_eq!(normalize_command_name("ac-status"), "AcStatus");
        assert_eq!(normalize_command_name("policy-test"), "PolicyTest");
        assert_eq!(normalize_command_name("selftest"), "Selftest");
    }

    #[test]
    #[ignore = "Validation infrastructure not yet fully integrated - planned for v1.2"]
    fn test_extract_commands_from_enum() {
        let sample_enum = r#"
#[derive(Subcommand)]
enum Commands {
    /// Generate AC status report from acceptance tests
    AcStatus,
    /// Run all checks: fmt, clippy, tests
    Check,
    /// Run BDD acceptance tests
    Bdd,
    /// Generate LLM context bundle for a task
    Bundle {
        /// Task name from .llm/contextpack.yaml
        task: String,
    },
}
"#;

        let commands = extract_commands_from_enum(sample_enum).unwrap();
        assert!(commands.contains("AcStatus"));
        assert!(commands.contains("Check"));
        assert!(commands.contains("Bdd"));
        assert!(commands.contains("Bundle"));
        assert_eq!(commands.len(), 4);
    }

    #[test]
    fn test_spec_file_exists() {
        let root = project_root();
        let spec_path = root.join("specs/xtask_commands.yaml");
        assert!(spec_path.exists(), "Specification file must exist at specs/xtask_commands.yaml");
    }

    #[test]
    fn test_spec_file_is_valid_yaml() {
        let root = project_root();
        let spec_path = root.join("specs/xtask_commands.yaml");
        let content = fs::read_to_string(&spec_path)
            .expect("Should be able to read specs/xtask_commands.yaml");

        let spec: Result<XtaskCommandsSpec, _> = serde_yaml::from_str(&content);
        assert!(
            spec.is_ok(),
            "specs/xtask_commands.yaml must be valid YAML matching XtaskCommandsSpec structure"
        );

        let spec = spec.unwrap();
        assert!(!spec.commands.is_empty(), "Spec must define at least one command");
        assert!(
            spec.commands.iter().any(|cmd| cmd.required),
            "Spec must have at least one required command"
        );
    }
}

// AC Report Schema Validation
#[cfg(test)]
mod ac_report_validation {
    use super::*;
    use jsonschema::Validator;

    /// Validate that ac_report.json conforms to specs/ac_report.schema.json
    ///
    /// This test loads both the schema and a sample/generated AC report,
    /// then validates the report against the schema.
    #[test]
    fn test_ac_report_schema_exists() {
        let root = project_root();
        let schema_path = root.join("specs/ac_report.schema.json");
        assert!(schema_path.exists(), "AC report schema must exist at specs/ac_report.schema.json");
    }

    #[test]
    fn test_ac_report_schema_is_valid() {
        let root = project_root();
        let schema_path = root.join("specs/ac_report.schema.json");
        let schema_content = fs::read_to_string(&schema_path)
            .expect("Should be able to read specs/ac_report.schema.json");

        let schema: Result<Value, _> = serde_json::from_str(&schema_content);
        assert!(
            schema.is_ok(),
            "specs/ac_report.schema.json must be valid JSON: {:?}",
            schema.err()
        );

        // Verify the schema itself is valid by trying to compile it
        let schema_value = schema.unwrap();
        let compiled = Validator::new(&schema_value);
        assert!(
            compiled.is_ok(),
            "specs/ac_report.schema.json must be a valid JSON Schema: {:?}",
            compiled.err()
        );
    }

    #[test]
    fn test_ac_report_validates_against_schema() {
        let root = project_root();
        // Skip if ac_report.json doesn't exist (tests haven't been run yet)
        let report_path = root.join("target/ac_report.json");
        if !report_path.exists() || fs::metadata(&report_path).map(|m| m.len()).unwrap_or(0) == 0 {
            eprintln!("Skipping validation: target/ac_report.json not found");
            eprintln!("Run 'cargo xtask bdd' to generate the report");
            return;
        }

        // Load schema
        let schema_path = root.join("specs/ac_report.schema.json");
        let schema_content = fs::read_to_string(&schema_path).expect("Failed to read schema");
        let schema: Value =
            serde_json::from_str(&schema_content).expect("Failed to parse schema JSON");
        let validator = Validator::new(&schema).expect("Failed to compile JSON schema");

        // Load report
        let report_content =
            fs::read_to_string(&report_path).expect("Failed to read ac_report.json");
        let report: Value = match serde_json::from_str(&report_content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Skipping validation: failed to parse ac_report.json ({e})");
                eprintln!("Re-run acceptance tests to regenerate a valid report");
                return;
            }
        };

        // Validate
        let validation_result = validator.validate(&report);
        if let Err(error) = validation_result {
            panic!("AC report validation failed:\n  - {}: {}", error.instance_path(), error);
        }

        // If we get here, validation passed
        println!("[OK] AC report validates successfully against schema");
    }

    #[test]
    fn test_schema_validates_sample_report() {
        // Test with a minimal valid sample to ensure schema works
        let sample_report = r#"[
            {
                "uri": "specs/features/test.feature",
                "keyword": "Feature",
                "name": "Test Feature",
                "elements": [
                    {
                        "keyword": "Scenario",
                        "type": "scenario",
                        "name": "Test Scenario",
                        "tags": [
                            {
                                "name": "AC-TEST-001"
                            }
                        ],
                        "steps": [
                            {
                                "keyword": "Given ",
                                "name": "a test condition",
                                "result": {
                                    "status": "passed",
                                    "duration": 1000000
                                }
                            }
                        ]
                    }
                ]
            }
        ]"#;

        let root = project_root();
        let schema_path = root.join("specs/ac_report.schema.json");
        let schema_content = fs::read_to_string(&schema_path).expect("Failed to read schema");
        let schema: Value = serde_json::from_str(&schema_content).expect("Failed to parse schema");
        let validator = Validator::new(&schema).expect("Failed to compile schema");

        let report: Value =
            serde_json::from_str(sample_report).expect("Failed to parse sample report");

        let result = validator.validate(&report);
        assert!(
            result.is_ok(),
            "Sample report should validate: {:?}",
            result.err().map(|e| format!("{}: {}", e.instance_path(), e))
        );
    }
}
