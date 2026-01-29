//! Error types for specification processing and validation.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during specification loading, parsing, or validation.
#[derive(Debug, Error)]
pub enum SpecError {
    /// Failed to load the specification ledger.
    #[error("Failed to load spec ledger: {0}")]
    LedgerLoad(String),

    /// Failed to parse a specification file.
    #[error("Failed to parse spec: {0}")]
    Parse(String),

    /// Configuration validation error.
    #[error("Config validation failed: {0}")]
    ConfigValidation(String),

    /// Generic I/O error with path context.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path to the file or directory that caused the error.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// YAML deserialization error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Other internal errors.
    #[error("Internal spec error: {0}")]
    Internal(String),
}

impl SpecError {
    /// Create an IO error with path context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }
}

/// Type alias for Results using [`SpecError`].
pub type Result<T> = std::result::Result<T, SpecError>;
