//! Spec layout: Path conventions for AC-related artifacts.
//!
//! This module defines the expected file layout for AC governance artifacts.
//! It provides a central place to document and configure where files live,
//! making it easier for other repos to adopt the AC kernel with different layouts.

use std::path::{Path, PathBuf};

/// The expected layout of AC-related files in a repository.
///
/// This structure documents the contract between the AC kernel and the
/// consuming repository. Different repos can use `SpecLayout::for_repo_root()`
/// as a starting point and customize paths as needed.
///
/// # Default Layout
///
/// ```text
/// <repo_root>/
/// ├── specs/
/// │   ├── spec_ledger.yaml     # The spec ledger
/// │   └── features/            # BDD feature files
/// ├── target/
/// │   ├── ac/
/// │   │   └── coverage.jsonl   # BDD coverage output (streaming)
/// │   └── junit/
/// │       └── acceptance.xml   # JUnit XML fallback for test results
/// └── artifacts/
///     └── ac-status/           # Historical ac-status snapshots
/// ```
#[derive(Debug, Clone)]
pub struct SpecLayout {
    /// Path to the spec_ledger.yaml file
    pub ledger: PathBuf,
    /// Path to the coverage.jsonl file produced by BDD tests
    pub coverage_file: PathBuf,
    /// Path to the JUnit XML file for BDD test results (fallback)
    pub junit_file: PathBuf,
    /// Directory containing historical ac-status-*.json snapshots
    pub history_dir: PathBuf,
    /// Directory containing BDD feature files
    pub features_dir: PathBuf,
}

impl SpecLayout {
    /// Create a `SpecLayout` using the default paths relative to a repo root.
    ///
    /// This implements the standard Rust-as-Spec platform layout:
    /// - `specs/spec_ledger.yaml` - the spec ledger
    /// - `specs/features/` - BDD feature files
    /// - `target/ac/coverage.jsonl` - BDD coverage output (streaming)
    /// - `target/junit/acceptance.xml` - JUnit XML fallback
    /// - `artifacts/ac-status/` - historical snapshots
    pub fn for_repo_root(root: &Path) -> Self {
        Self {
            ledger: root.join("specs/spec_ledger.yaml"),
            coverage_file: root.join("target/ac/coverage.jsonl"),
            junit_file: root.join("target/junit/acceptance.xml"),
            history_dir: root.join("artifacts/ac-status"),
            features_dir: root.join("specs/features"),
        }
    }

    /// Create a layout with all paths under a custom base.
    ///
    /// Useful for testing or when the standard layout is nested under a subdir.
    pub fn with_base(base: &Path) -> Self {
        Self::for_repo_root(base)
    }

    /// Check if the spec ledger file exists.
    pub fn has_ledger(&self) -> bool {
        self.ledger.exists()
    }

    /// Check if the coverage file exists and is non-empty.
    pub fn has_coverage(&self) -> bool {
        self.coverage_file.exists()
            && std::fs::metadata(&self.coverage_file).is_ok_and(|m| m.len() > 0)
    }

    /// Check if the JUnit XML file exists.
    pub fn has_junit(&self) -> bool {
        self.junit_file.exists()
    }

    /// Check if the history directory exists.
    pub fn has_history(&self) -> bool {
        self.history_dir.exists() && self.history_dir.is_dir()
    }

    /// Check if the features directory exists.
    pub fn has_features(&self) -> bool {
        self.features_dir.exists() && self.features_dir.is_dir()
    }
}

impl Default for SpecLayout {
    /// Create a layout using the current working directory as the repo root.
    fn default() -> Self {
        Self::for_repo_root(Path::new("."))
    }
}

/// Builder for customizing `SpecLayout` paths.
///
/// Start with the default layout for a repo root, then override individual paths
/// as needed. This is useful for forks or repos that don't follow the standard
/// Rust-as-Spec layout.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use ac_kernel::SpecLayout;
///
/// let layout = SpecLayout::builder(Path::new("/my/repo"))
///     .with_ledger("custom/ledger.yaml")
///     .with_features_dir("custom/features")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SpecLayoutBuilder {
    layout: SpecLayout,
}

impl SpecLayoutBuilder {
    /// Create a new builder starting from the default layout for the given root.
    pub fn new(root: &Path) -> Self {
        Self { layout: SpecLayout::for_repo_root(root) }
    }

    /// Override the path to the spec ledger file.
    pub fn with_ledger(mut self, path: impl Into<PathBuf>) -> Self {
        self.layout.ledger = path.into();
        self
    }

