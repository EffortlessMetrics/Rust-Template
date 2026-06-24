//! UI routes for platform visualization.
//!
//! Provides HTML-based UI for:
//! - Dashboard
//! - Graph visualization
//! - Flows and tasks
//! - AC coverage

use axum::{extract::State, response::Html};
use maud::{DOCTYPE, Markup, html};
use spec_runtime::{ServiceMetadata, load_all_specs, load_service_metadata};
use tracing::instrument;

// ============================================================================
// UI Handlers
// ============================================================================

/// Dashboard page.
#[instrument(skip(state))]
pub async fn dashboard<S>(State(state): State<S>) -> Html<String>
where
    S: super::PlatformState,
{
    let config = super::config_summary(&state);
    let root = state.workspace_root().to_path_buf();

    // Optimize: Offload blocking I/O and parsing to a thread pool
    type DashboardResult = (
        Result<spec_runtime::AllSpecs, spec_runtime::SpecError>,
        Result<spec_runtime::TasksSpec, spec_runtime::SpecError>,
        Option<ServiceMetadata>,
        String,
        usize,
        usize,
        usize,
        usize,
    );

    let (
        status_result,
        tasks_result,
        metadata,
        policy_status,
        passing,
        failing,
        unknown,
        coverage_rows,
    ) = tokio::task::spawn_blocking::<_, DashboardResult>(move || {
        let status_result = load_all_specs(&root);
        let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));
        let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

        // Read policy status
        let policy_path = root.join("target/policy_status.json");
        let policy_status = std::fs::read_to_string(policy_path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|v| v.get("summary").and_then(|s| s.as_str()).map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

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

        (
            status_result,
            tasks_result,
            metadata,
            policy_status,
            passing,
            failing,
            unknown,
            coverage_rows,
        )
    })
    .await
    .unwrap();

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

            let status_class = match policy_status.as_str() {
                "pass" => "status-pass",
                "fail" => "status-fail",
                _ => "status-unknown",
            };

            let mut unknown = unknown;

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

/// Graph visualization page.
#[instrument(skip(state))]
pub async fn graph_view<S>(State(state): State<S>) -> Html<String>
where
    S: super::PlatformState,
{
    let root = state.workspace_root().to_path_buf();

    // Optimize: Offload blocking I/O and graph building to a thread pool
    type GraphViewResult = (Option<ServiceMetadata>, Result<String, spec_runtime::SpecError>);

    let (metadata, graph_result) = tokio::task::spawn_blocking::<_, GraphViewResult>(move || {
        let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

        let result = match load_all_specs(&root) {
            Ok(specs) => {
                match spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs) {
                    Ok(graph) => Ok(graph.to_mermaid()),
                    Err(e) => Err(spec_runtime::SpecError::Internal(format!(
                        "Error Building Graph: {:?}",
                        e
                    ))),
                }
            }
            Err(e) => Err(e),
        };

        (metadata, result)
    })
    .await
    .unwrap();

    let content = match graph_result {
        Ok(mermaid_diagram) => {
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
                    h2 { "Error" }
                    pre { (format!("{:?}", e)) }
                }
            }
        }
    };

    Html(layout("Graph", "graph", &metadata, content).into_string())
}

/// Flows and tasks page.
#[instrument(skip(state))]
pub async fn flows_view<S>(State(state): State<S>) -> Html<String>
where
    S: super::PlatformState,
{
    let root = state.workspace_root().to_path_buf();

    // Optimize: Offload blocking I/O and parsing to a thread pool
    type FlowsResult = (
        Option<ServiceMetadata>,
        Result<spec_runtime::DevExFlows, spec_runtime::SpecError>,
        Result<spec_runtime::TasksSpec, spec_runtime::SpecError>,
    );

    let (metadata, flows_result, tasks_result) =
        tokio::task::spawn_blocking::<_, FlowsResult>(move || {
            let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();
            let flows_result = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"));
            let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));
            (metadata, flows_result, tasks_result)
        })
        .await
        .unwrap();

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

/// Coverage details page.
#[instrument(skip(state))]
pub async fn coverage_view<S>(State(state): State<S>) -> Html<String>
where
    S: super::PlatformState,
{
    let root = state.workspace_root().to_path_buf();

    // Optimize: Offload blocking I/O to a thread pool
    let metadata = tokio::task::spawn_blocking(move || {
        load_service_metadata(&root.join("specs/service_metadata.yaml")).ok()
    })
    .await
    .unwrap();

    let content = coverage_content();

    Html(layout("AC Coverage", "coverage", &metadata, content).into_string())
}

// ============================================================================
// Layout and Content Helpers
// ============================================================================

