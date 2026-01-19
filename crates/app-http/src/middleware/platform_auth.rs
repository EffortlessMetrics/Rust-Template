use axum::http::{Method, Request};
use axum::{
    body::Body, extract::State, http::StatusCode, middleware::Next, response::IntoResponse,
    response::Response,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::{AppError, AppState, ErrorCode};

pub const PLATFORM_AUTH_HEADER: &str = "x-platform-token";
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// Enforces platform auth for write endpoints when PLATFORM_AUTH_MODE requires auth.
pub async fn platform_auth_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !state.platform_auth.requires_auth() {
        return Ok(next.run(request).await);
    }

    // Only bypass OPTIONS for CORS preflight
    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let provided = extract_auth_token(&request);

    if state.platform_auth.is_authorized(provided.as_deref()) {
        return Ok(next.run(request).await);
    }

    let error = AppError::new(
        StatusCode::UNAUTHORIZED,
        ErrorCode::Unauthorized,
        "Unauthorized: missing or invalid platform token",
    );

    // If in Basic mode, return WWW-Authenticate header to trigger browser prompt
    if state.platform_auth.mode == crate::security::PlatformAuthMode::Basic {
        let mut response = error.into_response();
        response.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_static("Basic realm=\"Platform\""),
        );
        return Ok(response);
    }

    Err(error)
}

/// Extract authentication token, preferring Authorization over the legacy header.
fn extract_auth_token(request: &Request<Body>) -> Option<String> {
    if let Some(auth_val) = request.headers().get(AUTHORIZATION_HEADER) {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) =
                auth_str.strip_prefix("Bearer ").or_else(|| auth_str.strip_prefix("bearer "))
            {
                return Some(token.to_string());
            }

            if let Some(basic) =
                auth_str.strip_prefix("Basic ").or_else(|| auth_str.strip_prefix("basic "))
            {
                if let Ok(decoded) = STANDARD.decode(basic) {
                    if let Ok(credentials) = String::from_utf8(decoded) {
                        // Standard Basic Auth is "username:password"
                        // We extract the password part as the token
                        if let Some((_user, pass)) = credentials.split_once(':') {
                            return Some(pass.to_string());
                        } else {
                            // If no colon, treat the whole string as the password/token
                            return Some(credentials);
                        }
                    }
                }
            }
        }
    }

    request
        .headers()
        .get(PLATFORM_AUTH_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
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
    use jsonwebtoken::{EncodingKey, Header};
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

    fn app_state(
        mode: crate::security::PlatformAuthMode,
        token: Option<&str>,
        jwt_secret: Option<&str>,
    ) -> AppState {
        let workspace_root = PathBuf::new();
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
            security_headers_config: crate::middleware::SecurityHeadersConfig::default(),
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
        // Verify WWW-Authenticate header
        assert!(response.headers().contains_key(axum::http::header::WWW_AUTHENTICATE));
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
        assert!(response.headers().contains_key(axum::http::header::WWW_AUTHENTICATE));
    }

    #[tokio::test]
    async fn accepts_jwt_when_basic_token_is_enabled() {
        let secret = "test-secret";
        let token =
            crate::security::create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
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
        // JWT mode does not necessarily imply Basic prompt
        assert!(!response.headers().contains_key(axum::http::header::WWW_AUTHENTICATE));
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
        let token =
            crate::security::create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
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
        let token =
            crate::security::create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();
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
            exp: (now - 3600) as usize, // Expired 1 hour ago (beyond leeway)
            iat: (now - 7200) as usize,
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
    async fn accepts_basic_header_auth() {
        // Test standard Basic Auth header: "Basic base64(user:pass)"
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        // user:secret -> base64
        let token = "user:secret";
        let encoded = base64::engine::general_purpose::STANDARD.encode(token);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Basic {}", encoded))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepts_basic_header_auth_no_user() {
        // Test Basic Auth header with no username: "Basic base64(:secret)"
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"), None);
        let app = guarded_router(state);

        // :secret -> base64
        let token = ":secret";
        let encoded = base64::engine::general_purpose::STANDARD.encode(token);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .header(AUTHORIZATION_HEADER, format!("Basic {}", encoded))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
