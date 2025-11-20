use axum::response::Html;
use maud::{DOCTYPE, Markup, html};
use spec_runtime::load_all_specs;

use super::workspace_root;

/// Shared layout for all UI pages
fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " - Rust-as-Spec Platform" }
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" {}
                style {
                    r#"
                    * { margin: 0; padding: 0; box-sizing: border-box; }
                    body {
                        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        background: #f5f5f5;
                    }
                    .container {
                        max-width: 1200px;
                        margin: 0 auto;
                        padding: 20px;
                    }
                    header {
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        padding: 2rem;
                        margin-bottom: 2rem;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                    }
                    header h1 {
                        font-size: 2rem;
                        margin-bottom: 0.5rem;
                    }
                    header p {
                        opacity: 0.9;
                    }
                    nav {
                        background: white;
                        padding: 1rem;
                        margin-bottom: 2rem;
                        border-radius: 8px;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
                    }
                    nav a {
                        color: #667eea;
                        text-decoration: none;
                        margin-right: 2rem;
                        font-weight: 500;
                    }
                    nav a:hover {
                        text-decoration: underline;
                    }
                    .card {
                        background: white;
                        border-radius: 8px;
                        padding: 1.5rem;
                        margin-bottom: 1.5rem;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
                    }
                    .card h2 {
                        color: #667eea;
                        margin-bottom: 1rem;
                        font-size: 1.5rem;
                    }
                    .metrics {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                        gap: 1rem;
                    }
                    .metric {
                        padding: 1rem;
                        background: #f8f9fa;
                        border-radius: 6px;
                        border-left: 4px solid #667eea;
                    }
                    .metric-label {
                        font-size: 0.875rem;
                        color: #666;
                        margin-bottom: 0.25rem;
                    }
                    .metric-value {
                        font-size: 2rem;
                        font-weight: bold;
                        color: #333;
                    }
                    .status-badge {
                        display: inline-block;
                        padding: 0.25rem 0.75rem;
                        border-radius: 12px;
                        font-size: 0.875rem;
                        font-weight: 500;
                    }
                    .status-pass {
                        background: #d4edda;
                        color: #155724;
                    }
                    .status-fail {
                        background: #f8d7da;
                        color: #721c24;
                    }
                    .status-unknown {
                        background: #fff3cd;
                        color: #856404;
                    }
                    pre {
                        background: #f8f9fa;
                        padding: 1rem;
                        border-radius: 6px;
                        overflow-x: auto;
                    }
                    .mermaid {
                        background: white;
                        padding: 2rem;
                        border-radius: 8px;
                    }
                    "#
                }
                script {
                    "mermaid.initialize({ startOnLoad: true, theme: 'default' });"
                }
            }
            body {
                header {
                    .container {
                        h1 { "🦀 Rust-as-Spec Platform" }
                        p { "Self-Governing Platform Cell - Real-time Governance Dashboard" }
                    }
                }
                nav .container {
                    a href="/" { "Dashboard" }
                    a href="/ui/graph" { "Graph" }
                    a href="/ui/flows" { "Flows & Tasks" }
                    a href="/platform/status" target="_blank" { "API: Status" }
                    a href="/platform/graph" target="_blank" { "API: Graph" }
                }
                main .container {
                    (content)
                }
            }
        }
    }
}

