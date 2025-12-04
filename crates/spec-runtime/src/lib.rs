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

pub use config::{ValidatedConfig, validate_config};
pub use devex::{DevExFlows, load_devex_flows};
pub use docs::{DocEntry, DocIndex, load_doc_index};
pub use graph::{Edge, Graph, Node, build_graph};
pub use hints::{
    AcCoverage, AcCoverageIndex, AcExecutionStatus, Hint, HintEngine, HintFilter, HintKind,
    HintLinks, HintPriority, HintReason, HintStatus, HintTarget,
};
pub use ledger::{AcceptanceCriterion, Requirement, SpecLedger, Story, load_spec_ledger};
pub use schema::{
    EndpointSchema, PlatformSchemas, SchemaInfo, get_all_schemas, get_schema_by_name,
};
pub use service_metadata::{ServiceMetadata, load_service_metadata};
pub use tasks::*;

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
