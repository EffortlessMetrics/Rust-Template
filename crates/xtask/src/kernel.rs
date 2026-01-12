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
use std::path::{Path, PathBuf};

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

/// Diagnostic information about the spec root configuration.
#[derive(Debug, Clone)]
pub struct SpecRootInfo {
    pub path: PathBuf,
    pub source: String,
    pub valid: bool,
    pub missing_files: Vec<String>,
}

/// Get diagnostic information about the spec root.
///
/// This provides detailed information about where spec_root is coming from,
/// whether it's valid, and what files are missing (if any).
///
/// # Example
///
/// ```no_run
/// use crate::kernel::spec_root_info;
///
/// let info = spec_root_info();
/// if !info.valid {
///     eprintln!("Spec root {} is invalid", info.path.display());
///     eprintln!("  Source: {}", info.source);
///     eprintln!("  Missing files: {}", info.missing_files.join(", "));
/// }
/// ```
pub fn spec_root_info() -> SpecRootInfo {
    let path = spec_root();
    let source = if std::env::var("SPEC_ROOT").is_ok() {
        "SPEC_ROOT environment variable".to_string()
    } else {
        "default (relative to xtask)".to_string()
    };

    let (valid, missing_files) = validate_spec_root_path(&path);

    SpecRootInfo { path, source, valid, missing_files }
}

/// Validate that the spec root directory exists and contains expected files.
///
/// Checks for:
/// - Directory exists
/// - specs/spec_ledger.yaml exists
/// - specs/devex_flows.yaml exists
///
/// Returns (valid, missing_files) tuple.
fn validate_spec_root_path(root: &Path) -> (bool, Vec<String>) {
    let mut missing = Vec::new();

    if !root.exists() {
        missing.push(format!("Directory '{}' does not exist", root.display()));
        return (false, missing);
    }

    if !root.is_dir() {
        missing.push(format!("Path '{}' is not a directory", root.display()));
        return (false, missing);
    }

    // Check for expected files
    let expected_files = vec!["specs/spec_ledger.yaml", "specs/devex_flows.yaml"];

    for file in expected_files {
        let file_path = root.join(file);
        if !file_path.exists() {
            missing.push(file.to_string());
        }
    }

    (missing.is_empty(), missing)
}

