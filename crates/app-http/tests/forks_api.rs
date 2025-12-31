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
async fn test_get_all_forks() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("forks").is_some(), "Response should have 'forks' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let forks = json["forks"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(forks.len() as u64, total, "Total count should match forks length");

    // Verify fork entries have expected fields
    if let Some(first_fork) = forks.first() {
        assert!(first_fork.get("id").is_some());
        assert!(first_fork.get("name").is_some());
        assert!(first_fork.get("domain").is_some());
        assert!(first_fork.get("status").is_some());
        assert!(first_fork.get("kernel_version").is_some());
    }
}

#[tokio::test]
async fn test_get_fork_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/forks/FORK-NONEXISTENT-999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify error response structure
    assert!(json.get("error").is_some(), "Error response should have 'error' field");
    assert!(json.get("message").is_some(), "Error response should have 'message' field");

    let message = json["message"].as_str().unwrap();
    assert!(message.contains("FORK-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_forks_response_is_valid_json() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

    // Verify we can parse the response as JSON
    let _json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");
}

#[tokio::test]
async fn test_forks_sorted_by_id() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let forks = json["forks"].as_array().unwrap();

    // Verify forks are sorted by ID
    if forks.len() > 1 {
        let ids: Vec<&str> = forks.iter().filter_map(|f| f["id"].as_str()).collect();

        for i in 0..ids.len() - 1 {
            assert!(
                ids[i] <= ids[i + 1],
                "Forks should be sorted by ID ascending (got {} before {})",
                ids[i],
                ids[i + 1]
            );
        }
    }
}
