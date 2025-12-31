#![recursion_limit = "256"]
//! Spec runtime library for the Rust-as-Spec platform.
//!
//! This library provides runtime support for loading, validating, and operating on
//! governance specifications, configuration schemas, and development flows. It serves
//! as the core runtime for spec-driven development workflows.
//!
//! # Main Components
//!
//! - **Configuration**: Schema-driven config validation ([`config`])
//! - **Governance**: Spec ledger, stories, requirements, ACs ([`ledger`])
//! - **Developer Experience**: Flow and command definitions ([`devex`])
//! - **Documentation**: Doc inventory and indexing ([`docs`])
//! - **Governance Graph**: Story → REQ → AC → tests → docs relationships ([`graph`])
//! - **Agent Hints**: AC coverage and task prioritization ([`hints`])
//! - **Tasks**: Work item tracking and sequencing ([`tasks`])
//! - **Schema**: Platform API schemas and endpoints ([`schema`])
//!
//! # Example
//!
//! ```ignore
//! use spec_runtime::{load_all_specs, validate_config};
//! use std::path::Path;
//!
//! let root = Path::new("/workspace");
//! let specs = load_all_specs(root)?;
//! let config = validate_config(
//!     &root.join("specs/config_schema.yaml"),
//!     &root.join("config/local.yaml")
//! )?;
//! ```

use anyhow::Result;
use std::path::Path;

/// Configuration schema validation and runtime config types.
pub mod config;
/// Developer experience flows and xtask command specs.
pub mod devex;
/// Documentation inventory and indexing.
pub mod docs;
/// Governance graph builder (stories → REQs → ACs → tests → docs).
pub mod graph;
/// Agent hint engine for AC coverage and task prioritization.
pub mod hints;
/// Kubernetes IaC configuration support.
pub mod k8s_iac;
/// Spec ledger (stories, requirements, acceptance criteria).
pub mod ledger;
/// Local Docker configuration support.
pub mod local_docker;
/// Platform API schemas and endpoint metadata.
pub mod schema;
/// Service metadata loader.
pub mod service_metadata;
/// Task management and sequencing.
pub mod tasks;
/// UI contract for governed platform UI surfaces.
pub mod ui_contract;

pub use config::{ValidatedConfig, validate_config};
pub use devex::{DevExFlows, load_devex_flows};
pub use docs::{DocEntry, DocIndex, load_doc_index};
pub use graph::{Edge, Graph, Node, build_graph};
pub use hints::{
    AcCoverage, AcCoverageIndex, AcExecutionStatus, Hint, HintEngine, HintFilter, HintKind,
    HintLinks, HintPriority, HintReason, HintStatus, HintTarget, KernelAcStatus,
    ReferentialWarning, build_kernel_ac_statuses,
};
pub use ledger::{
    AcIdIndex, AcceptanceCriterion, ReqIdIndex, Requirement, SpecLedger, Story, build_ac_id_index,
    build_req_id_index, load_spec_ledger,
};
pub use schema::{
    EndpointSchema, PlatformSchemas, SchemaInfo, get_all_schemas, get_schema_by_name,
};
pub use service_metadata::{ServiceMetadata, load_service_metadata};
pub use tasks::*;
pub use ui_contract::{Region, Screen, UiContract, load_ui_contract};

// Re-export RepoContext for convenience
pub use gov_model::RepoContext;

/// Load all governance specs from the workspace root.
///
/// This is a convenience function that loads the three core spec files:
/// - `specs/spec_ledger.yaml` (governance ledger)
/// - `specs/devex_flows.yaml` (developer flows)
/// - `specs/doc_index.yaml` (documentation index)
///
/// # Arguments
///
/// * `root` - Workspace root directory containing the `specs/` folder
///
/// # Returns
///
/// Returns [`AllSpecs`] containing the parsed spec files.
///
/// # Errors
///
/// Returns an error if any spec file is missing, malformed, or fails validation.
///
/// # Example
///
/// ```ignore
/// let specs = load_all_specs(Path::new("/workspace"))?;
/// println!("Loaded {} stories", specs.ledger.stories.len());
/// ```
pub fn load_all_specs(root: &Path) -> Result<AllSpecs> {
    Ok(AllSpecs {
        ledger: load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?,
        devex: load_devex_flows(&root.join("specs/devex_flows.yaml"))?,
        docs: load_doc_index(&root.join("specs/doc_index.yaml"))?,
    })
}

