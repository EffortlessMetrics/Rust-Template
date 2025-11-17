//! Error handling example for rust_iac_xtask_core
//!
//! This example demonstrates comprehensive error handling patterns.
//!
//! Run with: cargo run --example error_handling

use rust_iac_xtask_core::{ConfigError, IaCConfig};
use std::path::Path;

fn main() {
    println!("=== Error Handling Examples ===\n");

    // Example 1: File not found
    println!("1. Attempting to load non-existent config:");
    match IaCConfig::from_file(Path::new("/nonexistent/config.yaml")) {
        Ok(_) => println!("  Unexpected success!"),
        Err(ConfigError::FileNotFound(path)) => {
            println!("  ✓ Caught FileNotFound error");
            println!("    Path: {}", path.display());
            println!("    Message: Configuration file not found");
            println!("    Suggestion: Create a config file at that location");
        }
        Err(e) => println!("  ✗ Unexpected error: {}", e),
    }
    println!();

    // Example 2: Demonstrating structured error handling
    println!("2. Structured error handling pattern:");
    let result = load_and_validate_config("examples/sample-config.yaml");
    match result {
        Ok(_) => println!("  ✓ Configuration loaded successfully"),
        Err(e) => {
            eprintln!("  ✗ Configuration error:");
            handle_config_error(&e);
        }
    }
    println!();

    // Example 3: Environment lookup with error handling
    println!("3. Safe environment lookup:");
    if let Ok(config) = IaCConfig::from_file(Path::new("examples/sample-config.yaml")) {
        match find_environment_safe(&config, "staging") {
            Some(env) => println!("  ✓ Found staging: {}", env.manifests_path.display()),
            None => println!("  ✗ Staging environment not configured"),
        }

        // Try to find non-existent environment
        match find_environment_safe(&config, "test") {
            Some(_) => println!("  Unexpected: found 'test' environment"),
            None => {
                println!("  ✓ Correctly handled missing 'test' environment");
                println!("    Available: {}", config.environment_names().join(", "));
            }
        }
    }
}

/// Load and validate configuration with proper error handling
fn load_and_validate_config(path: &str) -> Result<rust_iac_xtask_core::IaCConfig, ConfigError> {
    let config = IaCConfig::from_file(Path::new(path))?;

    // Additional custom validation could go here
    if config.environments.is_empty() {
        return Err(ConfigError::ValidationFailed(
            "Configuration must define at least one environment".to_string(),
        ));
    }

    Ok(config)
}

/// Comprehensive error handler that provides actionable feedback
fn handle_config_error(error: &ConfigError) {
    match error {
        ConfigError::FileNotFound(path) => {
            eprintln!("    File not found: {}", path.display());
            eprintln!("    → Create a configuration file at that location");
            eprintln!("    → See examples/sample-config.yaml for a template");
        }
        ConfigError::FileReadError { path, source } => {
            eprintln!("    Cannot read file: {}", path.display());
            eprintln!("    → Error: {}", source);
            eprintln!("    → Check file permissions");
        }
        ConfigError::InvalidYaml(msg) => {
            eprintln!("    Invalid YAML syntax:");
            eprintln!("    → {}", msg);
            eprintln!("    → Check for proper indentation (use spaces, not tabs)");
            eprintln!("    → Ensure proper quoting of special characters");
        }
        ConfigError::MissingField { field, hint } => {
            eprintln!("    Missing required field: {}", field);
            eprintln!("    → {}", hint);
        }
        ConfigError::InvalidValue { field, value, hint } => {
            eprintln!("    Invalid value for '{}': {}", field, value);
            eprintln!("    → {}", hint);
        }
        ConfigError::DirectoryNotFound { path, hint } => {
            eprintln!("    Directory not found: {}", path.display());
            eprintln!("    → {}", hint);
        }
        ConfigError::EnvironmentNotFound(name, available) => {
            eprintln!("    Environment '{}' not found", name);
            eprintln!("    → Available environments: {}", available);
        }
        ConfigError::DuplicateEnvironment(name) => {
            eprintln!("    Duplicate environment name: {}", name);
            eprintln!("    → Each environment must have a unique name");
        }
        ConfigError::NotGitRepository => {
            eprintln!("    Not a git repository");
            eprintln!("    → Initialize git: git init");
            eprintln!("    → Or disable check_git_repo in validation config");
        }
        ConfigError::ValidationFailed(msg) => {
            eprintln!("    Validation failed: {}", msg);
        }
        ConfigError::Io(e) => {
            eprintln!("    I/O error: {}", e);
        }
    }
}

/// Safe environment lookup that returns Option
fn find_environment_safe(
    config: &rust_iac_xtask_core::IaCConfig,
    name: &str,
) -> Option<&rust_iac_xtask_core::Environment> {
    config.find_environment(name)
}
