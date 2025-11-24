use anyhow::Result;
use std::path::Path;

pub mod devex;
pub mod docs;
pub mod graph;
pub mod ledger;
pub mod tasks;

pub use devex::{DevExFlows, load_devex_flows};
pub use docs::{DocEntry, DocIndex, load_doc_index};
pub use graph::{Edge, Graph, Node, build_graph};
pub use ledger::{AcceptanceCriterion, Requirement, SpecLedger, Story, load_spec_ledger};
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