/// Dashboard page
pub async fn dashboard() -> Html<String> {
    let root = workspace_root();
    let status_result = load_all_specs(&root);
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));

    let content = match (status_result, tasks_result) {
        (Ok(specs), Ok(tasks_spec)) => {
            let req_count: usize = specs.ledger.stories.iter().map(|s| s.requirements.len()).sum();
            let ac_count: usize = specs
                .ledger
                .stories
                .iter()
                .flat_map(|s| s.requirements.iter())
                .map(|r| r.acceptance_criteria.len())
                .sum();

            // Read policy status
            let policy_path = root.join("target/policy_status.json");
            let policy_status = std::fs::read_to_string(policy_path)
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|v| v.get("summary").and_then(|s| s.as_str()).map(String::from))
                .unwrap_or_else(|| "unknown".to_string());

            let status_class = match policy_status.as_str() {
                "pass" => "status-pass",
                "fail" => "status-fail",
                _ => "status-unknown",
            };

            html! {
                .card {
                    h2 { "Platform Health" }
                    .metrics {
                        .metric {
                            .metric-label { "Stories" }
                            .metric-value { (specs.ledger.stories.len()) }
                        }
                        .metric {
                            .metric-label { "Requirements" }
                            .metric-value { (req_count) }
                        }
                        .metric {
                            .metric-label { "Acceptance Criteria" }
                            .metric-value { (ac_count) }
                        }
                        .metric {
                            .metric-label { "DevEx Commands" }
                            .metric-value { (specs.devex.commands.len()) }
                        }
                        .metric {
                            .metric-label { "Flows" }
                            .metric-value { (specs.devex.flows.len()) }
                        }
                        .metric {
                            .metric-label { "Tasks" }
                            .metric-value { (tasks_spec.tasks.len()) }
                        }
                        .metric {
                            .metric-label { "Documents" }
                            .metric-value { (specs.docs.docs.len()) }
                        }
                        .metric {
                            .metric-label { "Policies" }
                            .metric-value {
                                span class=(status_class) { (policy_status) }
                            }
                        }
                    }
                }

                .card {
                    h2 { "Governance Contracts" }
                    p { "All governance checks are enforced via " code { "cargo xtask selftest" } ":" }
                    ul style="margin: 1rem 0 0 2rem;" {
                        li { "✅ Core checks (fmt, clippy, tests)" }
                        li { "✅ BDD acceptance tests" }
                        li { "✅ AC status mapping & ADR references" }
                        li { "✅ LLM context bundler" }
                        li { "✅ Policy tests " span class=(status_class) { "(" (policy_status) ")" } }
                        li { "✅ DevEx contract satisfaction" }
                        li { "✅ Graph invariants" }
                    }
                }

                .card {
                    h2 { "Quick Links" }
                    ul style="margin: 1rem 0 0 2rem;" {
                        li { a href="/ui/graph" { "View Governance Graph" } " - Visual map of stories, requirements, and ACs" }
                        li { a href="/ui/flows" { "View Flows & Tasks" } " - Developer workflows and task guidance" }
                        li { a href="/platform/status" target="_blank" { "Platform Status API" } " - JSON metrics for agents" }
                        li { a href="/platform/graph" target="_blank" { "Graph API" } " - Full governance graph as JSON" }
                    }
                }
            }
        }
        _ => {
            html! {
                .card {
                    h2 { "Error" }
                    p { "Failed to load platform specifications. Ensure specs are valid and available." }
                }
            }
        }
    };

    Html(layout("Dashboard", content).into_string())
}

/// Graph visualization page
pub async fn graph_view() -> Html<String> {
    let root = workspace_root();

    let content = match load_all_specs(&root) {
        Ok(specs) => match spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs) {
            Ok(graph) => {
                let mermaid_diagram = graph.to_mermaid();

                html! {
                    .card {
                        h2 { "Governance Graph" }
                        p style="margin-bottom: 1rem;" {
                            "This graph shows the relationships between stories, requirements, acceptance criteria, "
                            "documentation, DevEx commands, and flows."
                        }
                        .mermaid {
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

    Html(layout("Graph", content).into_string())
}

/// Flows and tasks page
pub async fn flows_view() -> Html<String> {
    let root = workspace_root();

    let flows_result = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"));
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));

    let content = match (flows_result, tasks_result) {
        (Ok(devex), Ok(tasks_spec)) => {
            html! {
                .card {
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

                .card {
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

    Html(layout("Flows & Tasks", content).into_string())
}
