//! Integration tests for CORS security vulnerability reproduction
//!
//! These tests verify that wildcard subdomain matching is secure and does not allow
//! partial domain matches (e.g., "evilexample.com" matching "*.example.com").

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use business_core::governance::{GovernanceError, GovernanceRepository, Task, TaskId, TaskStatus};
use std::{path::PathBuf, sync::Arc};
use testing::process::EnvVarGuard;
use tower::ServiceExt;

use app_http::app_with_workspace_root;

/// All env vars that these tests read/write.
const TEST_ENV_VARS: &[&str] = &[
    "ENV",
    "CORS_ENABLED",
    "CORS_ALLOWED_ORIGINS",
];

/// Create an environment guard that clears all test env vars.
fn clean_test_env() -> EnvVarGuard {
    let guard = EnvVarGuard::new(TEST_ENV_VARS);
    for key in TEST_ENV_VARS {
        guard.remove(key);
    }
    guard
}

#[derive(Clone)]
struct NoopRepo;

impl GovernanceRepository for NoopRepo {
    fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
        Err(GovernanceError::TaskNotFound(task_id.clone()))
    }

    fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
        Ok(vec![])
    }

    fn set_task_status(
        &self,
        _task_id: &TaskId,
        _status: TaskStatus,
    ) -> Result<(), GovernanceError> {
        Ok(())
    }
}

fn test_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

#[tokio::test]
async fn test_cors_vulnerability_partial_domain_match() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = clean_test_env();

    // Configure CORS to allow *.example.com
    env_guard.set("CORS_ALLOWED_ORIGINS", "https://*.example.com");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    // Try to access from evilexample.com (partial match suffix)
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "https://evilexample.com")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // BEFORE FIX: This might be 200 OK because of the bug
    // AFTER FIX: This should be 403 Forbidden

    // We assert that it SHOULD be FORBIDDEN. If the bug exists, this assertion will fail.
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "Partial domain match (evilexample.com) should be rejected for wildcard *.example.com");
}

#[tokio::test]
async fn test_cors_valid_subdomain_match() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = clean_test_env();

    // Configure CORS to allow *.example.com
    env_guard.set("CORS_ALLOWED_ORIGINS", "https://*.example.com");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    // Try to access from api.example.com (valid subdomain)
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "https://api.example.com")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should be allowed
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("access-control-allow-origin"));
}
