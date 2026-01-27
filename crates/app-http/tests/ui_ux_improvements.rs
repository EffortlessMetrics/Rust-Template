use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
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
async fn test_active_navigation_state_dashboard() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let html = fetch_html(app, "/").await;
    let document = Html::parse_document(&html);

    // Verify Skip Link exists
    let skip_selector = Selector::parse("a.skip-link").unwrap();
    let skip_link = document.select(&skip_selector).next();
    assert!(skip_link.is_some(), "Skip link should exist");
    let skip_link = skip_link.unwrap();
    assert_eq!(skip_link.value().attr("href"), Some("#main-content"), "Skip link should point to main content");

    // Verify Dashboard link is active
    let nav_selector = Selector::parse("nav a").unwrap();
    let links: Vec<_> = document.select(&nav_selector).collect();

    let dashboard_link = links.iter().find(|l| l.text().collect::<String>() == "Dashboard").expect("Dashboard link not found");
    assert!(dashboard_link.value().classes().any(|c| c == "active"), "Dashboard link should have active class");
    assert_eq!(dashboard_link.value().attr("aria-current"), Some("page"), "Dashboard link should have aria-current='page'");

    // Verify Graph link is NOT active
    let graph_link = links.iter().find(|l| l.text().collect::<String>() == "Graph").expect("Graph link not found");
    assert!(!graph_link.value().classes().any(|c| c == "active"), "Graph link should not be active on dashboard");
    assert!(graph_link.value().attr("aria-current").is_none(), "Graph link should not have aria-current on dashboard");
}

#[tokio::test]
async fn test_active_navigation_state_graph() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let html = fetch_html(app, "/ui/graph").await;
    let document = Html::parse_document(&html);

    // Verify Graph link is active
    let nav_selector = Selector::parse("nav a").unwrap();
    let links: Vec<_> = document.select(&nav_selector).collect();

    let graph_link = links.iter().find(|l| l.text().collect::<String>() == "Graph").expect("Graph link not found");
    assert!(graph_link.value().classes().any(|c| c == "active"), "Graph link should be active on graph page");
    assert_eq!(graph_link.value().attr("aria-current"), Some("page"), "Graph link should have aria-current='page'");
}
