use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scraper::{Html, Selector};
use std::sync::Arc;
use tower::ServiceExt;

fn test_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

async fn fetch_html(app: axum::Router, route: &str) -> String {
    let response =
        app.oneshot(Request::builder().uri(route).body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Route {} should return 200 OK", route);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

#[tokio::test]
async fn test_coverage_page_accessibility() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // 1. Verify Search Input Label
    let input_selector = Selector::parse("input#search-box").unwrap();
    let label_selector = Selector::parse("label[for='search-box']").unwrap();

    let _input = document.select(&input_selector).next().expect("Search input should exist");
    let label = document.select(&label_selector).next().expect("Label for search input should exist");

    assert!(label.value().classes().any(|c| c == "sr-only"), "Label should be visually hidden");

    // 2. Verify Filter Buttons ARIA
    let btn_selector = Selector::parse(".filter-btn").unwrap();
    let buttons: Vec<_> = document.select(&btn_selector).collect();

    assert!(!buttons.is_empty(), "Filter buttons should exist");

    for btn in buttons {
        let aria_pressed = btn.value().attr("aria-pressed");
        assert!(aria_pressed.is_some(), "Filter button should have aria-pressed attribute");

        let id = btn.value().attr("id").unwrap_or("");
        if id == "filter-all" {
             assert_eq!(aria_pressed, Some("true"), "Filter 'All' should be pressed by default");
        } else {
             assert_eq!(aria_pressed, Some("false"), "Other filters should not be pressed by default");
        }
    }
}