    /// Override the path to the coverage.jsonl file.
    pub fn with_coverage_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.layout.coverage_file = path.into();
        self
    }

    /// Override the path to the JUnit XML file.
    pub fn with_junit_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.layout.junit_file = path.into();
        self
    }

    /// Override the path to the history directory.
    pub fn with_history_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.layout.history_dir = path.into();
        self
    }

    /// Override the path to the features directory.
    pub fn with_features_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.layout.features_dir = path.into();
        self
    }

    /// Build the final `SpecLayout`.
    pub fn build(self) -> SpecLayout {
        self.layout
    }
}

impl SpecLayout {
    /// Create a builder for customizing the layout.
    ///
    /// This starts with the default layout for the given root and allows
    /// individual paths to be overridden.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use ac_kernel::SpecLayout;
    ///
    /// let layout = SpecLayout::builder(Path::new("/my/repo"))
    ///     .with_ledger("/custom/path/to/ledger.yaml")
    ///     .build();
    /// ```
    pub fn builder(root: &Path) -> SpecLayoutBuilder {
        SpecLayoutBuilder::new(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_repo_root_constructs_expected_paths() {
        let root = Path::new("/my/repo");
        let layout = SpecLayout::for_repo_root(root);

        assert_eq!(layout.ledger, PathBuf::from("/my/repo/specs/spec_ledger.yaml"));
        assert_eq!(layout.coverage_file, PathBuf::from("/my/repo/target/ac/coverage.jsonl"));
        assert_eq!(layout.junit_file, PathBuf::from("/my/repo/target/junit/acceptance.xml"));
        assert_eq!(layout.history_dir, PathBuf::from("/my/repo/artifacts/ac-status"));
        assert_eq!(layout.features_dir, PathBuf::from("/my/repo/specs/features"));
    }

    #[test]
    fn default_uses_current_dir() {
        let layout = SpecLayout::default();

        assert_eq!(layout.ledger, PathBuf::from("./specs/spec_ledger.yaml"));
    }

    #[test]
    fn with_base_is_same_as_for_repo_root() {
        let root = Path::new("/custom/base");
        let layout1 = SpecLayout::for_repo_root(root);
        let layout2 = SpecLayout::with_base(root);

        assert_eq!(layout1.ledger, layout2.ledger);
        assert_eq!(layout1.coverage_file, layout2.coverage_file);
        assert_eq!(layout1.junit_file, layout2.junit_file);
        assert_eq!(layout1.history_dir, layout2.history_dir);
        assert_eq!(layout1.features_dir, layout2.features_dir);
    }

    #[test]
    fn has_methods_return_false_for_nonexistent() {
        let layout = SpecLayout::for_repo_root(Path::new("/nonexistent/path"));

        assert!(!layout.has_ledger());
        assert!(!layout.has_coverage());
        assert!(!layout.has_junit());
        assert!(!layout.has_history());
        assert!(!layout.has_features());
    }

    #[test]
    fn builder_starts_with_defaults() {
        let root = Path::new("/my/repo");
        let layout = SpecLayout::builder(root).build();

        // Builder starts with the same defaults as for_repo_root
        let direct = SpecLayout::for_repo_root(root);
        assert_eq!(layout.ledger, direct.ledger);
        assert_eq!(layout.coverage_file, direct.coverage_file);
        assert_eq!(layout.junit_file, direct.junit_file);
        assert_eq!(layout.history_dir, direct.history_dir);
        assert_eq!(layout.features_dir, direct.features_dir);
    }

    #[test]
    fn builder_allows_overriding_individual_paths() {
        let root = Path::new("/my/repo");
        let layout = SpecLayout::builder(root)
            .with_ledger("/custom/ledger.yaml")
            .with_coverage_file("/custom/coverage.jsonl")
            .with_junit_file("/custom/junit.xml")
            .with_history_dir("/custom/history")
            .with_features_dir("/custom/features")
            .build();

        assert_eq!(layout.ledger, PathBuf::from("/custom/ledger.yaml"));
        assert_eq!(layout.coverage_file, PathBuf::from("/custom/coverage.jsonl"));
        assert_eq!(layout.junit_file, PathBuf::from("/custom/junit.xml"));
        assert_eq!(layout.history_dir, PathBuf::from("/custom/history"));
        assert_eq!(layout.features_dir, PathBuf::from("/custom/features"));
    }

    #[test]
    fn builder_allows_partial_overrides() {
        let root = Path::new("/my/repo");
        let layout = SpecLayout::builder(root).with_ledger("/custom/ledger.yaml").build();

        // Only ledger is overridden
        assert_eq!(layout.ledger, PathBuf::from("/custom/ledger.yaml"));
        // Others remain at defaults
        assert_eq!(layout.coverage_file, PathBuf::from("/my/repo/target/ac/coverage.jsonl"));
        assert_eq!(layout.features_dir, PathBuf::from("/my/repo/specs/features"));
    }
}