/// Validate the spec root and return a helpful error if invalid.
///
/// This is a convenience function that returns a Result with a detailed
/// error message if the spec root is not valid.
///
/// # Errors
///
/// Returns an error if:
/// - SPEC_ROOT is set but the directory doesn't exist
/// - The spec root directory is missing required files
///
/// # Example
///
/// ```no_run
/// use crate::kernel::validate_spec_root;
///
/// validate_spec_root()?;
/// // Now safe to use spec_root() or kernel_for_repo()
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_spec_root() -> anyhow::Result<()> {
    let info = spec_root_info();

    if !info.valid {
        let mut msg =
            format!("spec root is invalid: {}\n  source: {}", info.path.display(), info.source);

        if !info.missing_files.is_empty() {
            msg.push_str("\n  missing files:\n");
            for file in &info.missing_files {
                msg.push_str(&format!("    - {}\n", file));
            }
        }

        msg.push_str("\n\ntry: cargo xtask doctor");

        if std::env::var("SPEC_ROOT").is_ok() {
            msg.push_str("\nhint: SPEC_ROOT environment variable is set");
            msg.push_str("\n      verify it points to the repository root");
        }

        anyhow::bail!(msg);
    }

    Ok(())
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

/// Get the `SpecLayout` for the current repository without creating a full kernel.
///
/// Use this when you only need path information and don't need to load AC data.
/// For commands that need to load AC status, history, or coverage, use
/// `kernel_for_repo()` instead.
///
/// # Example
///
/// ```no_run
/// use crate::kernel::layout_for_repo;
///
/// let layout = layout_for_repo();
/// println!("Ledger: {}", layout.ledger.display());
/// println!("Coverage: {}", layout.coverage_file.display());
/// ```
pub fn layout_for_repo() -> SpecLayout {
    let root = spec_root();
    SpecLayout::for_repo_root(&root)
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
    use std::path::Path;
    use testing::process::EnvVarGuard;

    /// Run a closure with SPEC_ROOT temporarily set (or unset).
    ///
    /// This ensures tests that modify SPEC_ROOT don't race against each other.
    /// Uses `EnvVarGuard` which holds a global lock and restores on drop.
    fn with_spec_root<R, F>(value: Option<&Path>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let guard = EnvVarGuard::new(&["SPEC_ROOT"]);

        match value {
            Some(path) => guard.set("SPEC_ROOT", path.to_str().unwrap()),
            None => guard.remove("SPEC_ROOT"),
        }

        f()
    }

    #[test]
    fn spec_root_returns_path() {
        with_spec_root(None, || {
            let root = spec_root();
            // Should be an absolute path (or at least exist for the test environment)
            assert!(root.is_absolute() || root.exists());
        });
    }

    #[test]
    fn kernel_for_repo_creates_kernel() {
        with_spec_root(None, || {
            // This test runs in the repo, so the kernel should be constructable
            let kernel = kernel_for_repo().unwrap();
            // The kernel should have the repo's layout
            assert!(kernel.layout().ledger.ends_with("specs/spec_ledger.yaml"));
        });
    }

    #[test]
    fn kernel_with_custom_history_dir() {
        use tempfile::TempDir;

        with_spec_root(None, || {
            let custom_dir = TempDir::new().unwrap();
            let kernel = kernel_with_history_dir(custom_dir.path().to_path_buf()).unwrap();

            assert_eq!(kernel.layout().history_dir, custom_dir.path());
            // Other paths should still use the repo root
            assert!(kernel.layout().ledger.ends_with("specs/spec_ledger.yaml"));
        });
    }

    #[test]
    fn layout_for_repo_returns_standard_paths() {
        with_spec_root(None, || {
            let layout = layout_for_repo();

            // Should return paths matching the standard Rust-as-Spec layout
            assert!(layout.ledger.ends_with("specs/spec_ledger.yaml"));
            assert!(layout.coverage_file.ends_with("target/ac/coverage.jsonl"));
            assert!(layout.junit_file.ends_with("target/junit/acceptance.xml"));
            assert!(layout.history_dir.ends_with("artifacts/ac-status"));
            assert!(layout.features_dir.ends_with("specs/features"));
        });
    }

    #[test]
    fn spec_root_honors_env_var() {
        let custom_root = PathBuf::from("/tmp/custom-spec-root");

        with_spec_root(Some(custom_root.as_path()), || {
            // spec_root() should return the env var value
            let root = spec_root();
            assert_eq!(root, custom_root, "spec_root() should honor SPEC_ROOT env var");

            // layout_for_repo() should use that root for all paths
            let layout = layout_for_repo();
            assert!(
                layout.ledger.starts_with(&custom_root),
                "ledger path should be under SPEC_ROOT: {:?}",
                layout.ledger
            );
            assert!(
                layout.features_dir.starts_with(&custom_root),
                "features_dir should be under SPEC_ROOT: {:?}",
                layout.features_dir
            );
            assert!(
                layout.coverage_file.starts_with(&custom_root),
                "coverage_file should be under SPEC_ROOT: {:?}",
                layout.coverage_file
            );
        });
    }

    #[test]
    fn spec_root_info_detects_valid_repo() {
        with_spec_root(None, || {
            // In the test environment, spec_root should resolve to a valid repo
            let info = spec_root_info();
            assert!(info.valid, "spec_root_info should report valid in test environment");
            assert!(info.missing_files.is_empty(), "No files should be missing");
            assert!(
                info.source == "default (relative to xtask)"
                    || info.source == "SPEC_ROOT environment variable"
            );
        });
    }

    #[test]
    fn spec_root_info_detects_missing_directory() {
        let nonexistent = PathBuf::from("/nonexistent/path/to/repo");

        with_spec_root(Some(nonexistent.as_path()), || {
            let info = spec_root_info();
            assert!(!info.valid, "Should detect invalid path");
            assert!(!info.missing_files.is_empty(), "Should report missing directory");
            assert_eq!(info.source, "SPEC_ROOT environment variable");
        });
    }

    #[test]
    fn validate_spec_root_succeeds_for_valid_repo() {
        with_spec_root(None, || {
            // Should succeed in the test environment
            let result = validate_spec_root();
            assert!(result.is_ok(), "validate_spec_root should succeed in valid repo");
        });
    }

    #[test]
    fn validate_spec_root_fails_with_helpful_message() {
        let invalid = PathBuf::from("/tmp/invalid-spec-root-for-test");

        with_spec_root(Some(invalid.as_path()), || {
            let result = validate_spec_root();
            assert!(result.is_err(), "Should fail for invalid path");

            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.to_lowercase().contains("spec root is invalid"),
                "Error should mention spec root"
            );
            assert!(err_msg.contains("cargo xtask doctor"), "Error should suggest running doctor");
            assert!(err_msg.contains("SPEC_ROOT"), "Error should mention SPEC_ROOT env var");
        });
    }
}
