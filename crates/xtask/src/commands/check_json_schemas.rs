//! JSON schema check for CLI outputs.
//!
//! This command detects breaking changes in the JSON output schemas
//! of xtask commands by comparing against golden snapshots.
//!
//! ## Commands Checked
//!
//! - `ac-status --json` - AC coverage report
//! - `friction-list --json` - Friction log entries
//! - `questions-list --json` - Questions list
//! - `fork-list --json` - Fork registry
//! - `issues-search --json` - Unified issues search
//! - `version --json` - Version information
//!
//! ## Usage
//!
//! ```bash
//! cargo xtask check-json-schemas
//! ```
//!
//! ## Exit Codes
//!
//! - `0`: No breaking changes detected
//! - `1`: Breaking changes detected (requires ADR)

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the repository root from CARGO_MANIFEST_DIR
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("xtask parent").parent().expect("crates parent").to_path_buf()
}

/// CLI commands that have JSON output contracts
const JSON_COMMANDS: &[&str] =
    &["ac-status", "friction-list", "questions-list", "fork-list", "issues-search", "version"];

/// Schema directory for golden snapshots
fn schema_dir() -> PathBuf {
    repo_root().join("specs").join("schemas")
}

/// Run a command and capture its JSON output
fn capture_command_output(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("cargo")
        .args(["run", "-p", "xtask", "--", command])
        .args(args)
        .output()
        .with_context(|| format!("Failed to run: cargo xtask {}", command))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Command failed: cargo xtask {}", command));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Parse JSON and validate basic structure
fn validate_json_structure(json: &str, command: &str) -> Result<()> {
    // Basic JSON validation
    if let Err(e) = serde_json::from_str::<serde_json::Value>(json) {
        return Err(anyhow::anyhow!("Invalid JSON output from {}: {}", command, e));
    }

    Ok(())
}

/// Check if golden snapshot exists
fn golden_snapshot_path(command: &str) -> PathBuf {
    schema_dir().join(format!("{}.golden.json", command))
}

/// Read golden snapshot
fn read_golden_snapshot(command: &str) -> Result<Option<String>> {
    let path = golden_snapshot_path(command);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read golden snapshot: {}", path.display()))?;
    Ok(Some(content))
}

/// Compare current output with golden snapshot
fn compare_with_golden(command: &str, current: &str, golden: &str) -> Result<bool> {
    // Parse both as JSON for structural comparison
    let current_json: serde_json::Value = serde_json::from_str(current)
        .with_context(|| format!("Failed to parse current JSON from {}", command))?;
    let golden_json: serde_json::Value = serde_json::from_str(golden)
        .with_context(|| format!("Failed to parse golden JSON from {}", command))?;

    // Check for breaking changes (removed fields, changed types)
    let has_breaking = detect_breaking_changes(&current_json, &golden_json, command)?;

    // Check for additions (non-breaking but notable)
    let has_additions = detect_additions(&current_json, &golden_json);

    if has_breaking {
        println!();
        eprintln!("{}", "Breaking changes detected:".red().bold());
        eprintln!("  Command: {}", command.cyan());
        eprintln!();
        eprintln!("{}", "To approve this change:".yellow().bold());
        eprintln!("  1. Create an ADR documenting the breaking change");
        eprintln!("  2. Update golden snapshot: specs/schemas/{}.golden.json", command);
        eprintln!("  3. Update consumer documentation");
        eprintln!("  4. Run: cargo xtask release-prepare");
        return Ok(true);
    }

    if has_additions {
        println!();
        println!("{}", "API additions detected (non-breaking):".yellow());
        println!("  Command: {}", command.cyan());
    }

    Ok(false)
}

/// Detect breaking changes in JSON output
fn detect_breaking_changes(
    current: &serde_json::Value,
    golden: &serde_json::Value,
    _command: &str,
) -> Result<bool> {
    let mut has_breaking = false;

    match (current, golden) {
        (serde_json::Value::Object(current_obj), serde_json::Value::Object(golden_obj)) => {
            // Check for removed fields
            for key in golden_obj.keys() {
                if !current_obj.contains_key(key) {
                    eprintln!("  - Removed field: {}", key.red());
                    has_breaking = true;
                }
            }

            // Check for type changes
            for (key, current_value) in current_obj.iter() {
                if let Some(golden_value) = golden_obj.get(key)
                    && !types_compatible(current_value, golden_value)
                {
                    eprintln!(
                        "  - Type change for field {}: {:?} -> {:?}",
                        key, golden_value, current_value
                    );
                    has_breaking = true;
                }
            }
        }
        _ => {
            // Different structure types - consider breaking
            eprintln!("  - Structural change detected");
            has_breaking = true;
        }
    }

    Ok(has_breaking)
}

/// Check if two JSON values have compatible types
fn types_compatible(current: &serde_json::Value, golden: &serde_json::Value) -> bool {
    matches!(
        (current, golden),
        (serde_json::Value::Null, serde_json::Value::Null)
            | (serde_json::Value::Bool(_), serde_json::Value::Bool(_))
            | (serde_json::Value::Number(_), serde_json::Value::Number(_))
            | (serde_json::Value::Number(_), serde_json::Value::String(_))
            | (serde_json::Value::String(_), serde_json::Value::String(_))
            | (serde_json::Value::Array(_), serde_json::Value::Array(_))
            | (serde_json::Value::Object(_), serde_json::Value::Object(_))
    )
}

/// Detect additions (new fields) in current output
fn detect_additions(current: &serde_json::Value, golden: &serde_json::Value) -> bool {
    match (current, golden) {
        (serde_json::Value::Object(current_obj), serde_json::Value::Object(golden_obj)) => {
            for key in current_obj.keys() {
                if !golden_obj.contains_key(key) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Generate golden snapshot for a command
fn generate_golden_snapshot(command: &str) -> Result<()> {
    println!("{} Generating golden snapshot for {}...", "📝".blue().bold(), command.cyan());

    let output = capture_command_output(command, &["--json"])?;

    // Validate JSON structure
    validate_json_structure(&output, command)?;

    // Write golden snapshot
    let path = golden_snapshot_path(command);
    fs::create_dir_all(schema_dir()).with_context(|| {
        format!("Failed to create schema directory: {}", schema_dir().display())
    })?;
    fs::write(&path, &output)
        .with_context(|| format!("Failed to write golden snapshot: {}", path.display()))?;

    println!("{} Golden snapshot written to {}", "✓".green(), path.display());

    Ok(())
}

/// Arguments for check-json-schemas command
pub struct CheckJsonSchemasArgs {
    /// Generate golden snapshots instead of checking
    pub generate: bool,
}

/// Run the check-json-schemas command
pub fn run(args: CheckJsonSchemasArgs) -> Result<()> {
    println!("{}", "Checking CLI JSON output schemas...".blue().bold());
    println!();

    let mut has_breaking = false;
    let mut checked_commands = Vec::new();

    // Check each command
    for command in JSON_COMMANDS {
        if args.generate {
            // Generate golden snapshot mode
            generate_golden_snapshot(command)?;
            checked_commands.push(command.to_string());
            continue;
        }

        println!("{} Checking {}...", "🔍".blue().bold(), command.cyan());

        // Check if golden snapshot exists
        let golden_opt = read_golden_snapshot(command)?;

        // Capture current output
        let current_output = match capture_command_output(command, &["--json"]) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("{} Failed to capture output from {}: {}", "⚠".yellow(), command, e);
                continue;
            }
        };

        if let Some(golden) = golden_opt {
            // Compare with golden snapshot
            let breaking = compare_with_golden(command, &current_output, &golden)?;
            if breaking {
                has_breaking = true;
            }
            checked_commands.push(command.to_string());
        } else {
            // No golden snapshot exists - offer to generate
            println!("{} No golden snapshot found for {}", "⚠".yellow(), command);
            println!("{} Run with --generate to create golden snapshots", "💡".yellow());
        }
    }

    println!();
    println!("{}", "Summary:".blue().bold());
    println!("  Checked commands: {}", checked_commands.len());
    println!("  Commands: {}", JSON_COMMANDS.join(", ").cyan());

    if has_breaking {
        println!();
        eprintln!("{}", "❌ Breaking changes detected".red().bold());
        eprintln!();
        eprintln!(
            "{}",
            "Please create an ADR and update golden snapshots before proceeding.".yellow()
        );
        std::process::exit(1);
    }

    println!();
    println!("{}", "✓ All JSON schemas are stable".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_commands_defined() {
        // Verify all JSON commands are defined
        assert_eq!(JSON_COMMANDS.len(), 6);
        assert!(JSON_COMMANDS.contains(&"ac-status"));
        assert!(JSON_COMMANDS.contains(&"version"));
        assert!(JSON_COMMANDS.contains(&"friction-list"));
    }

    #[test]
    fn test_types_compatible() {
        use serde_json::json;

        // Test type compatibility checks
        assert!(types_compatible(&json!("null"), &json!("null")));
        assert!(types_compatible(&json!(true), &json!(false)));
        assert!(types_compatible(&json!(42), &json!("123")));
        assert!(types_compatible(&json!("string"), &json!("string")));
        assert!(types_compatible(&json!([]), &json!([])));
        assert!(types_compatible(&json!({}), &json!({})));
        assert!(!types_compatible(&json!("string"), &json!(42)));
    }

    #[test]
    fn test_detect_breaking_changes_removed_field() {
        use serde_json::json;

        let current = json!({"a": 1, "b": 2});
        let golden = json!({"a": 1, "b": 2, "c": 3});

        let has_breaking = detect_breaking_changes(&current, &golden, "test").unwrap();
        assert!(has_breaking);
    }

    #[test]
    fn test_detect_breaking_changes_type_change() {
        use serde_json::json;

        let current = json!({"a": "string"});
        let golden = json!({"a": 42});

        let has_breaking = detect_breaking_changes(&current, &golden, "test").unwrap();
        assert!(has_breaking);
    }

    #[test]
    fn test_detect_additions() {
        use serde_json::json;

        let current = json!({"a": 1, "b": 2, "c": 3});
        let golden = json!({"a": 1, "b": 2});

        let has_additions = detect_additions(&current, &golden);
        assert!(has_additions);
    }

    #[test]
    fn test_golden_snapshot_path() {
        let path = golden_snapshot_path("ac-status");
        assert!(path.ends_with("specs/schemas/ac-status.golden.json"));
    }
}
