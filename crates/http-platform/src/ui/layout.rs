use maud::{DOCTYPE, Markup, html};
use spec_runtime::ServiceMetadata;

/// Shared layout for all UI pages.
pub(super) fn layout(
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
