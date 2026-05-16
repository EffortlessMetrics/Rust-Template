use axum::{extract::State, response::Html};
use maud::html;
use spec_runtime::load_service_metadata;
use tracing::instrument;

use super::layout::layout;
use crate::PlatformState;

/// Flows and tasks page.
#[instrument(skip(state))]
pub async fn flows_view<S>(State(state): State<S>) -> Html<String>
where
    S: PlatformState,
{
    let root = state.workspace_root();
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

    let flows_result = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"));
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));

    let content = match (flows_result, tasks_result) {
        (Ok(devex), Ok(tasks_spec)) => {
            html! {
                .card data-uiid="flows.devex" {
                    h2 { "DevEx Flows" }
                    p style="margin-bottom: 1rem;" {
                        "Developer experience flows define common workflows for working with this repository."
                    }
                    @for flow in devex.flows.values() {
                        .metric style="margin-bottom: 1rem;" {
                            h3 style="color: #667eea; font-size: 1.1rem;" { (flow.name) }
                            p style="color: #666; margin: 0.5rem 0;" { (flow.description) }
                            details {
                                summary style="cursor: pointer; color: #667eea;" { "Steps (" (flow.steps.len()) ")" }
                                ol style="margin: 0.5rem 0 0 2rem;" {
                                    @for step in &flow.steps {
                                        li { code { "cargo xtask " (step) } }
                                    }
                                }
                            }
                        }
                    }
                }

                .card data-uiid="flows.tasks" {
                    h2 { "Tasks" }
                    p style="margin-bottom: 1rem;" {
                        "Tasks represent concrete work items with recommended flows and suggested sequences."
                    }
                    @for task in &tasks_spec.tasks {
                        .metric style="margin-bottom: 1rem;" {
                            h3 style="color: #667eea; font-size: 1.1rem;" { (task.title) }
                            p style="color: #666; margin: 0.5rem 0;" { (task.summary) }
                            p style="font-size: 0.875rem; margin: 0.5rem 0;" {
                                strong { "Status: " } (task.status)
                                " | "
                                strong { "Requirement: " } (task.requirement)
                            }
                            details {
                                summary style="cursor: pointer; color: #667eea;" {
                                    "View suggested sequence"
                                }
                                p style="margin: 0.5rem 0; font-size: 0.875rem;" {
                                    "Run: " code { "cargo xtask suggest-next --task " (task.id) }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            html! {
                .card {
                    h2 { "Error" }
                    p { "Failed to load flows or tasks." }
                }
            }
        }
    };

    Html(layout("Flows & Tasks", "flows", &metadata, content).into_string())
}
