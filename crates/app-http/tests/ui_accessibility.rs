//! UI Accessibility Tests
//!
//! These tests verify that the actual HTML rendered by /ui* routes
//! meets accessibility standards.

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

/// @AC-TPL-PLATFORM-UI-A11Y: Coverage search input has accessible label
#[tokio::test]
async fn test_coverage_search_has_label() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch coverage HTML
    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Find the search input
    let selector = Selector::parse("#search-box").unwrap();
    let search_input =
        document.select(&selector).next().expect("Search input #search-box should exist");

    // Check for aria-label
    let aria_label = search_input.value().attr("aria-label");
    assert!(aria_label.is_some(), "Search input should have aria-label attribute");
    assert_eq!(
        aria_label.unwrap(),
        "Search acceptance criteria",
        "Search input aria-label should match expected text"
    );
}

/// @AC-TPL-PLATFORM-UI-A11Y: Coverage filter buttons have aria-pressed state
#[tokio::test]
async fn test_coverage_filters_have_aria_pressed() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch coverage HTML
    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Check filter buttons
    let selector = Selector::parse(".filter-btn").unwrap();
    let buttons: Vec<_> = document.select(&selector).collect();

    assert!(!buttons.is_empty(), "Filter buttons should exist");

    for button in buttons {
        let id = button.value().id().unwrap_or("unknown");
        let aria_pressed = button.value().attr("aria-pressed");

        assert!(aria_pressed.is_some(), "Filter button #{} should have aria-pressed attribute", id);

        // specifically verify initial state
        if id == "filter-all" {
            assert_eq!(aria_pressed.unwrap(), "true", "All filter should be pressed by default");
        } else {
            assert_eq!(
                aria_pressed.unwrap(),
                "false",
                "Other filters should not be pressed by default"
            );
        }
    }
}
