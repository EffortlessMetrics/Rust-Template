//! AC Kernel integration for xtask.
//!
//! This module provides the bridge between `ac-kernel` (the library) and
//! `xtask` (the CLI). It handles repo-specific concerns like discovering
//! the spec root and constructing the appropriate `SpecLayout`.
//!
//! # Design
//!
//! Core types and logic live in `ac-kernel`:
//! - `AcStatus`, `AcSource`, `Ac` (data model)
//! - `AcStatusJson`, `AcHistoryReport` (JSON schemas)
//! - Ledger, coverage, and history parsing
//!
//! This module adds repo-specific concerns:
//! - `spec_root()` discovery via `SPEC_ROOT` env var or cargo manifest
//! - `kernel_for_repo()` convenience constructor

use ac_kernel::{AcKernel, SpecLayout};
use std::path::PathBuf;

/// Get the spec root directory for the current repo.
///
/// Resolution order:
/// 1. `SPEC_ROOT` environment variable (if set)
/// 2. Two directories up from `CARGO_MANIFEST_DIR` (the xtask crate location)
///
/// This function is used by multiple xtask commands that need to locate
/// the `specs/` directory.
pub fn spec_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

/// Create an `AcKernel` configured for the current repository.
///
/// This is the main entry point for xtask commands that need to work with
/// AC governance data. It uses `spec_root()` to discover the repo location
/// and constructs a `SpecLayout` with the standard paths.
///
/// # Example
///
/// ```no_run
/// use crate::kernel::kernel_for_repo;
///
/// let kernel = kernel_for_repo()?;
/// let status = kernel.load_status_json()?;
/// println!("Coverage: {:.1}%", status.coverage_percent);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn kernel_for_repo() -> anyhow::Result<AcKernel> {
    let root = spec_root();
    let layout = SpecLayout::for_repo_root(&root);
    Ok(AcKernel::new(layout))
}

/// Create an `AcKernel` with a custom history directory.
///
/// This is useful for `ac-history` when the user specifies a different
/// directory for snapshot files via `--dir`.
///
/// # Arguments
///
/// * `history_dir` - The directory containing AC status snapshots
pub fn kernel_with_history_dir(history_dir: PathBuf) -> anyhow::Result<AcKernel> {
    let root = spec_root();
    let mut layout = SpecLayout::for_repo_root(&root);
    layout.history_dir = history_dir;
    Ok(AcKernel::new(layout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_root_returns_path() {
        let root = spec_root();
        // Should be an absolute path (or at least exist for the test environment)
        assert!(root.is_absolute() || root.exists());
    }

    #[test]
    fn kernel_for_repo_creates_kernel() {
        // This test runs in the repo, so the kernel should be constructable
        let kernel = kernel_for_repo().unwrap();
        // The kernel should have the repo's layout
        assert!(kernel.layout().ledger.ends_with("specs/spec_ledger.yaml"));
    }

    #[test]
    fn kernel_with_custom_history_dir() {
        use tempfile::TempDir;

        let custom_dir = TempDir::new().unwrap();
        let kernel = kernel_with_history_dir(custom_dir.path().to_path_buf()).unwrap();

        assert_eq!(kernel.layout().history_dir, custom_dir.path());
        // Other paths should still use the repo root
        assert!(kernel.layout().ledger.ends_with("specs/spec_ledger.yaml"));
    }
}