/// Shared layout for all UI pages.
fn layout(
    title: &str,
    page_id: &str,
    metadata: &Option<ServiceMetadata>,
    content: Markup,
) -> Markup {
    let service_name = metadata
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("Rust-as-Spec Platform");
    let service_tagline =
        metadata.as_ref().and_then(|m| m.description.as_deref()).unwrap_or_default();

    let links = metadata.as_ref().map(|m| m.links.clone()).unwrap_or_default();

    let nav_link = |href: &str, text: &str, target_id: &str| {
        let is_active = page_id == target_id;
        html! {
            a href=(href) aria-current=[is_active.then(|| "page")] { (text) }
        }
    };

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " - Rust-as-Spec Platform" }
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" {}
                style { (styles()) }
                script { "mermaid.initialize({ startOnLoad: true, theme: 'default' });" }
            }
            body {
                header data-uiid=(format!("{}.header", page_id)) {
                    .container {
                        h1 { (service_name) }
                        p { (service_tagline) }
                    }
                }
                nav .container data-uiid=(format!("{}.nav", page_id)) {
                    (nav_link("/", "Dashboard", "dashboard"))
                    (nav_link("/ui/graph", "Graph", "graph"))
                    (nav_link("/ui/flows", "Flows & Tasks", "flows"))
                    (nav_link("/ui/coverage", "AC Coverage", "coverage"))
                    a href="/platform/status" target="_blank" { "API: Status" }
                    a href="/platform/graph" target="_blank" { "API: Graph" }
                    @if let Some(runbook) = links.get("kernel_contract") {
                        a href=(runbook) target="_blank" { "Runbook" }
                    }
                    @if let Some(roadmap) = links.get("roadmap") {
                        a href=(roadmap) target="_blank" { "Roadmap" }
                    }
                    @if let Some(agent_guide) = links.get("agent_guide") {
                        a href=(agent_guide) target="_blank" { "Agent Guide" }
                    }
                    @if let Some(feature_status) = links.get("feature_status") {
                        a href=(feature_status) target="_blank" { "Feature Status" }
                    }
                    @if let Some(support) = links.get("support") {
                        a href=(support) target="_blank" { "Platform Support" }
                    }
                }
                main .container {
                    (content)
                }
            }
        }
    }
}

