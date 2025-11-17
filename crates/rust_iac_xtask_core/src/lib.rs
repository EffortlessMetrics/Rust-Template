//! # rust_iac_xtask_core
//!
//! Production-ready Infrastructure as Code (IaC) orchestration library for Rust projects.
//!
//! This crate provides a configuration-driven framework for orchestrating development and deployment
//! workflows in Rust-based IaC templates. It is designed to be reusable across different projects
//! and makes no assumptions about specific acceptance criteria IDs, environment names, or project
//! structure beyond what is explicitly configured.
//!
//! ## Overview
//!
//! `rust_iac_xtask_core` helps you:
//! - Define and validate project-specific IaC configurations
//! - Ensure required directories and files exist before deployment
//! - Provide clear, actionable error messages for misconfigurations
//! - Maintain a minimal, well-documented public API
//!
//! ## Quick Start
//!
//! Create a configuration file (e.g., `iac-config.yaml`):
//!
//! ```yaml
//! project:
//!   name: my-rust-project
//!   workspace_root: .
//!
//! environments:
//!   - name: dev
//!     manifests_path: infra/k8s/dev
//!     requires_kustomize: false
//!   - name: staging
//!     manifests_path: infra/k8s/staging
//!     requires_kustomize: true
//!   - name: prod
//!     manifests_path: infra/k8s/prod
//!     requires_kustomize: true
//!
//! validation:
//!   check_git_repo: true
//!   required_directories:
//!     - specs
//!     - infra
//! ```
//!
//! Load and validate the configuration in your code:
//!
//! ```rust,no_run
//! use rust_iac_xtask_core::{IaCConfig, ConfigError};
//! use std::path::Path;
//!
//! fn main() -> Result<(), ConfigError> {
//!     let config = IaCConfig::from_file(Path::new("iac-config.yaml"))?;
//!
//!     // Access validated configuration
//!     println!("Project: {}", config.project.name);
//!
//!     // Find an environment
//!     let dev_env = config.find_environment("dev")
//!         .ok_or_else(|| ConfigError::EnvironmentNotFound("dev".to_string()))?;
//!
//!     println!("Dev manifests: {}", dev_env.manifests_path.display());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! All errors are represented by the [`ConfigError`] type, which provides:
//! - Structured error variants for different failure modes
//! - Human-readable error messages
//! - Context about what went wrong and how to fix it
//!
//! ```rust,no_run
//! use rust_iac_xtask_core::{IaCConfig, ConfigError};
//! use std::path::Path;
//!
//! match IaCConfig::from_file(Path::new("config.yaml")) {
//!     Ok(config) => println!("Config loaded successfully"),
//!     Err(ConfigError::FileNotFound(path)) => {
//!         eprintln!("Config file not found: {}", path.display());
//!         eprintln!("Create a config file at that location.");
//!     }
//!     Err(ConfigError::InvalidYaml(msg)) => {
//!         eprintln!("Invalid YAML: {}", msg);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! ## Design Philosophy
//!
//! This library follows these principles:
//! - **No panics**: All error conditions return `Result` types
//! - **Validation first**: Configuration is validated on load, not at use time
//! - **Clear errors**: Error messages explain what's wrong and suggest fixes
//! - **Minimal API**: Small public surface area that's easy to understand
//! - **No assumptions**: Works with any project structure defined in config

mod config;
mod error;
mod validation;

pub use config::{Environment, IaCConfig, ProjectInfo, ValidationRules};
pub use error::ConfigError;

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, ConfigError>;
