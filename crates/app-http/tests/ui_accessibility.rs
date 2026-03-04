//! UI Accessibility Validation Tests
//!
//! These tests verify that UI pages contain the appropriate ARIA attributes for accessibility.

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

/// @AC-TPL-PLATFORM-UI-ACCESSIBILITY: Coverage page has accessible filters
#[tokio::test]
async fn test_coverage_filters_accessibility() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Verify search input has aria-label
    let search_selector = Selector::parse("#search-box").unwrap();
    let search_input =
        document.select(&search_selector).next().expect("Search input #search-box should exist");

    let aria_label = search_input.value().attr("aria-label");
    assert_eq!(
        aria_label,
        Some("Search coverage by AC ID or title"),
        "Search input should have an aria-label attribute"
    );

    // Verify filter buttons have aria-pressed
    let btn_selector = Selector::parse(".filter-btn").unwrap();
    let filter_btns = document.select(&btn_selector).collect::<Vec<_>>();
    assert!(!filter_btns.is_empty(), "Filter buttons should exist");

    for btn in filter_btns {
        let aria_pressed = btn.value().attr("aria-pressed");
        assert!(
            aria_pressed == Some("true") || aria_pressed == Some("false"),
            "Filter button {:?} should have aria-pressed attribute (true or false)",
            btn.value().id()
        );
    }

    // Verify #filter-all starts pressed
    let filter_all_selector = Selector::parse("#filter-all").unwrap();
    let filter_all_btn =
        document.select(&filter_all_selector).next().expect("#filter-all button should exist");
    assert_eq!(
        filter_all_btn.value().attr("aria-pressed"),
        Some("true"),
        "#filter-all should start with aria-pressed='true'"
    );

    // Verify dynamic table container has aria-live="polite"
    let tbody_selector = Selector::parse("#coverage-tbody").unwrap();
    let tbody = document.select(&tbody_selector).next().expect("#coverage-tbody should exist");

    assert_eq!(
        tbody.value().attr("aria-live"),
        Some("polite"),
        "#coverage-tbody should have aria-live='polite'"
    );
}
