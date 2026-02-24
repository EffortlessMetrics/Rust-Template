//! Shared utilities for xtask automation.
//!
//! This crate provides common functionality used across xtask commands:
//! - Environment detection (CI, non-interactive, low-resources modes)
//! - Validation helpers for xtask implementation verification
//! - Error types and scaffolding
//!
//! ## Design Principles
//!
//! - **Minimal dependencies**: Only essential crates are used
//! - **No clap**: This is a library, not a CLI tool
//! - **Re-exports**: Provides a clean public API for xtask consumers

pub mod env;
pub mod hash;
pub mod path_safety;
pub mod validation;

pub use env::{describe_mode, is_ci, is_low_resources, is_noninteractive, should_skip_bdd};

/// Result type alias for xtask-lib operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Common error types for xtask-lib operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error reading a file
    #[error("Failed to read file: {path}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Error writing a file
    #[error("Failed to write file: {path}")]
    FileWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Error parsing YAML
    #[error("Failed to parse YAML: {path}")]
    YamlParse {
        path: String,
        #[source]
        source: serde_yaml_ng::Error,
    },

    /// Error parsing JSON
    #[error("Failed to parse JSON: {path}")]
    JsonParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    /// Error executing a command
    #[error("Command failed: {command}")]
    Command {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// Generic error with context
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Repository context providing information about the current workspace.
#[derive(Debug, Clone)]
pub struct RepoContext {
    /// Path to the repository root
    pub root: std::path::PathBuf,
}

impl RepoContext {
    /// Create a new RepoContext from the current working directory.
    ///
    /// This attempts to find the repository root by looking for common markers
    /// like `.git`, `Cargo.toml`, etc.
    pub fn from_current_dir() -> Result<Self> {
        let current = std::env::current_dir()
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to get current directory: {}", e)))?;

        // Try to find repo root by looking for .git directory
        let root = find_repo_root(&current)?;
        Ok(Self { root })
    }

    /// Get the path to the specs directory.
    pub fn specs_dir(&self) -> std::path::PathBuf {
        self.root.join("specs")
    }

    /// Get the path to the crates directory.
    pub fn crates_dir(&self) -> std::path::PathBuf {
        self.root.join("crates")
    }
}

/// Find the repository root by searching upward for `.git` directory.
fn find_repo_root(start: &std::path::Path) -> Result<std::path::PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err(Error::Other(anyhow::anyhow!(
                "Could not find repository root (no .git directory found)"
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_context_from_current_dir() {
        let ctx = RepoContext::from_current_dir();
        assert!(ctx.is_ok(), "Should be able to create RepoContext from current dir");
        let ctx = ctx.unwrap();
        assert!(ctx.root.exists(), "Repo root should exist");
        assert!(ctx.specs_dir().exists(), "Specs directory should exist");
        assert!(ctx.crates_dir().exists(), "Crates directory should exist");
    }
}
