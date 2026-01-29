use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Validate a configuration file against the schema.
pub fn run(env: &str) -> Result<()> {
    println!("{} {}...", "Validating configuration for:".bold(), env.cyan());

    let schema_path = Path::new("specs/config_schema.yaml");
    let config_path = Path::new("config").join(format!("{}.yaml", env));

    if !config_path.exists() {
        anyhow::bail!("Configuration file not found: {}", config_path.display());
    }

    match spec_runtime::validate_config(schema_path, &config_path) {
        Ok(config) => {
            println!("  {} Schema validation passed", "✓".green());
            println!("  {} All required fields present", "✓".green());
            println!("  {} Type constraints satisfied", "✓".green());
            println!();
            println!("{} Config valid for {}", "✓".green().bold(), env.cyan());
            if !config.secrets.is_empty() {
                println!(
                    "  (Found {} secrets, {} settings)",
                    config.secrets.len(),
                    config.settings.len()
                );
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("  {} Validation failed:", "✗".red());
            eprintln!("    {}", e.to_string().red());
            anyhow::bail!("Configuration validation failed for '{}'", env);
        }
    }
}
