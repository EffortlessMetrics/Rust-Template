//! UI Accessibility Tests
//!
//! These tests verify that the UI components have the necessary ARIA attributes
//! and accessibility features.

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

/// @AC-TPL-PLATFORM-UI-A11Y: Coverage filters use aria-pressed
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

    assert!(!buttons.is_empty(), "Should have filter buttons");

    for button in buttons {
        let id = button.value().attr("id").unwrap_or("");
        let aria_pressed = button.value().attr("aria-pressed");

        assert!(aria_pressed.is_some(), "Filter button {} should have aria-pressed attribute", id);

        if id == "filter-all" {
             assert_eq!(aria_pressed, Some("true"), "All filter should be pressed by default");
        } else {
             assert_eq!(aria_pressed, Some("false"), "Other filters should not be pressed by default");
        }
    }
}

/// @AC-TPL-PLATFORM-UI-A11Y: Search input has aria-label
#[tokio::test]
async fn test_search_input_has_aria_label() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    let selector = Selector::parse("#search-box").unwrap();
    let input = document.select(&selector).next().expect("Search box should exist");

    assert!(input.value().attr("aria-label").is_some(), "Search input should have aria-label");
}

/// @AC-TPL-PLATFORM-UI-A11Y: Coverage table has aria-live
#[tokio::test]
async fn test_coverage_table_has_aria_live() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    let selector = Selector::parse("#table-container").unwrap();
    let container = document.select(&selector).next().expect("Table container should exist");

    assert_eq!(container.value().attr("aria-live"), Some("polite"), "Table container should have aria-live='polite'");
}
