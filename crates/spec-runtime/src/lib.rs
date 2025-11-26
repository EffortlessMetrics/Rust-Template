use anyhow::Result;
use std::path::Path;

pub mod config;
pub mod devex;
pub mod docs;
pub mod graph;
pub mod k8s_iac;
pub mod ledger;
pub mod local_docker;
pub mod schema;
pub mod service_metadata;
pub mod tasks;

pub use config::{ValidatedConfig, validate_config};
pub use devex::{DevExFlows, load_devex_flows};
pub use docs::{DocEntry, DocIndex, load_doc_index};
pub use graph::{Edge, Graph, Node, build_graph};
pub use ledger::{AcceptanceCriterion, Requirement, SpecLedger, Story, load_spec_ledger};
pub use schema::{
    EndpointSchema, PlatformSchemas, SchemaInfo, get_all_schemas, get_schema_by_name,
};
pub use service_metadata::{ServiceMetadata, load_service_metadata};
pub use tasks::*;

/// Load all specs from the workspace root
pub fn load_all_specs(root: &Path) -> Result<AllSpecs> {
    Ok(AllSpecs {
        ledger: load_spec_ledger(&root.join("specs/spec_ledger.yaml"))?,
        devex: load_devex_flows(&root.join("specs/devex_flows.yaml"))?,
        docs: load_doc_index(&root.join("specs/doc_index.yaml"))?,
    })
}

pub struct AllSpecs {
    pub ledger: SpecLedger,
    pub devex: DevExFlows,
    pub docs: DocIndex,
}
