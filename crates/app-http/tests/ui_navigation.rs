use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scraper::{Html, Selector};
use std::sync::Arc;
use tower::ServiceExt;

// Helper to resolve workspace root from test binary location
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
async fn test_active_nav_highlight() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));

    // Define pages and their expected active link href
    let pages = vec![
        ("/", "/"),
        ("/ui/graph", "/ui/graph"),
        ("/ui/flows", "/ui/flows"),
        ("/ui/coverage", "/ui/coverage"),
    ];

    for (route, active_href) in pages {
        // Create app fresh for each request to ensure isolation if needed,
        // though strictly not necessary here it's safer given app consumption by oneshot.
        let app = app_with_workspace_root(repo.clone(), workspace_root.clone()).expect("valid config");
        let html = fetch_html(app, route).await;
        let document = Html::parse_document(&html);

        // Select the active link
        let selector = Selector::parse(&format!("nav a[href='{}']", active_href)).unwrap();
        let selection = document.select(&selector).next();

        assert!(selection.is_some(), "Link with href='{}' not found on page '{}'", active_href, route);

        let element = selection.unwrap();
        let classes = element.value().attr("class").unwrap_or("");

        assert!(classes.contains("active"), "Link '{}' on page '{}' missing 'active' class. Found: '{}'", active_href, route, classes);

        let aria_current = element.value().attr("aria-current");
        assert_eq!(aria_current, Some("page"), "Link '{}' on page '{}' missing or incorrect aria-current", active_href, route);
    }
}
