//! Integration tests for CORS Vary: Origin header
//!
//! These tests verify that the Vary: Origin header is present when CORS is enabled
//! and an Origin header is provided in the request. This is crucial for correct caching behavior.

use axum::{
    body::Body,
    http::{Method, Request},
};
use business_core::governance::{GovernanceError, GovernanceRepository, Task, TaskId, TaskStatus};
use std::{path::PathBuf, sync::Arc};
use testing::process::EnvVarGuard;
use tower::ServiceExt;

use app_http::app_with_workspace_root;

/// All env vars that these tests read/write.
const TEST_ENV_VARS: &[&str] = &["CORS_ENABLED", "CORS_ALLOWED_ORIGINS"];

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
async fn test_cors_vary_header_present() {
    let _env_guard = clean_test_env();

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should have Vary: Origin header
    assert!(response.headers().contains_key("vary"), "Response missing Vary header");
    let vary = response.headers().get("vary").unwrap().to_str().unwrap();
    assert!(
        vary.to_lowercase().contains("origin"),
        "Vary header '{}' does not contain 'Origin'",
        vary
    );
}

#[tokio::test]
async fn test_cors_vary_header_present_preflight() {
    let _env_guard = clean_test_env();

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "http://localhost:3000")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should have Vary: Origin header
    assert!(response.headers().contains_key("vary"), "Preflight response missing Vary header");
    let vary = response.headers().get("vary").unwrap().to_str().unwrap();
    assert!(
        vary.to_lowercase().contains("origin"),
        "Preflight Vary header '{}' does not contain 'Origin'",
        vary
    );
}
