//! UI Accessibility Tests
//!
//! These tests verify that the UI pages render with appropriate accessibility attributes
//! directly in the HTML response, preventing FOUC and improving screen reader experience.
//!
//! @AC-TPL-PLATFORM-UI-ACCESSIBILITY: UI has correct accessible roles and states

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
async fn test_coverage_filters_have_aria_pressed_initial_state() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Filter Buttons
    let filter_all = document
        .select(&Selector::parse("#filter-all").unwrap())
        .next()
        .expect("Should have #filter-all");
    assert_eq!(
        filter_all.value().attr("aria-pressed"),
        Some("true"),
        "#filter-all must have aria-pressed=true initially"
    );

    let filter_passing = document
        .select(&Selector::parse("#filter-passing").unwrap())
        .next()
        .expect("Should have #filter-passing");
    assert_eq!(
        filter_passing.value().attr("aria-pressed"),
        Some("false"),
        "#filter-passing must have aria-pressed=false initially"
    );

    // Search Box
    let search_box = document
        .select(&Selector::parse("#search-box").unwrap())
        .next()
        .expect("Should have #search-box");
    assert_eq!(
        search_box.value().attr("aria-label"),
        Some("Search coverage criteria"),
        "Search box should have an aria-label"
    );

    // Table Container
    let table_container = document
        .select(&Selector::parse("#table-container").unwrap())
        .next()
        .expect("Should have #table-container");
    assert_eq!(
        table_container.value().attr("aria-live"),
        Some("polite"),
        "Table container should have aria-live=polite"
    );
}
