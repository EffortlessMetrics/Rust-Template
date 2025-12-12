//! Common validation utilities.

use anyhow::Result;
use std::path::Path;

/// Validate that a file exists.
pub fn require_file(path: &Path, description: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        anyhow::bail!("{} not found at: {}", description, path.display())
    }
}

/// Validate that a directory exists.
pub fn require_dir(path: &Path, description: &str) -> Result<()> {
    if path.is_dir() {
        Ok(())
    } else {
        anyhow::bail!("{} directory not found at: {}", description, path.display())
    }
}
