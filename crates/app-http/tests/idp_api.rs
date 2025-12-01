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
async fn test_idp_snapshot_returns_200() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_idp_snapshot_has_required_fields() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // Verify required top-level fields
    assert!(snapshot.get("timestamp").is_some(), "Missing timestamp");
    assert!(snapshot.get("template_version").is_some(), "Missing template_version");
    assert!(snapshot.get("governance_health").is_some(), "Missing governance_health");
    assert!(snapshot.get("documentation").is_some(), "Missing documentation");
    assert!(snapshot.get("task_hints").is_some(), "Missing task_hints");
}

#[tokio::test]
async fn test_idp_snapshot_governance_health_structure() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let governance_health = snapshot.get("governance_health").expect("Missing governance_health");

    // Verify governance_health structure
    assert!(governance_health.get("status").is_some(), "Missing status");
    assert!(governance_health.get("ac_coverage").is_some(), "Missing ac_coverage");
    assert!(governance_health.get("spec_counts").is_some(), "Missing spec_counts");

    // Verify status is one of expected values
    let status = governance_health["status"].as_str().unwrap();
    assert!(matches!(status, "healthy" | "degraded" | "failing"), "Unexpected status: {}", status);

    // Verify ac_coverage structure
    let ac_coverage = &governance_health["ac_coverage"];
    assert!(ac_coverage.get("total").is_some(), "Missing ac_coverage.total");
    assert!(ac_coverage.get("passing").is_some(), "Missing ac_coverage.passing");
    assert!(ac_coverage.get("failing").is_some(), "Missing ac_coverage.failing");
    assert!(ac_coverage.get("unknown").is_some(), "Missing ac_coverage.unknown");

    // Verify spec_counts structure
    let spec_counts = &governance_health["spec_counts"];
    assert!(spec_counts.get("stories").is_some(), "Missing spec_counts.stories");
    assert!(spec_counts.get("requirements").is_some(), "Missing spec_counts.requirements");
    assert!(
        spec_counts.get("acceptance_criteria").is_some(),
        "Missing spec_counts.acceptance_criteria"
    );
}

#[tokio::test]
async fn test_idp_snapshot_documentation_structure() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let documentation = snapshot.get("documentation").expect("Missing documentation");

    // Verify documentation structure
    assert!(documentation.get("total").is_some(), "Missing documentation.total");
    assert!(documentation.get("valid").is_some(), "Missing documentation.valid");
    assert!(documentation.get("with_issues").is_some(), "Missing documentation.with_issues");
}

#[tokio::test]
async fn test_idp_snapshot_task_hints_structure() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let task_hints = snapshot.get("task_hints").expect("Missing task_hints");

    // Verify task_hints structure
    assert!(task_hints.get("total_pending").is_some(), "Missing task_hints.total_pending");
    assert!(task_hints.get("total_in_progress").is_some(), "Missing task_hints.total_in_progress");
    assert!(task_hints.get("friction_count").is_some(), "Missing task_hints.friction_count");
    assert!(task_hints.get("question_count").is_some(), "Missing task_hints.question_count");

    // high_priority is optional but should be an array if present
    if let Some(high_priority) = task_hints.get("high_priority") {
        assert!(high_priority.is_array(), "high_priority should be an array");

        // If there are high priority tasks, verify structure
        if let Some(tasks) = high_priority.as_array() {
            for task in tasks {
                assert!(task.get("task_id").is_some(), "Missing task_hint.task_id");
                assert!(task.get("title").is_some(), "Missing task_hint.title");
                assert!(task.get("status").is_some(), "Missing task_hint.status");
                assert!(task.get("requirement_ids").is_some(), "Missing task_hint.requirement_ids");
                assert!(task.get("ac_ids").is_some(), "Missing task_hint.ac_ids");
            }
        }
    }
}

#[tokio::test]
async fn test_idp_snapshot_timestamp_format() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let timestamp = snapshot.get("timestamp").and_then(|v| v.as_str()).expect("Missing timestamp");

    // Verify ISO 8601 format (RFC 3339)
    assert!(
        chrono::DateTime::parse_from_rfc3339(timestamp).is_ok(),
        "Timestamp is not in valid ISO 8601 format: {}",
        timestamp
    );
}

#[tokio::test]
async fn test_idp_snapshot_template_version() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root);

    let response = app
        .oneshot(Request::builder().uri("/platform/idp/snapshot").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let snapshot: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let template_version = snapshot
        .get("template_version")
        .and_then(|v| v.as_str())
        .expect("Missing template_version");

    // Verify version format (e.g., "v3.3.6")
    assert!(
        template_version.starts_with('v') || template_version.chars().next().unwrap().is_numeric(),
        "Template version has unexpected format: {}",
        template_version
    );
}
