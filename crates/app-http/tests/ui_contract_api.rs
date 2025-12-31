use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
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
async fn test_get_ui_contract() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/ui/contract").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("schema_version").is_some(), "Response should have 'schema_version' field");
    assert!(
        json.get("template_version").is_some(),
        "Response should have 'template_version' field"
    );
    assert!(json.get("screens").is_some(), "Response should have 'screens' field");

    let screens = json["screens"].as_array().unwrap();
    assert!(!screens.is_empty(), "Should have at least one screen defined");

    // Verify platform_dashboard screen exists
    let dashboard = screens.iter().find(|s| s["id"].as_str() == Some("platform_dashboard"));
    assert!(dashboard.is_some(), "Should have platform_dashboard screen");

    let dashboard = dashboard.unwrap();
    assert!(dashboard.get("route").is_some(), "Screen should have 'route' field");
    assert!(dashboard.get("regions").is_some(), "Screen should have 'regions' field");

    let regions = dashboard["regions"].as_array().unwrap();
    assert!(!regions.is_empty(), "Dashboard should have regions defined");

    // Verify regions have expected fields
    if let Some(first_region) = regions.first() {
        assert!(first_region.get("id").is_some(), "Region should have 'id' field");
        assert!(first_region.get("kind").is_some(), "Region should have 'kind' field");
        assert!(
            first_region.get("description").is_some(),
            "Region should have 'description' field"
        );
    }
}

#[tokio::test]
async fn test_ui_contract_region_ids_are_unique() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/ui/contract").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let screens = json["screens"].as_array().unwrap();

    // Collect all region IDs across all screens
    let mut all_region_ids: Vec<&str> = Vec::new();
    for screen in screens {
        if let Some(regions) = screen["regions"].as_array() {
            for region in regions {
                if let Some(id) = region["id"].as_str() {
                    all_region_ids.push(id);
                }
            }
        }
    }

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for id in &all_region_ids {
        assert!(seen.insert(*id), "Duplicate region ID found: {}", id);
    }

    // Verify we have expected region IDs
    assert!(all_region_ids.contains(&"dashboard.health"), "Should have dashboard.health region");
    assert!(
        all_region_ids.contains(&"dashboard.ac_coverage"),
        "Should have dashboard.ac_coverage region"
    );
    assert!(all_region_ids.contains(&"graph.diagram"), "Should have graph.diagram region");
    assert!(all_region_ids.contains(&"coverage.summary"), "Should have coverage.summary region");
}

#[tokio::test]
async fn test_ui_contract_all_screens_defined() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/ui/contract").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let screens = json["screens"].as_array().unwrap();
    let screen_ids: Vec<&str> = screens.iter().filter_map(|s| s["id"].as_str()).collect();

    // Verify all expected screens are defined
    assert!(screen_ids.contains(&"platform_dashboard"), "Should have platform_dashboard screen");
    assert!(screen_ids.contains(&"governance_graph"), "Should have governance_graph screen");
    assert!(screen_ids.contains(&"flows_tasks"), "Should have flows_tasks screen");
    assert!(screen_ids.contains(&"ac_coverage"), "Should have ac_coverage screen");
}
