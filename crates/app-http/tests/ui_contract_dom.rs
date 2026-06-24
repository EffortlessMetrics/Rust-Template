//! UI Contract DOM Validation Tests
//!
//! These tests verify that the actual HTML rendered by /ui* routes
//! contains the `data-uiid` attributes specified in specs/ui_contract.yaml.
//!
//! @AC-TPL-PLATFORM-UI-CONTRACT: UI spec + anchors + endpoint are governed

use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scraper::{Html, Selector};
use spec_runtime::load_ui_contract;
use std::collections::HashSet;
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

/// Extract all data-uiid attribute values from HTML
fn extract_uiids(html: &str) -> HashSet<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("[data-uiid]").unwrap();

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("data-uiid").map(|s| s.to_string()))
        .collect()
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: Platform dashboard has all contract regions
#[tokio::test]
async fn test_dashboard_has_contract_regions() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Find the dashboard screen in contract
    let dashboard = contract
        .screens
        .iter()
        .find(|s| s.id == "platform_dashboard")
        .expect("Contract should have platform_dashboard screen");

    // Fetch dashboard HTML
    let html = fetch_html(app, &dashboard.route).await;
    let found_uiids = extract_uiids(&html);

    // Verify each region from contract exists in DOM
    let mut missing = Vec::new();
    for region in &dashboard.regions {
        if !found_uiids.contains(&region.id) {
            missing.push(&region.id);
        }
    }

    assert!(
        missing.is_empty(),
        "Dashboard is missing data-uiid attributes for regions: {:?}",
        missing
    );
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: Graph page has all contract regions
#[tokio::test]
async fn test_graph_has_contract_regions() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Find the graph screen in contract
    let graph = contract
        .screens
        .iter()
        .find(|s| s.id == "governance_graph")
        .expect("Contract should have governance_graph screen");

    // Fetch graph HTML
    let html = fetch_html(app, &graph.route).await;
    let found_uiids = extract_uiids(&html);

    // Verify each region from contract exists in DOM
    let mut missing = Vec::new();
    for region in &graph.regions {
        if !found_uiids.contains(&region.id) {
            missing.push(&region.id);
        }
    }

    assert!(
        missing.is_empty(),
        "Graph page is missing data-uiid attributes for regions: {:?}",
        missing
    );
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: Flows page has all contract regions
#[tokio::test]
async fn test_flows_has_contract_regions() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Find the flows screen in contract
    let flows = contract
        .screens
        .iter()
        .find(|s| s.id == "flows_tasks")
        .expect("Contract should have flows_tasks screen");

    // Fetch flows HTML
    let html = fetch_html(app, &flows.route).await;
    let found_uiids = extract_uiids(&html);

    // Verify each region from contract exists in DOM
    let mut missing = Vec::new();
    for region in &flows.regions {
        if !found_uiids.contains(&region.id) {
            missing.push(&region.id);
        }
    }

    assert!(
        missing.is_empty(),
        "Flows page is missing data-uiid attributes for regions: {:?}",
        missing
    );
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: Coverage page has all contract regions
#[tokio::test]
async fn test_coverage_has_contract_regions() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Find the coverage screen in contract
    let coverage = contract
        .screens
        .iter()
        .find(|s| s.id == "ac_coverage")
        .expect("Contract should have ac_coverage screen");

    // Fetch coverage HTML
    let html = fetch_html(app, &coverage.route).await;
    let found_uiids = extract_uiids(&html);

    // Verify each region from contract exists in DOM
    let mut missing = Vec::new();
    for region in &coverage.regions {
        if !found_uiids.contains(&region.id) {
            missing.push(&region.id);
        }
    }

    assert!(
        missing.is_empty(),
        "Coverage page is missing data-uiid attributes for regions: {:?}",
        missing
    );
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: All screens in contract are reachable
#[tokio::test]
async fn test_all_contract_screens_are_reachable() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    for screen in &contract.screens {
        let app = app_with_workspace_root(
            Arc::new(FsGovernanceRepository::new(workspace_root.clone())),
            workspace_root.clone(),
        )
        .expect("valid config");

        let response = app
            .oneshot(Request::builder().uri(&screen.route).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Screen '{}' route '{}' should be reachable",
            screen.id,
            screen.route
        );
    }
}

/// @AC-TPL-PLATFORM-UI-CONTRACT: All regions in contract have matching DOM elements
#[tokio::test]
async fn test_full_contract_coverage() {
    let workspace_root = test_workspace_root();
    let contract = load_ui_contract(&workspace_root.join("specs/ui_contract.yaml"))
        .expect("UI contract should load");

    let mut all_missing: Vec<(String, String)> = Vec::new();

    for screen in &contract.screens {
        let app = app_with_workspace_root(
            Arc::new(FsGovernanceRepository::new(workspace_root.clone())),
            workspace_root.clone(),
        )
        .expect("valid config");

        let html = fetch_html(app, &screen.route).await;
        let found_uiids = extract_uiids(&html);

        for region in &screen.regions {
            if !found_uiids.contains(&region.id) {
                all_missing.push((screen.id.clone(), region.id.clone()));
            }
        }
    }

    assert!(
        all_missing.is_empty(),
        "UI contract has {} missing data-uiid mappings:\n{}",
        all_missing.len(),
        all_missing
            .iter()
            .map(|(s, r)| format!("  - Screen '{}' missing region '{}'", s, r))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// @AC-TPL-PLATFORM-UI-ACCESSIBILITY: Active navigation link has aria-current="page"
#[tokio::test]
async fn test_active_nav_link_has_aria_current() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch dashboard HTML
    let html = fetch_html(app, "/").await;
    let document = Html::parse_document(&html);

    // Find the Dashboard link in the nav
    let selector = Selector::parse("nav a").unwrap();
    let dashboard_link = document
        .select(&selector)
        .find(|el| el.text().collect::<String>() == "Dashboard")
        .expect("Dashboard link should exist");

    // Check for aria-current="page"
    let aria_current = dashboard_link.value().attr("aria-current");
    assert_eq!(
        aria_current,
        Some("page"),
        "Dashboard link on dashboard page should have aria-current='page'"
    );
}

/// @AC-TPL-PLATFORM-UI-ACCESSIBILITY: Filter buttons should have aria-pressed attributes
#[tokio::test]
async fn test_filter_buttons_have_aria_pressed() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    // Fetch coverage HTML
    let html = fetch_html(app, "/ui/coverage").await;
    let document = Html::parse_document(&html);

    // Find the All filter button
    let selector = Selector::parse("button#filter-all").unwrap();
    let all_button = document.select(&selector).next().expect("All filter button should exist");

    // Check for aria-pressed="true" on All
    let aria_pressed = all_button.value().attr("aria-pressed");
    assert_eq!(
        aria_pressed,
        Some("true"),
        "All filter button should have aria-pressed='true' initially"
    );

    // Find the Passing filter button
    let passing_selector = Selector::parse("button#filter-passing").unwrap();
    let passing_button =
        document.select(&passing_selector).next().expect("Passing filter button should exist");

    // Check for aria-pressed="false" on Passing
    let aria_pressed_passing = passing_button.value().attr("aria-pressed");
    assert_eq!(
        aria_pressed_passing,
        Some("false"),
        "Passing filter button should have aria-pressed='false' initially"
    );
}
