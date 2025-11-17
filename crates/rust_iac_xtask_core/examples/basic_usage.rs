//! Basic usage example of rust_iac_xtask_core
//!
//! This example demonstrates how to load and validate an IaC configuration file.
//!
//! Run with: cargo run --example basic_usage

use rust_iac_xtask_core::{ConfigError, IaCConfig};
use std::path::Path;

fn main() -> Result<(), ConfigError> {
    println!("=== IaC Configuration Example ===\n");

    // Try to load configuration from the examples directory
    let config_path = Path::new("examples/sample-config.yaml");

    match IaCConfig::from_file(config_path) {
        Ok(config) => {
            println!("✓ Configuration loaded successfully!\n");

            // Display project information
            println!("Project Information:");
            println!("  Name: {}", config.project.name);
            println!("  Workspace: {}", config.project.workspace_root.display());
            if let Some(desc) = &config.project.description {
                println!("  Description: {}", desc);
            }
            println!();

            // Display environments
            println!("Environments:");
            for env in &config.environments {
                println!("  • {} ({})", env.name, env.manifests_path.display());
                if env.requires_kustomize {
                    println!("    - Requires Kustomize");
                }
                if let Some(desc) = &env.description {
                    println!("    - {}", desc);
                }
                if !env.required_files.is_empty() {
                    println!("    - Required files: {}", env.required_files.join(", "));
                }
            }
            println!();

            // Display validation rules
            println!("Validation Rules:");
            if config.validation.check_git_repo {
                println!("  • Git repository required");
            }
            if !config.validation.required_directories.is_empty() {
                println!("  • Required directories:");
                for dir in &config.validation.required_directories {
                    println!("    - {}", dir.display());
                }
            }
            if !config.validation.required_files.is_empty() {
                println!("  • Required files:");
                for file in &config.validation.required_files {
                    println!("    - {}", file.display());
                }
            }
            println!();

            // Demonstrate environment lookup
            println!("Environment Lookup:");
            if let Some(dev_env) = config.find_environment("dev") {
                println!("  ✓ Found 'dev' environment at: {}", dev_env.manifests_path.display());
            } else {
                println!("  ✗ 'dev' environment not found");
            }

            println!("\nAll available environments: {}", config.environment_names().join(", "));

            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Failed to load configuration: {}\n", e);
            eprintln!("To run this example:");
            eprintln!("1. Create a configuration file at {}", config_path.display());
            eprintln!("2. See examples/sample-config.yaml for reference");
            Err(e)
        }
    }
}
