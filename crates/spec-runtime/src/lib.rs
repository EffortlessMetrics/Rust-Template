use anyhow::Result;
use std::path::Path;

pub mod devex;
pub mod docs;
pub mod graph;
pub mod ledger;
pub mod tasks;

pub use devex::{load_devex_flows, DevExFlows};
pub use docs::{load_doc_index, DocEntry, DocIndex};
pub use graph::{build_graph, Edge, Graph, Node};
pub use ledger::{load_spec_ledger, AcceptanceCriterion, Requirement, SpecLedger, Story};
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