/// Load all governance specs using a `RepoContext`.
///
/// This variant uses [`gov_model::RepoContext`] for path resolution instead of
/// explicit root directory. This is the preferred approach for kernel crates
/// that need unified workspace path handling.
///
/// Loads the three core spec files:
/// - `specs/spec_ledger.yaml` (governance ledger)
/// - `specs/devex_flows.yaml` (developer flows)
/// - `specs/doc_index.yaml` (documentation index)
///
/// # Arguments
///
/// * `ctx` - Repository context providing workspace paths
///
/// # Returns
///
/// Returns [`AllSpecs`] containing the parsed spec files.
///
/// # Errors
///
/// Returns an error if any spec file is missing, malformed, or fails validation.
///
/// # Example
///
/// ```ignore
/// use gov_model::RepoContext;
///
/// let ctx = RepoContext::new("/workspace");
/// let specs = load_all_specs_with_context(&ctx)?;
/// println!("Loaded {} stories", specs.ledger.stories.len());
/// ```
pub fn load_all_specs_with_context(ctx: &gov_model::RepoContext) -> Result<AllSpecs> {
    Ok(AllSpecs {
        ledger: load_spec_ledger(&ctx.spec_ledger_path())?,
        devex: load_devex_flows(&ctx.devex_flows_path())?,
        docs: load_doc_index(&ctx.doc_index_path())?,
    })
}

// =============================================================================
// Individual loader _with_context variants
// =============================================================================

/// Load spec ledger using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::spec_ledger_path`].
pub fn load_spec_ledger_with_context(ctx: &gov_model::RepoContext) -> Result<SpecLedger> {
    load_spec_ledger(&ctx.spec_ledger_path())
}

/// Load devex flows using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::devex_flows_path`].
pub fn load_devex_flows_with_context(ctx: &gov_model::RepoContext) -> Result<DevExFlows> {
    load_devex_flows(&ctx.devex_flows_path())
}

/// Load doc index using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::doc_index_path`].
pub fn load_doc_index_with_context(ctx: &gov_model::RepoContext) -> Result<DocIndex> {
    load_doc_index(&ctx.doc_index_path())
}

/// Load tasks spec using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::tasks_path`].
pub fn load_tasks_with_context(ctx: &gov_model::RepoContext) -> Result<TasksSpec> {
    load_tasks(&ctx.tasks_path())
}

/// Load service metadata using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::service_metadata_path`].
pub fn load_service_metadata_with_context(ctx: &gov_model::RepoContext) -> Result<ServiceMetadata> {
    load_service_metadata(&ctx.service_metadata_path())
}

/// Load UI contract using RepoContext.
///
/// Convenience wrapper that uses [`RepoContext::ui_contract_path`].
pub fn load_ui_contract_with_context(ctx: &gov_model::RepoContext) -> Result<UiContract> {
    load_ui_contract(&ctx.ui_contract_path())
}

/// Validate config using RepoContext.
///
/// Uses the config schema from [`RepoContext::config_schema_path`] and
/// validates against the specified config file.
pub fn validate_config_with_context(
    ctx: &gov_model::RepoContext,
    config_path: &std::path::Path,
) -> Result<ValidatedConfig> {
    validate_config(&ctx.config_schema_path(), config_path)
}

/// Container for all governance specs.
///
/// This struct aggregates the core spec files used by the platform runtime:
/// spec ledger (governance), devex flows (workflows), and doc index (documentation).
pub struct AllSpecs {
    /// Governance spec ledger (stories, requirements, acceptance criteria).
    pub ledger: SpecLedger,
    /// Developer experience flows and xtask command definitions.
    pub devex: DevExFlows,
    /// Documentation inventory and metadata.
    pub docs: DocIndex,
}
