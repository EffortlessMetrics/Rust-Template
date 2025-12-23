//! Infrastructure-as-code xtask utilities.
//!
//! This crate provides core utilities for Rust IaC project scaffolding:
//! - Project initialization (brownfield and greenfield modes)
//! - Self-test validation of project structure
//! - Configuration management via `RUST_IAC.toml`
//! - Specification ledger scaffolding (`specs/spec_ledger.yaml`)
//!
//! # Main exports
//!
//! - [`init`]: Initialize project structure in brownfield or greenfield mode
//! - [`selftest`]: Verify project structure and configuration
//! - [`InitMode`]: Choose between brownfield (existing project) or greenfield (new project)
//! - [`config`]: Configuration types and defaults for `RUST_IAC.toml`
//! - [`commands`]: Command implementations for IaC operations

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

pub mod commands;
pub mod config;

/// Initialize a Rust IaC project structure
pub fn init(mode: InitMode, project_root: Option<PathBuf>) -> Result<()> {
    let root = project_root.unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("{} Initializing Rust IaC structure in {} mode...", "=>".bright_blue(), mode);

    match mode {
        InitMode::Brownfield => init_brownfield(&root)?,
        InitMode::Greenfield => init_greenfield(&root)?,
    }

    println!("{} Rust IaC initialization complete!", "✓".bright_green());
    Ok(())
}

/// Run self-test to verify the project structure
pub fn selftest(project_root: Option<PathBuf>) -> Result<()> {
    let root = project_root.unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("{} Running Rust IaC self-test...", "=>".bright_blue());

    // Check for required files
    check_file_exists(&root, "RUST_IAC.toml", "Configuration file")?;
    check_dir_exists(&root, "specs", "Specifications directory")?;
    check_dir_exists(&root, "policy", "Policy directory")?;
    check_dir_exists(&root, ".llm", "LLM context directory")?;

    // Verify RUST_IAC.toml is valid
    let config_path = root.join("RUST_IAC.toml");
    let config_content = fs::read_to_string(&config_path)
        .with_context(|| "Failed to read RUST_IAC.toml".to_string())?;
    let _config: config::RustIacConfig =
        toml::from_str(&config_content).with_context(|| "RUST_IAC.toml is not valid TOML")?;

    println!("{} Configuration is valid", "✓".bright_green());

    // Check spec_ledger.yaml exists
    check_file_exists(&root, "specs/spec_ledger.yaml", "Specification ledger")?;

    println!("{} All self-tests passed!", "✓".bright_green());
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum InitMode {
    Brownfield,
    Greenfield,
}

impl std::fmt::Display for InitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitMode::Brownfield => write!(f, "brownfield"),
            InitMode::Greenfield => write!(f, "greenfield"),
        }
    }
}

impl std::str::FromStr for InitMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "brownfield" => Ok(InitMode::Brownfield),
            "greenfield" => Ok(InitMode::Greenfield),
            _ => Err(anyhow::anyhow!("Invalid mode: {}. Use 'brownfield' or 'greenfield'", s)),
        }
    }
}

fn init_brownfield(root: &Path) -> Result<()> {
    println!("{} Setting up brownfield project structure...", "  ".bright_blue());

    // Create RUST_IAC.toml
    create_config_file(root)?;

    // Create directory structure
    create_dir_if_not_exists(&root.join("specs"))?;
    create_dir_if_not_exists(&root.join("specs/features"))?;
    create_dir_if_not_exists(&root.join("policy"))?;
    create_dir_if_not_exists(&root.join("policy/tests"))?;
    create_dir_if_not_exists(&root.join(".llm"))?;

    // Create spec_ledger.yaml
    create_spec_ledger(root)?;

    // Create sample policy
    create_sample_policy(root)?;

    // Create .llm/contextpack.yaml
    create_contextpack(root)?;

    println!("{} Created directory structure", "  ✓".bright_green());

    Ok(())
}

fn init_greenfield(root: &Path) -> Result<()> {
    println!("{} Setting up greenfield project structure...", "  ".bright_blue());

    // Greenfield includes everything from brownfield plus additional scaffolding
    init_brownfield(root)?;

    // Add additional greenfield-specific files
    println!("{} Added greenfield-specific scaffolding", "  ✓".bright_green());

    Ok(())
}