/// CSS styles for UI pages.
fn styles() -> &'static str {
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
    nav a[aria-current="page"] {
        font-weight: 700;
        text-decoration: underline;
        color: #4c51bf;
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

/// Dashboard content markup.
#[allow(clippy::too_many_arguments)]
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

/// Coverage page content markup.
fn coverage_content() -> Markup {
    html! {
        style { (coverage_styles()) }
        script { (coverage_script()) }

        .card data-uiid="coverage.summary" {
            h2 { "AC Coverage Summary" }
            .metrics {
                .metric style="border-left-color: #155724;" {
                    .metric-label { "Passing" }
                    .metric-value style="color: #155724;" id="passing-count" { "..." }
                }
                .metric style="border-left-color: #721c24;" {
                    .metric-label { "Failing" }
                    .metric-value style="color: #721c24;" id="failing-count" { "..." }
                }
                .metric style="border-left-color: #856404;" {
                    .metric-label { "Unknown" }
                    .metric-value style="color: #856404;" id="unknown-count" { "..." }
                }
                .metric {
                    .metric-label { "Total" }
                    .metric-value id="total-count" { "..." }
                }
            }
        }

        .card {
            h2 { "Acceptance Criteria Coverage" }
            .filter-controls data-uiid="coverage.filters" {
                button #filter-all.filter-btn onclick="filterData('all')" { "All" }
                button #filter-passing.filter-btn onclick="filterData('passing')" { "Passing" }
                button #filter-failing.filter-btn onclick="filterData('failing')" { "Failing" }
                button #filter-unknown.filter-btn onclick="filterData('unknown')" { "Unknown" }
                input #search-box.search-box type="text" placeholder="Search by AC ID or title..."
                    oninput="searchData()";
            }

            #table-container data-uiid="coverage.table" {
                table .coverage-table {
                    thead {
                        tr {
                            th { "AC ID" }
                            th { "Title" }
                            th { "Status" }
                            th { "Story" }
                            th { "Requirement" }
                            th { "Scenarios" }
                        }
                    }
                    tbody #coverage-tbody {
                        tr {
                            td colspan="6" style="text-align: center; padding: 2rem; color: #999;" {
                                "Loading coverage data..."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Coverage page CSS styles.
fn coverage_styles() -> &'static str {
    r#"
    .filter-controls {
        margin-bottom: 1.5rem;
        display: flex;
        gap: 1rem;
        align-items: center;
        flex-wrap: wrap;
    }
    .filter-btn {
        padding: 0.5rem 1rem;
        border: 2px solid #667eea;
        background: white;
        color: #667eea;
        border-radius: 6px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.2s;
    }
    .filter-btn:hover {
        background: #667eea;
        color: white;
    }
    .filter-btn.active {
        background: #667eea;
        color: white;
    }
    .search-box {
        flex: 1;
        min-width: 250px;
        padding: 0.5rem 1rem;
        border: 2px solid #ddd;
        border-radius: 6px;
        font-size: 1rem;
    }
    .search-box:focus {
        outline: none;
        border-color: #667eea;
    }
    .coverage-table {
        width: 100%;
        border-collapse: collapse;
        background: white;
    }
    .coverage-table th {
        background: #f8f9fa;
        padding: 0.75rem;
        text-align: left;
        font-weight: 600;
        border-bottom: 2px solid #dee2e6;
        position: sticky;
        top: 0;
    }
    .coverage-table td {
        padding: 0.75rem;
        border-bottom: 1px solid #dee2e6;
        vertical-align: top;
    }
    .coverage-table tr:hover {
        background: #f8f9fa;
    }
    .ac-row {
        transition: opacity 0.2s;
    }
    .ac-row.hidden {
        display: none;
    }
    .scenario-list {
        margin: 0;
        padding-left: 1.5rem;
        font-size: 0.875rem;
    }
    .scenario-list li {
        margin: 0.25rem 0;
    }
    "#
}

/// Coverage page JavaScript.
fn coverage_script() -> &'static str {
    r#"
    let currentFilter = 'all';
    let allData = [];

    // Fetch coverage data on page load
    fetch('/platform/coverage')
        .then(res => res.json())
        .then(data => {
            allData = data.details;
            updateSummary(data.summary);
            renderTable(allData);
        })
        .catch(err => {
            console.error('Failed to load coverage data:', err);
            document.getElementById('table-container').innerHTML =
                '<p style="color: red;">Failed to load coverage data. Please try again.</p>';
        });

    function updateSummary(summary) {
        document.getElementById('passing-count').textContent = summary.passing;
        document.getElementById('failing-count').textContent = summary.failing;
        document.getElementById('unknown-count').textContent = summary.unknown;
        document.getElementById('total-count').textContent = summary.total;
    }

    function filterData(status) {
        currentFilter = status;

        // Update active button
        document.querySelectorAll('.filter-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.getElementById('filter-' + status).classList.add('active');

        // Apply filter
        applyFilters();
    }

    function searchData() {
        applyFilters();
    }

    function applyFilters() {
        const searchTerm = document.getElementById('search-box').value.toLowerCase();
        const rows = document.querySelectorAll('.ac-row');

        rows.forEach(row => {
            const status = row.dataset.status;
            const text = row.textContent.toLowerCase();

            const statusMatch = currentFilter === 'all' || status === currentFilter;
            const searchMatch = searchTerm === '' || text.includes(searchTerm);

            if (statusMatch && searchMatch) {
                row.classList.remove('hidden');
            } else {
                row.classList.add('hidden');
            }
        });
    }

    function renderTable(data) {
        const tbody = document.getElementById('coverage-tbody');
        tbody.innerHTML = '';

        data.forEach(ac => {
            const row = document.createElement('tr');
            row.className = 'ac-row';
            row.dataset.status = ac.status;

            const statusBadge = ac.status === 'passing' ? '✅ pass' :
                               ac.status === 'failing' ? '❌ fail' :
                               '❓ unknown';
            const badgeClass = ac.status === 'passing' ? 'status-pass' :
                              ac.status === 'failing' ? 'status-fail' :
                              'status-unknown';

            const scenarios = ac.scenarios.length > 0
                ? '<ul class="scenario-list">' +
                  ac.scenarios.map(s => '<li>' + s + '</li>').join('') +
                  '</ul>'
                : '<em style="color: #999;">No scenarios</em>';

            row.innerHTML = `
                <td><code>${ac.id}</code></td>
                <td>${ac.title}</td>
                <td><span class="status-badge ${badgeClass}">${statusBadge}</span></td>
                <td><code>${ac.story}</code></td>
                <td><code>${ac.requirement}</code></td>
                <td>${scenarios}</td>
            `;

            tbody.appendChild(row);
        });
    }

    // Initialize with 'all' filter active
    window.addEventListener('DOMContentLoaded', () => {
        document.getElementById('filter-all').classList.add('active');
    });
    "#
}
