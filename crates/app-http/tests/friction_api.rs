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
async fn test_get_all_friction_entries() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/friction").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("entries").is_some(), "Response should have 'entries' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let entries = json["entries"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(entries.len() as u64, total, "Total count should match entries length");

    // Verify we have at least the test friction entries
    assert!(total >= 2, "Should have at least 2 friction entries");

    // Verify entries have expected fields
    if let Some(first_entry) = entries.first() {
        assert!(first_entry.get("id").is_some());
        assert!(first_entry.get("date").is_some());
        assert!(first_entry.get("category").is_some());
        assert!(first_entry.get("severity").is_some());
        assert!(first_entry.get("summary").is_some());
        assert!(first_entry.get("description").is_some());
        assert!(first_entry.get("status").is_some());
    }
}

#[tokio::test]
async fn test_get_friction_by_id_success() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/friction/FRICTION-AGENT-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the specific friction entry
    assert_eq!(json["id"].as_str().unwrap(), "FRICTION-AGENT-001");
    assert_eq!(json["category"].as_str().unwrap(), "api");
    assert_eq!(json["severity"].as_str().unwrap(), "high");
    assert_eq!(json["status"].as_str().unwrap(), "resolved");
    assert!(json["summary"].as_str().unwrap().contains("UI/API inconsistency"));
}

#[tokio::test]
async fn test_get_friction_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/friction/FRICTION-NONEXISTENT-999")
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
    assert!(json.get("requestId").is_some(), "Error response should have 'requestId' field");

    let message = json["message"].as_str().unwrap();
    assert!(message.contains("FRICTION-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_friction_entries_sorted_by_date() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/friction").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let entries = json["entries"].as_array().unwrap();

    // Verify entries are sorted by date (most recent first)
    if entries.len() > 1 {
        let dates: Vec<&str> = entries.iter().filter_map(|e| e["date"].as_str()).collect();

        for i in 0..dates.len() - 1 {
            assert!(
                dates[i] >= dates[i + 1],
                "Entries should be sorted by date descending (most recent first)"
            );
        }
    }
}