fn create_config_file(root: &Path) -> Result<()> {
    let config_path = root.join("RUST_IAC.toml");

    if config_path.exists() {
        println!("{} RUST_IAC.toml already exists, skipping", "  !".yellow());
        return Ok(());
    }

    let config = config::RustIacConfig::default();
    let toml_content = toml::to_string_pretty(&config)?;

    fs::write(&config_path, toml_content).with_context(|| "Failed to write RUST_IAC.toml")?;

    println!("{} Created RUST_IAC.toml", "  ✓".bright_green());
    Ok(())
}

fn create_spec_ledger(root: &Path) -> Result<()> {
    let ledger_path = root.join("specs/spec_ledger.yaml");

    if ledger_path.exists() {
        println!("{} spec_ledger.yaml already exists, skipping", "  !".yellow());
        return Ok(());
    }

    let ledger_content = r#"# Specification Ledger
# This file tracks the relationship between user stories, requirements, and acceptance criteria

user_stories: []
  # - id: US-001
  #   title: Example User Story
  #   description: As a user, I want to...
  #   acceptance_criteria:
  #     - AC-001

requirements: []
  # - id: REQ-001
  #   title: Example Requirement
  #   description: System shall...
  #   user_story: US-001
  #   priority: high

acceptance_criteria: []
  # - id: AC-001
  #   title: Example Acceptance Criterion
  #   description: When... Then...
  #   requirement: REQ-001
  #   priority: must-have
"#;

    fs::write(&ledger_path, ledger_content).with_context(|| "Failed to write spec_ledger.yaml")?;

    println!("{} Created specs/spec_ledger.yaml", "  ✓".bright_green());
    Ok(())
}

fn create_sample_policy(root: &Path) -> Result<()> {
    let policy_path = root.join("policy/example.rego");

    if policy_path.exists() {
        println!("{} policy/example.rego already exists, skipping", "  !".yellow());
        return Ok(());
    }

    let policy_content = r#"# Example Policy
# This is a sample Rego policy for demonstration purposes

package policies.example

# Deny rule example
deny[msg] {
    # Add your policy rules here
    false  # This will never trigger
    msg := "Example deny message"
}

# Allow rule example
allow {
    # Add your allow conditions here
    true
}
"#;

    fs::write(&policy_path, policy_content).with_context(|| "Failed to write example.rego")?;

    println!("{} Created policy/example.rego", "  ✓".bright_green());
    Ok(())
}

fn create_contextpack(root: &Path) -> Result<()> {
    let contextpack_path = root.join(".llm/contextpack.yaml");

    if contextpack_path.exists() {
        println!("{} .llm/contextpack.yaml already exists, skipping", "  !".yellow());
        return Ok(());
    }

    let contextpack_content = r#"# LLM Context Packs
# Define task-specific context bundles for LLM assistance

tasks:
  - name: selftest
    description: "Context for running self-tests"
    includes:
      - "specs/**/*.yaml"
      - "specs/**/*.feature"
      - "policy/**/*.rego"
      - "RUST_IAC.toml"
"#;

    fs::write(&contextpack_path, contextpack_content)
        .with_context(|| "Failed to write contextpack.yaml")?;

    println!("{} Created .llm/contextpack.yaml", "  ✓".bright_green());
    Ok(())
}

fn create_dir_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

fn check_file_exists(root: &Path, relative_path: &str, description: &str) -> Result<()> {
    let path = root.join(relative_path);
    if !path.exists() {
        anyhow::bail!("{} not found: {}", description, path.display());
    }
    println!("{} {} exists", "  ✓".bright_green(), description);
    Ok(())
}

fn check_dir_exists(root: &Path, relative_path: &str, description: &str) -> Result<()> {
    let path = root.join(relative_path);
    if !path.is_dir() {
        anyhow::bail!("{} not found: {}", description, path.display());
    }
    println!("{} {} exists", "  ✓".bright_green(), description);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_brownfield_creates_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        init(InitMode::Brownfield, Some(root.to_path_buf())).unwrap();

        // Check files exist
        assert!(root.join("RUST_IAC.toml").exists());
        assert!(root.join("specs").is_dir());
        assert!(root.join("specs/spec_ledger.yaml").exists());
        assert!(root.join("policy").is_dir());
        assert!(root.join("policy/example.rego").exists());
        assert!(root.join(".llm").is_dir());
        assert!(root.join(".llm/contextpack.yaml").exists());
    }

    #[test]
    fn test_selftest_passes_after_init() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        init(InitMode::Brownfield, Some(root.to_path_buf())).unwrap();
        selftest(Some(root.to_path_buf())).unwrap();
    }
}
