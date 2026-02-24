use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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

#[tokio::test]
async fn metrics_endpoint_returns_prometheus_payload() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let health_response = app
        .clone()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);

    let response =
        app.oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains("http_requests_total"));
    assert!(body_text.contains("http_request_duration_seconds"));
}

#[tokio::test]
async fn metrics_middleware_tracks_health_requests() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let health_response = app
        .clone()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);

    let metrics_response =
        app.oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(metrics_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(metrics_response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains("path=\"/health\""));
    assert!(body_text.contains("method=\"GET\""));
    assert!(body_text.contains("status=\"200\""));
}
