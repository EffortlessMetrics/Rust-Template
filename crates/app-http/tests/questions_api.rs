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
async fn test_get_all_questions() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/questions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("questions").is_some(), "Response should have 'questions' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let questions = json["questions"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(questions.len() as u64, total, "Total count should match questions length");

    // Verify we have at least the example question
    assert!(total >= 1, "Should have at least 1 question entry");

    // Verify questions have expected fields
    if let Some(first_question) = questions.first() {
        assert!(first_question.get("id").is_some());
        assert!(first_question.get("summary").is_some());
        assert!(first_question.get("status").is_some());
        assert!(first_question.get("flow").is_some());
        assert!(first_question.get("phase").is_some());
        assert!(first_question.get("created_at").is_some());
    }
}

#[tokio::test]
async fn test_get_questions_filtered_by_status() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(
            Request::builder().uri("/platform/questions?status=open").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let questions = json["questions"].as_array().unwrap();

    // All returned questions should have status "open"
    for question in questions {
        assert_eq!(question["status"].as_str().unwrap(), "open");
    }
}

#[tokio::test]
async fn test_get_question_by_id_success() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/questions/Q-EXAMPLE-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the specific question entry
    assert_eq!(json["id"].as_str().unwrap(), "Q-EXAMPLE-001");
    assert_eq!(json["context"]["flow"].as_str().unwrap(), "bundle");
    assert_eq!(json["context"]["phase"].as_str().unwrap(), "ac_selection");
    assert!(json["summary"].as_str().unwrap().contains("multiple ACs"));
    assert!(json.get("options").is_some());
    assert!(json.get("recommendation").is_some());
}

#[tokio::test]
async fn test_get_question_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/questions/Q-NONEXISTENT-999")
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
    assert!(message.contains("Q-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_questions_sorted_by_created_at() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone()).expect("valid config");

    let response = app
        .oneshot(Request::builder().uri("/platform/questions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let questions = json["questions"].as_array().unwrap();

    // Verify questions are sorted by created_at (most recent first)
    if questions.len() > 1 {
        let dates: Vec<&str> = questions.iter().filter_map(|q| q["created_at"].as_str()).collect();

        for i in 0..dates.len() - 1 {
            assert!(
                dates[i] >= dates[i + 1],
                "Questions should be sorted by created_at descending (most recent first)"
            );
        }
    }
}
