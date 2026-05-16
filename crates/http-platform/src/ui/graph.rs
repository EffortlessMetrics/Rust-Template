use axum::{extract::State, response::Html};
use maud::html;
use spec_runtime::{load_all_specs, load_service_metadata};
use tracing::instrument;

use super::layout::layout;
use crate::PlatformState;

/// Graph visualization page.
#[instrument(skip(state))]
pub async fn graph_view<S>(State(state): State<S>) -> Html<String>
where
    S: PlatformState,
{
    let root = state.workspace_root();
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

    let content = match load_all_specs(root) {
        Ok(specs) => match spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs) {
            Ok(graph) => {
                let mermaid_diagram = graph.to_mermaid();

                html! {
                    .card {
                        h2 { "Governance Graph" }
                        p style="margin-bottom: 1rem;" {
                            "This graph shows relationships between stories, requirements, acceptance criteria, "
                            "documentation, DevEx commands, and flows."
                        }
                        .mermaid data-uiid="graph.diagram" {
                            (mermaid_diagram)
                        }
                    }
                }
            }
            Err(e) => {
                html! {
                    .card {
                        h2 { "Error Building Graph" }
                        pre { (format!("{:?}", e)) }
                    }
                }
            }
        },
        Err(e) => {
            html! {
                .card {
                    h2 { "Error Loading Specs" }
                    pre { (format!("{:?}", e)) }
                }
            }
        }
    };

    Html(layout("Graph", "graph", &metadata, content).into_string())
}
