use adapters_spec_fs::FsGovernanceRepository;
use app_http::{AppState, PlatformAuthConfig};
use app_http::security::PlatformAuthMode;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

fn test_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

// Security fix verification: UI routes must be protected when auth is enabled.
#[tokio::test]
async fn test_ui_dashboard_protected_when_auth_enabled() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));

    // Configure Basic Auth
    let mut settings = HashMap::new();
    settings.insert("platform.auth_mode".to_string(), serde_yaml::Value::String("basic".to_string()));

    let mut secrets = HashMap::new();
    secrets.insert("platform.auth_token".to_string(), "test-token".to_string());

    let config = ValidatedConfig {
        env: Some("test".to_string()),
        http_port: 8080,
        settings,
        secrets,
    };

    // Construct app state manually to bypass load_valid_config
    let platform_auth = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: Some("test-token".to_string()),
        jwt_secret: None,
    };

    let app_state = AppState {
        governance_repo: repo,
        workspace_root: workspace_root.clone(),
        config: Some(config),
        platform_auth,
        cors_config: app_http::CorsConfig::default(),
        security_headers_config: app_http::SecurityHeadersConfig::default(),
        repo_context: gov_model::RepoContext::new(&workspace_root),
    };

    let app = app_http::app_with_state(app_state);

    // 1. Request dashboard without auth header -> 401 Unauthorized
    let response = app.clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "UI should be protected");

    // 2. Request dashboard with valid x-platform-token -> 200 OK
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-platform-token", "test-token")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "UI should be accessible with valid token");
}
