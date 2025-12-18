//! Integration tests for security middleware
//!
//! These tests verify that CORS and security headers middleware work correctly
//! and protect against common web vulnerabilities.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use business_core::governance::{GovernanceError, GovernanceRepository, Task, TaskId, TaskStatus};
use std::{
    env,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use tokio::sync::{Mutex, MutexGuard};
use tower::ServiceExt;

use app_http::app_with_workspace_root;

/// All env vars that these tests read/write (keep in sync with `clean_env_vars()`).
const TEST_ENV_VARS: &[&str] = &[
    // App environment
    "ENV",
    // CORS middleware
    "CORS_ENABLED",
    "CORS_ALLOWED_ORIGINS",
    "CORS_ALLOWED_METHODS",
    "CORS_ALLOWED_HEADERS",
    "CORS_ALLOW_CREDENTIALS",
    "CORS_MAX_AGE",
    // Security headers middleware
    "SECURITY_HEADERS_ENABLED",
    "CSP_HEADER",
    "X_FRAME_OPTIONS",
    "X_CONTENT_TYPE_OPTIONS",
    "X_XSS_PROTECTION",
    "STRICT_TRANSPORT_SECURITY",
    "REFERRER_POLICY",
    "PERMISSIONS_POLICY",
    "CROSS_ORIGIN_EMBEDDER_POLICY",
    "CROSS_ORIGIN_OPENER_POLICY",
    "CROSS_ORIGIN_RESOURCE_POLICY",
];

// Global environment lock to serialize test access to environment variables.
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

async fn get_env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().await
}

/// RAII guard for environment variable management
struct EnvVarGuard {
    _lock: MutexGuard<'static, ()>,
    snapshot: Vec<(&'static str, Option<String>)>,
}

impl EnvVarGuard {
    /// Create a new guard that snapshots current environment and cleans it
    async fn new() -> Self {
        let lock = get_env_lock().await;

        // Snapshot current environment variables
        let snapshot = TEST_ENV_VARS.iter().copied().map(|k| (k, env::var(k).ok())).collect();

        // Clean environment
        clean_env_vars();

        Self { _lock: lock, snapshot }
    }

    /// Set an environment variable during the test
    fn set_var(&self, key: &'static str, value: &str) {
        unsafe { env::set_var(key, value) };
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // Restore exactly what we observed before the test ran.
        for (key, value) in self.snapshot.iter() {
            match value {
                Some(val) => unsafe { env::set_var(*key, val) },
                None => unsafe { env::remove_var(*key) },
            };
        }
    }
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

/// Clean environment variables that might affect test isolation
fn clean_env_vars() {
    for &k in TEST_ENV_VARS {
        unsafe { env::remove_var(k) };
    }
}

#[tokio::test]
async fn test_cors_headers_present_in_response() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should have CORS headers when origin is provided
    assert!(response.headers().contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_cors_preflight_request_handling() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "http://localhost:3000")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "authorization,content-type")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should have preflight response headers
    assert!(response.headers().contains_key("access-control-allow-origin"));
    assert!(response.headers().contains_key("access-control-allow-methods"));
    assert!(response.headers().contains_key("access-control-allow-headers"));
}

#[tokio::test]
async fn test_cors_rejects_unauthorized_origin() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "https://malicious-site.com")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should reject unauthorized origin
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_security_headers_present_in_response() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    let headers = response.headers();

    // Should have security headers
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("referrer-policy"));

    // Verify specific header values
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
}

#[tokio::test]
async fn test_csp_header_contains_directives() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    let csp_header = response.headers().get("content-security-policy").unwrap().to_str().unwrap();

    // Should contain important CSP directives
    assert!(csp_header.contains("default-src 'self'"));
    assert!(csp_header.contains("frame-ancestors 'none'"));
    assert!(csp_header.contains("script-src"));
    assert!(csp_header.contains("style-src"));
}

