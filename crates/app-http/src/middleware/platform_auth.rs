use axum::http::{Method, Request};
use axum::{body::Body, extract::State, http::StatusCode, middleware::Next, response::Response};
#[cfg(test)]
use http_auth_token::AUTHORIZATION_HEADER;
use http_auth_token::extract_auth_token_from_headers;

use crate::{AppError, AppState, ErrorCode};

pub use http_auth_token::PLATFORM_AUTH_HEADER;

/// Enforces platform auth for write endpoints when PLATFORM_AUTH_MODE requires auth.
pub async fn platform_auth_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Skip auth if mode is Open OR if credentials aren't configured
    // (can't enforce auth without credentials to validate against)
    if !state.platform_auth.can_enforce_auth() {
        return Ok(next.run(request).await);
    }

    // Only OPTIONS (preflight) is public; all other methods require auth
    if matches!(request.method(), &Method::OPTIONS) {
        return Ok(next.run(request).await);
    }

    let provided = extract_auth_token(&request);

    if state.platform_auth.is_authorized(provided) {
        return Ok(next.run(request).await);
    }

    Err(AppError::new(
        StatusCode::UNAUTHORIZED,
        ErrorCode::Unauthorized,
        "Unauthorized: missing or invalid platform token",
    ))
}

/// Extract authentication token, preferring Authorization over the legacy header.
fn extract_auth_token(request: &Request<Body>) -> Option<&str> {
    extract_auth_token_from_headers(request.headers())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode},
        routing::get,
    };
    use business_core::governance::{
        GovernanceError, GovernanceRepository, Task, TaskId, TaskStatus,
    };
    use jsonwebtoken::{EncodingKey, Header, encode};
    use std::{
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tower::ServiceExt;

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

    async fn protected_handler() -> &'static str {
        "ok"
    }

    fn create_jwt_token(
        secret: &str,
        subject: &str,
        issuer: &str,
        expires_in_seconds: u64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let claims = crate::security::Claims {
            sub: subject.to_string(),
            exp: now + expires_in_seconds,
            iat: now,
            iss: issuer.to_string(),
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
    }

    fn app_state(
        mode: crate::security::PlatformAuthMode,
        token: Option<&str>,
        jwt_secret: Option<&str>,
    ) -> AppState {
        let workspace_root = PathBuf::new();
        let security_headers_config = crate::middleware::SecurityHeadersConfig::default();
        let cached_security_headers = std::sync::Arc::new(crate::middleware::security_headers::CachedSecurityHeaders::new(&security_headers_config));
        AppState {
            governance_repo: Arc::new(NoopRepo),
            workspace_root: workspace_root.clone(),
            config: None,
            platform_auth: crate::security::PlatformAuthConfig {
                mode,
                token: token.map(|t| t.to_string()),
                jwt_secret: jwt_secret.map(|s| s.to_string()),
            },
            cors_config: crate::middleware::CorsConfig::default(),
            security_headers_config,
            cached_security_headers,
            repo_context: gov_model::RepoContext::new(&workspace_root),
        }
    }

    fn guarded_router(state: AppState) -> Router {
        Router::new()
            .route("/platform/protected", get(protected_handler).post(protected_handler))
            .layer(axum::middleware::from_fn_with_state(state.clone(), platform_auth_guard))
            .with_state(state)
    }

    #[tokio::test]
    async fn rejects_post_without_token_in_basic_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_post_with_correct_token() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(PLATFORM_AUTH_HEADER, "secret")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_get_without_auth_in_basic_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_get_with_correct_token_in_basic_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .header(PLATFORM_AUTH_HEADER, "secret")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepts_jwt_when_basic_token_is_enabled() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
        let state =
            app_state(crate::security::PlatformAuthMode::Basic, Some("legacy-token"), Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_post_without_token_in_jwt_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_post_with_basic_token_when_jwt_secret_is_present() {
        let secret = "test-secret";
        let state =
            app_state(crate::security::PlatformAuthMode::Jwt, Some("legacy-token"), Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(PLATFORM_AUTH_HEADER, "legacy-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepts_post_with_valid_jwt_bearer_token() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepts_post_with_valid_jwt_custom_header() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(PLATFORM_AUTH_HEADER, token)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_post_with_invalid_jwt_token() {
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, "Bearer invalid.jwt.token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authorization_header_takes_precedence_over_platform_header() {
        let state = app_state(
            crate::security::PlatformAuthMode::Basic,
            Some("legacy-token"),
            Some("secret"),
        );
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, "Bearer invalid.jwt.token")
            .header(PLATFORM_AUTH_HEADER, "legacy-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_expired_jwt_tokens() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let claims = crate::security::Claims {
            sub: "user123".to_string(),
            exp: now - 3600, // Expired 1 hour ago (beyond leeway)
            iat: now - 7200,
            iss: "rust-template".to_string(),
        };
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        let state =
            app_state(crate::security::PlatformAuthMode::Jwt, Some("legacy-token"), Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_get_without_auth_in_jwt_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_get_with_valid_jwt_in_jwt_mode() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
        let state = app_state(crate::security::PlatformAuthMode::Jwt, None, Some(secret));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
