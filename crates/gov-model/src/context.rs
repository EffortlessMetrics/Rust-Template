//! Repository context for kernel crates.
//!
//! `RepoContext` is the single source of truth for workspace paths and layout.
//! All kernel crates accept this instead of reinventing path resolution.

use std::path::{Path, PathBuf};

/// Repository context providing workspace paths and configuration.
///
/// This struct is the "glue" that all kernel crates use for path resolution.
/// Instead of each crate figuring out workspace root, they receive this context.
#[derive(Debug, Clone)]
pub struct RepoContext {
    /// Root directory of the workspace (where Cargo.toml is).
    pub workspace_root: PathBuf,
    /// Layout for spec paths (defaults to standard layout).
    pub spec_layout: SpecLayout,
    /// Service identifier (from service_metadata.yaml).
    pub service_id: Option<String>,
}

/// Spec file layout within the workspace.
#[derive(Debug, Clone)]
pub struct SpecLayout {
    /// Directory containing spec files (default: "specs").
    pub specs_dir: String,
    /// Directory containing config files (default: "config").
    pub config_dir: String,
    /// Directory containing policy files (default: "policy").
    pub policy_dir: String,
    /// Directory containing documentation (default: "docs").
    pub docs_dir: String,
}

impl Default for SpecLayout {
    fn default() -> Self {
        Self {
            specs_dir: "specs".to_string(),
            config_dir: "config".to_string(),
            policy_dir: "policy".to_string(),
            docs_dir: "docs".to_string(),
        }
    }
}

impl RepoContext {
    /// Create a new context with the given workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            spec_layout: SpecLayout::default(),
            service_id: None,
        }
    }

    /// Create context with a custom spec layout.
    pub fn with_layout(mut self, layout: SpecLayout) -> Self {
        self.spec_layout = layout;
        self
    }

    /// Set the service identifier.
    pub fn with_service_id(mut self, id: impl Into<String>) -> Self {
        self.service_id = Some(id.into());
        self
    }

    /// Get the specs directory path.
    pub fn specs_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.spec_layout.specs_dir)
    }

    /// Get the config directory path.
    pub fn config_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.spec_layout.config_dir)
    }

    /// Get the policy directory path.
    pub fn policy_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.spec_layout.policy_dir)
    }

    /// Get the docs directory path.
    pub fn docs_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.spec_layout.docs_dir)
    }

    /// Get path to a specific spec file.
    pub fn spec_file(&self, filename: &str) -> PathBuf {
        self.specs_dir().join(filename)
    }

    /// Get the spec ledger path.
    pub fn spec_ledger_path(&self) -> PathBuf {
        self.spec_file("spec_ledger.yaml")
    }

    /// Get the devex flows path.
    pub fn devex_flows_path(&self) -> PathBuf {
        self.spec_file("devex_flows.yaml")
    }

    /// Get the doc index path.
    pub fn doc_index_path(&self) -> PathBuf {
        self.spec_file("doc_index.yaml")
    }

    /// Get the config schema path.
    pub fn config_schema_path(&self) -> PathBuf {
        self.spec_file("config_schema.yaml")
    }

    /// Get the UI contract path.
    pub fn ui_contract_path(&self) -> PathBuf {
        self.spec_file("ui_contract.yaml")
    }

    /// Get the workspace root.
    pub fn root(&self) -> &Path {
        &self.workspace_root
    }
}
