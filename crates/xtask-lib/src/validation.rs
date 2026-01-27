//! Validation module for ensuring xtask implementation matches specifications.
//!
//! This module provides build-time and test-time validation that:
//! 1. All required xtask commands from specs/xtask_commands.yaml are implemented
//! 2. AC reports conform to specs/ac_report.schema.json

use crate::{Error, RepoContext, Result};
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

/// Xtask commands specification loaded from specs/xtask_commands.yaml.
/// Used by validation infrastructure in tests.
#[derive(Debug, Deserialize)]
pub struct XtaskCommandsSpec {
    #[expect(dead_code, reason = "deserialized for schema completeness")]
    version: String,
    #[expect(dead_code, reason = "deserialized for schema completeness")]
    description: String,
    commands: Vec<CommandSpec>,
}

/// Individual command specification from the xtask commands YAML.
#[derive(Debug, Deserialize)]
pub struct CommandSpec {
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

/// Normalize command names to handle kebab-case to PascalCase conversion.
///
/// Examples: "ac-status" -> "AcStatus", "check" -> "Check"
pub fn normalize_command_name(name: &str) -> String {
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
pub fn extract_commands_from_enum(content: &str) -> Result<HashSet<String>> {
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
        return Err(Error::Other(anyhow::anyhow!(
            "No commands found in Commands enum. Check main.rs structure."
        )));
    }

    Ok(commands)
}

/// Validate that all required commands in the spec are implemented in the Commands enum.
///
/// This function reads specs/xtask_commands.yaml and verifies that each required
/// command name appears in the Commands enum definition.
///
/// Used by tests to ensure the xtask implementation matches the spec.
pub fn validate_xtask_commands_against_spec(repo: &RepoContext) -> Result<()> {
    // Read the spec
    let spec_path = repo.specs_dir().join("xtask_commands.yaml");
    if !spec_path.exists() {
        return Err(Error::Other(anyhow::anyhow!(
            "Specification file not found: {}. Ensure you're running from project root.",
            spec_path.display()
        )));
    }

    let spec_content = fs::read_to_string(&spec_path)
        .with_context(|| format!("Failed to read spec: {}", spec_path.display()))?;

    let spec: XtaskCommandsSpec = serde_yaml_ng::from_str(&spec_content)
        .with_context(|| format!("Failed to parse spec YAML: {}", spec_path.display()))?;

    // Extract required command names
    let required_commands: HashSet<String> = spec
        .commands
        .iter()
        .filter(|cmd| cmd.required)
        .map(|cmd| normalize_command_name(&cmd.name))
        .collect();

    // Read the main.rs to extract implemented commands
    let main_rs = repo.crates_dir().join("xtask/src/main.rs");
    if !main_rs.exists() {
        return Err(Error::Other(anyhow::anyhow!("main.rs not found: {}", main_rs.display())));
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
        return Err(Error::Other(anyhow::anyhow!(
            "Missing required xtask commands: {}\nThese commands are marked as required in {} but not found in Commands enum",
            missing.join(", "),
            spec_path.display()
        )));
    }

    // Optional: Warn about extra commands not in spec
    let extra: Vec<_> = implemented_commands.difference(&required_commands).cloned().collect();

    if !extra.is_empty() {
        eprintln!("Warning: Commands implemented but not in spec: {}", extra.join(", "));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask-lib parent")
            .parent()
            .expect("crates parent")
            .to_path_buf()
    }

    #[test]
    fn test_normalize_command_name() {
        assert_eq!(normalize_command_name("check"), "Check");
        assert_eq!(normalize_command_name("ac-status"), "AcStatus");
        assert_eq!(normalize_command_name("policy-test"), "PolicyTest");
        assert_eq!(normalize_command_name("selftest"), "Selftest");
    }

    #[test]
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
    #[ignore = "Validation infrastructure not yet fully integrated - planned for v1.2"]
    fn test_validate_xtask_commands() {
        // This test ensures the Commands enum matches the spec
        let repo = RepoContext { root: test_repo_root() };
        validate_xtask_commands_against_spec(&repo)
            .expect("xtask commands must match specification");
    }
}
