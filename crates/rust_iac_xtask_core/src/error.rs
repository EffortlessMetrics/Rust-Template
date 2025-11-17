//! Error types for IaC configuration and validation

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when loading, parsing, or validating IaC configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file was not found at the specified path
    #[error(
        "Configuration file not found: {0}\n\nCreate an IaC configuration file at this location. See the crate documentation for the expected format."
    )]
    FileNotFound(PathBuf),

    /// Failed to read the configuration file
    #[error("Failed to read configuration file {path}: {source}")]
    FileReadError {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Configuration file contains invalid YAML
    #[error(
        "Invalid YAML in configuration file: {0}\n\nCheck the syntax of your configuration file. Common issues:\n- Indentation must use spaces, not tabs\n- String values with special characters should be quoted\n- List items must start with '-'"
    )]
    InvalidYaml(String),

    /// Required field is missing from configuration
    #[error("Missing required field in configuration: {field}\n\n{hint}")]
    MissingField {
        /// Name of the missing field
        field: String,
        /// Hint about what should be provided
        hint: String,
    },

    /// Invalid value provided for a configuration field
    #[error("Invalid value for field '{field}': {value}\n\n{hint}")]
    InvalidValue {
        /// Name of the field with invalid value
        field: String,
        /// The invalid value
        value: String,
        /// Hint about valid values
        hint: String,
    },

    /// Required directory does not exist
    #[error("Required directory not found: {path}\n\n{hint}")]
    DirectoryNotFound {
        /// Path to the missing directory
        path: PathBuf,
        /// Hint about what should be done
        hint: String,
    },

    /// Environment name not found in configuration
    #[error(
        "Environment '{0}' not found in configuration\n\nAvailable environments: {1}\n\nAdd this environment to your configuration file or use one of the available environments."
    )]
    EnvironmentNotFound(String, String),

    /// Duplicate environment names in configuration
    #[error(
        "Duplicate environment name in configuration: '{0}'\n\nEach environment must have a unique name."
    )]
    DuplicateEnvironment(String),

    /// Not in a git repository when git validation is enabled
    #[error(
        "Not in a git repository\n\nThis project requires git for version tracking. Initialize a git repository with:\n  git init"
    )]
    NotGitRepository,

    /// Validation error with custom message
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl ConfigError {
    /// Create a MissingField error with a helpful hint
    pub fn missing_field(field: impl Into<String>, hint: impl Into<String>) -> Self {
        ConfigError::MissingField { field: field.into(), hint: hint.into() }
    }

    /// Create an InvalidValue error with a helpful hint
    pub fn invalid_value(
        field: impl Into<String>,
        value: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        ConfigError::InvalidValue { field: field.into(), value: value.into(), hint: hint.into() }
    }

    /// Create a DirectoryNotFound error with a helpful hint
    pub fn directory_not_found(path: PathBuf, hint: impl Into<String>) -> Self {
        ConfigError::DirectoryNotFound { path, hint: hint.into() }
    }

    /// Create a FileReadError with context
    pub fn file_read_error(path: PathBuf, source: std::io::Error) -> Self {
        ConfigError::FileReadError { path, source }
    }
}
