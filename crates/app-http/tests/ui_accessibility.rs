//! Accessibility Tests for UI
//!
//! @AC-TPL-PLATFORM-UI-ACCESSIBILITY: UI elements are accessible

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

/// @AC-TPL-PLATFORM-UI-ACCESSIBILITY: Coverage page has accessible controls
#[tokio::test]
async fn test_coverage_controls_are_accessible() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch coverage HTML
    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Check filter buttons
    let button_selector = Selector::parse(".filter-btn").unwrap();
    let buttons: Vec<_> = document.select(&button_selector).collect();
    assert!(!buttons.is_empty(), "Should have filter buttons");

    for button in buttons {
        let text = button.text().collect::<String>();
        let aria_pressed = button.value().attr("aria-pressed");

        assert!(
            aria_pressed.is_some(),
            "Filter button '{}' should have aria-pressed attribute",
            text
        );

        // Check initial state
        if text.contains("All") {
             assert_eq!(aria_pressed, Some("true"), "All filter should be pressed by default");
             assert!(button.value().classes().any(|c| c == "active"), "All filter should have active class");
        } else {
             assert_eq!(aria_pressed, Some("false"), "Filter '{}' should not be pressed by default", text);
        }
    }

    // Check search input
    let input_selector = Selector::parse("#search-box").unwrap();
    let input = document.select(&input_selector).next().expect("Search box should exist");

    let aria_label = input.value().attr("aria-label");
    assert!(
        aria_label.is_some(),
        "Search input should have aria-label"
    );
    assert_eq!(aria_label, Some("Search coverage data"), "Search input should have correct label");
}
