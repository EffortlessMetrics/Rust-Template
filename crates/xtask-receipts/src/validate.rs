//! Receipt validation against schemas.

use anyhow::{Context, Result};
use colored::Colorize;
use jsonschema::Validator;
use std::collections::HashMap;
use std::path::PathBuf;

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
pub struct ValidationResult {
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
            receipt_name: file_name.clone(),
            schema_name: schema_path.file_name().unwrap().to_string_lossy().to_string(),
            passed: errors.is_empty(),
            errors,
        });
    }

    if receipt_count == 0 {
        anyhow::bail!(
            "No receipts found in {}. Expected receipts/*.json files.",
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