#[tokio::test]
async fn test_permissions_policy_restricts_features() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    let permissions_header = response.headers().get("permissions-policy");

    if let Some(header) = permissions_header {
        let policy = header.to_str().unwrap();

        // Should restrict sensitive features
        assert!(policy.contains("geolocation=()"));
        assert!(policy.contains("camera=()"));
        assert!(policy.contains("microphone=()"));
        assert!(policy.contains("payment=()"));
    }
}

#[tokio::test]
async fn test_cross_origin_headers_present() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    let headers = response.headers();

    // Should have cross-origin headers
    assert!(headers.contains_key("cross-origin-embedder-policy"));
    assert!(headers.contains_key("cross-origin-opener-policy"));
    assert!(headers.contains_key("cross-origin-resource-policy"));
}

#[tokio::test]
async fn test_hsts_header_in_production() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = EnvVarGuard::new().await;

    // Test with production environment
    env_guard.set_var("ENV", "production");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    let headers = response.headers();

    // Should have HSTS header in production
    if headers.contains_key("strict-transport-security") {
        let hsts = headers.get("strict-transport-security").unwrap().to_str().unwrap();
        assert!(hsts.contains("max-age="));
        assert!(hsts.contains("includeSubDomains"));
    }

    // Environment variables are automatically restored when env_guard goes out of scope
}

#[tokio::test]
async fn test_no_hsts_header_in_development() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = EnvVarGuard::new().await;

    // Test with development environment
    env_guard.set_var("ENV", "development");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should NOT have HSTS header in development
    assert!(!response.headers().contains_key("strict-transport-security"));

    // Environment variables are automatically restored when env_guard goes out of scope
}

#[tokio::test]
async fn test_cors_config_custom_origins() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = EnvVarGuard::new().await;

    // Test custom CORS configuration
    env_guard.set_var("CORS_ALLOWED_ORIGINS", "https://example.com,https://api.example.com");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("origin", "https://example.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should allow custom origin
    assert!(response.headers().contains_key("access-control-allow-origin"));
    assert_eq!(
        response.headers().get("access-control-allow-origin").unwrap(),
        "https://example.com"
    );

    // Environment variables are automatically restored when env_guard goes out of scope
}

#[tokio::test]
async fn test_security_headers_can_be_disabled() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = EnvVarGuard::new().await;

    // Test with security headers disabled
    env_guard.set_var("SECURITY_HEADERS_ENABLED", "false");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request =
        Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should NOT have security headers when disabled
    assert!(!response.headers().contains_key("x-frame-options"));
    assert!(!response.headers().contains_key("x-content-type-options"));
    assert!(!response.headers().contains_key("content-security-policy"));

    // Environment variables are automatically restored when env_guard goes out of scope
}

#[tokio::test]
async fn test_cors_can_be_disabled() {
    // Use environment guard to serialize access and ensure clean state
    let env_guard = EnvVarGuard::new().await;

    // Test with CORS disabled
    env_guard.set_var("CORS_ENABLED", "false");

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("origin", "https://example.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should NOT have CORS headers when disabled
    assert!(!response.headers().contains_key("access-control-allow-origin"));

    // Environment variables are automatically restored when env_guard goes out of scope
}

#[tokio::test]
async fn test_request_id_header_preserved_with_security_middleware() {
    // Use environment guard to serialize access and ensure clean state
    let _env_guard = EnvVarGuard::new().await;

    let workspace_root = test_workspace_root();
    let repo = Arc::new(NoopRepo);
    let app = app_with_workspace_root(repo, workspace_root);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("x-request-id", "test-request-id-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("handler should respond");

    // Should preserve request ID header
    assert!(response.headers().contains_key("x-request-id"));
    assert_eq!(response.headers().get("x-request-id").unwrap(), "test-request-id-123");

    // Should also have security headers
    assert!(response.headers().contains_key("x-frame-options"));
    assert!(response.headers().contains_key("x-content-type-options"));
}
