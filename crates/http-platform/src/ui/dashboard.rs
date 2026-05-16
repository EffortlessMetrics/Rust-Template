use axum::{extract::State, response::Html};
use maud::{Markup, html};
use spec_runtime::{load_all_specs, load_service_metadata};
use tracing::instrument;

use super::layout::layout;
use crate::{PlatformState, config_summary};

// ============================================================================
// UI Handlers
// ============================================================================

/// Dashboard page.
#[instrument(skip(state))]
pub async fn dashboard<S>(State(state): State<S>) -> Html<String>
where
    S: PlatformState,
{
    let root = state.workspace_root();
    let status_result = load_all_specs(root);
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();
    let config = config_summary(&state);

    let content = match (status_result, tasks_result) {
        (Ok(specs), Ok(tasks_spec)) => {
            let _req_count: usize = specs.ledger.stories.iter().map(|s| s.requirements.len()).sum();
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

            // Read AC coverage from feature_status.md
            let feature_status_path = root.join("docs/feature_status.md");
            let mut passing = 0;
            let mut failing = 0;
            let mut unknown = 0;
            let mut coverage_rows = 0;

            if feature_status_path.exists()
                && let Ok(content) = std::fs::read_to_string(feature_status_path)
            {
                for line in content.lines() {
                    if !line.starts_with("| AC-") {
                        continue;
                    }

                    coverage_rows += 1;

                    if line.contains("[PASS]") {
                        passing += 1;
                    } else if line.contains("[FAIL]") {
                        failing += 1;
                    } else if line.contains("[UNKNOWN]") {
                        unknown += 1;
                    }
                }
            }

            // If no coverage data, count all ACs as unknown
            if coverage_rows == 0 {
                unknown = ac_count;
            } else {
                let accounted_for = passing + failing + unknown;
                if accounted_for < ac_count {
                    unknown += ac_count - accounted_for;
                }
            }

            dashboard_content(
                specs,
                tasks_spec,
                config,
                policy_status,
                status_class,
                passing,
                failing,
                unknown,
            )
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

    Html(layout("Dashboard", "dashboard", &metadata, content).into_string())
}

fn dashboard_content(
    specs: spec_runtime::AllSpecs,
    tasks_spec: spec_runtime::TasksSpec,
    config: Option<platform_contract::ConfigSummary>,
    policy_status: String,
    status_class: &'static str,
    passing: usize,
    failing: usize,
    unknown: usize,
) -> Markup {
    html! {
        .card data-uiid="dashboard.health" {
            h2 { "Platform Health" }
            .metrics {
                .metric {
                    .metric-label { "Stories" }
                    .metric-value { (specs.ledger.stories.len()) }
                }
                .metric {
                    .metric-label { "Requirements" }
                    .metric-value { (specs.ledger.stories.iter().map(|s| s.requirements.len()).sum::<usize>()) }
                }
                .metric {
                    .metric-label { "Acceptance Criteria" }
                    .metric-value { (specs.ledger.stories.iter().flat_map(|s| s.requirements.iter()).map(|r| r.acceptance_criteria.len()).sum::<usize>()) }
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

        .card data-uiid="dashboard.ac_coverage" {
            h2 { "AC Coverage" }
            .stats style="display: flex; gap: 1.5rem; margin: 1rem 0; flex-wrap: wrap;" {
                span style="color: #155724; font-size: 1.1rem; font-weight: 500;" {
                    "✅ " (passing) " passing"
                }
                span style="color: #721c24; font-size: 1.1rem; font-weight: 500;" {
                    "❌ " (failing) " failing"
                }
                span style="color: #856404; font-size: 1.1rem; font-weight: 500;" {
                    "❓ " (unknown) " unknown"
                }
            }
            p style="margin-top: 1rem;" {
                a href="/ui/coverage" style="color: #667eea; text-decoration: none; font-weight: 500;" {
                    "View details →"
                }
            }
        }

        .card data-uiid="dashboard.config" {
            h2 { "Runtime Config (redacted)" }
            @if let Some(cfg) = config {
                p style="margin-bottom: 0.75rem; color: #555;" {
                    "Config values are rendered for visibility without leaking secrets; tokens are never shown."
                }
                ul style="margin-left: 1.25rem; line-height: 1.6;" {
                    li {
                        strong { "Env: " }
                        (cfg.env.clone().unwrap_or_else(|| "unknown".to_string()))
                    }
                    li { strong { "HTTP port: " } (cfg.http_port) }
                    li {
                        strong { "Auth mode: " }
                        (cfg.auth.mode.clone())
                        " ("
                        @if cfg.auth.token_present { "token configured" } @else { "no token" }
                        ")"
                    }
                }

                @if !cfg.settings.is_empty() {
                    details style="margin-top: 0.75rem;" {
                        summary style="cursor: pointer; color: #667eea;" { "Settings" }
                        ul style="margin: 0.5rem 0 0 1.25rem;" {
                            @for (k, v) in cfg.settings.iter() {
                                li { code { (k) } ": " (v) }
                            }
                        }
                    }
                }

                @if !cfg.secrets_redacted.is_empty() {
                    details style="margin-top: 0.75rem;" {
                        summary style="cursor: pointer; color: #667eea;" { "Secrets (redacted)" }
                        ul style="margin: 0.5rem 0 0 1.25rem;" {
                            @for (k, _) in cfg.secrets_redacted.iter() {
                                li { code { (k) } ": " "[REDACTED]" }
                            }
                        }
                    }
                }
            } @else {
                p { "Configuration details unavailable." }
            }
        }

        .card data-uiid="dashboard.contracts" {
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

        .card data-uiid="dashboard.links" {
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
