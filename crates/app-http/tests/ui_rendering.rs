use app_http::app_with_workspace_root;
use adapters_spec_fs::FsGovernanceRepository;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scraper::{Html, Selector};
use std::sync::Arc;
use tower::ServiceExt;

/// Helper to resolve workspace root from test binary location
fn test_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Fetch HTML from a route
async fn fetch_html(app: axum::Router, route: &str) -> String {
    let response =
        app.oneshot(Request::builder().uri(route).body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Route {} should return 200 OK", route);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

#[tokio::test]
async fn test_nav_active_state() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch dashboard HTML
    let html = fetch_html(app, "/").await;
    let document = Html::parse_document(&html);

    // Selector for the nav links
    let nav_selector = Selector::parse("nav a").unwrap();
    let links: Vec<_> = document.select(&nav_selector).collect();

    // Helper to find a link by text
    let find_link = |text: &str| {
        links.iter()
            .find(|el| el.text().collect::<String>() == text)
            .unwrap_or_else(|| panic!("Link with text '{}' not found", text))
    };

    // 1. Verify "Dashboard" is active
    let dashboard_link = find_link("Dashboard");
    assert_eq!(dashboard_link.value().attr("aria-current"), Some("page"), "Dashboard link should have aria-current='page'");
    assert!(dashboard_link.value().classes().any(|c| c == "active"), "Dashboard link should have 'active' class");

    // 2. Verify "Graph" is NOT active
    let graph_link = find_link("Graph");
    assert_eq!(graph_link.value().attr("aria-current"), None, "Graph link should NOT have aria-current");
    assert!(!graph_link.value().classes().any(|c| c == "active"), "Graph link should NOT have 'active' class");

    // 3. Verify "Flows & Tasks" is NOT active
    let flows_link = find_link("Flows & Tasks");
    assert_eq!(flows_link.value().attr("aria-current"), None);
}

#[tokio::test]
async fn test_nav_active_state_on_graph_page() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch graph HTML
    let html = fetch_html(app, "/ui/graph").await;
    let document = Html::parse_document(&html);

    // Selector for the nav links
    let nav_selector = Selector::parse("nav a").unwrap();
    let links: Vec<_> = document.select(&nav_selector).collect();

    let find_link = |text: &str| {
        links.iter()
            .find(|el| el.text().collect::<String>() == text)
            .unwrap_or_else(|| panic!("Link with text '{}' not found", text))
    };

    // 1. Verify "Graph" is active
    let graph_link = find_link("Graph");
    assert_eq!(graph_link.value().attr("aria-current"), Some("page"));
    assert!(graph_link.value().classes().any(|c| c == "active"));

    // 2. Verify "Dashboard" is NOT active
    let dashboard_link = find_link("Dashboard");
    assert_eq!(dashboard_link.value().attr("aria-current"), None);
}
