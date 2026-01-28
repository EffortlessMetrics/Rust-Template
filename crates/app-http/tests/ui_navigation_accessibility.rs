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

#[tokio::test]
async fn test_active_nav_link_has_aria_current() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html_str = String::from_utf8(body.to_vec()).unwrap();
    let document = Html::parse_document(&html_str);

    // Select the link that points to "/"
    let selector = Selector::parse("nav a[href='/']").unwrap();
    let link = document.select(&selector).next().expect("Dashboard link not found");

    // Check for aria-current="page"
    let aria_current = link.value().attr("aria-current");
    assert_eq!(aria_current, Some("page"), "Dashboard link should have aria-current='page' when active");
}
