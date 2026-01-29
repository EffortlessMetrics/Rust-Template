#![recursion_limit = "256"]
//! Spec runtime library for the Rust-as-Spec platform.
//!
//! This library provides runtime support for loading, validating, and operating on
//! governance specifications, configuration schemas, and development flows. It serves
//! as the core runtime for spec-driven development workflows.

pub mod config;
pub mod devex;
pub mod docs;
pub mod error;
pub mod graph;
pub mod hints;
pub mod k8s_iac;
pub mod ledger;
pub mod local_docker;
pub mod schema;
pub mod service_metadata;
pub mod tasks;
pub mod ui_contract;
pub mod validation;

pub use config::{ValidatedConfig, validate_config};
pub use devex::{DevExFlows, load_devex_flows};
pub use docs::{DocEntry, DocIndex, load_doc_index};
pub use error::{Result, SpecError};
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
pub use validation::*;

// Re-export RepoContext for convenience
pub use gov_model::RepoContext;

use std::path::Path;

/// Container for all governance specs.
#[derive(Debug, Clone)]
pub struct AllSpecs {
    /// Governance spec ledger (stories, requirements, acceptance criteria).
    pub ledger: SpecLedger,
    /// Developer experience flows and xtask command definitions.
    pub devex: DevExFlows,
    /// Documentation inventory and metadata.
    pub docs: DocIndex,
}

/// Load all governance specs from the workspace root.
pub fn load_all_specs(root: &Path) -> Result<AllSpecs> {
    Ok(AllSpecs {
        ledger: load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?,
        devex: load_devex_flows(&root.join("specs/devex_flows.yaml"))?,
        docs: load_doc_index(&root.join("specs/doc_index.yaml"))?,
    })
}

/// Load all governance specs using a `RepoContext`.
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
pub fn load_spec_ledger_with_context(ctx: &gov_model::RepoContext) -> Result<SpecLedger> {
    load_spec_ledger(&ctx.spec_ledger_path())
}

/// Load devex flows using RepoContext.
pub fn load_devex_flows_with_context(ctx: &gov_model::RepoContext) -> Result<DevExFlows> {
    load_devex_flows(&ctx.devex_flows_path())
}

/// Load doc index using RepoContext.
pub fn load_doc_index_with_context(ctx: &gov_model::RepoContext) -> Result<DocIndex> {
    load_doc_index(&ctx.doc_index_path())
}

/// Load tasks spec using RepoContext.
pub fn load_tasks_with_context(ctx: &gov_model::RepoContext) -> Result<TasksSpec> {
    load_tasks(&ctx.tasks_path())
}

/// Load service metadata using RepoContext.
pub fn load_service_metadata_with_context(ctx: &gov_model::RepoContext) -> Result<ServiceMetadata> {
    load_service_metadata(&ctx.service_metadata_path())
}

/// Load UI contract using RepoContext.
pub fn load_ui_contract_with_context(ctx: &gov_model::RepoContext) -> Result<UiContract> {
    load_ui_contract(&ctx.ui_contract_path())
}

/// Validate config using RepoContext.
pub fn validate_config_with_context(
    ctx: &gov_model::RepoContext,
    config_path: &std::path::Path,
) -> Result<ValidatedConfig> {
    validate_config(&ctx.config_schema_path(), config_path)
}
