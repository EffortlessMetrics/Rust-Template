use axum::{
    Router,
    extract::{Extension, Json},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

// Public modules
pub mod agent;
pub mod errors;
pub mod metrics;
pub mod middleware;
pub mod platform;
pub mod security;
pub mod shutdown;
pub mod tasks;
pub mod todos;

// Re-export commonly used types
pub use errors::{AppError, ErrorCode, ErrorSummary, get_error_summary};
pub use middleware::{REQUEST_ID_HEADER, RequestId};

use business_core::governance::GovernanceRepository;
use gov_model::RepoContext;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub governance_repo: Arc<dyn GovernanceRepository>,
    pub workspace_root: PathBuf,
    pub config: Option<spec_runtime::ValidatedConfig>,
    pub platform_auth: security::PlatformAuthConfig,
    /// CORS configuration
    pub cors_config: middleware::CorsConfig,
    /// Security headers configuration
    pub security_headers_config: middleware::SecurityHeadersConfig,
    /// Repository context for gov-http integration.
    pub repo_context: RepoContext,
}

// Implement PlatformState for AppState so we can use gov-http handlers
impl gov_http::PlatformState for AppState {
    fn context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn governance_repo(&self) -> Arc<dyn gov_model::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

impl AppState {
    #[allow(dead_code)]
    fn new(governance_repo: Arc<dyn GovernanceRepository>) -> Self {
        let workspace_root = resolve_workspace_root();
        Self::with_config(governance_repo, workspace_root, None)
    }

    pub fn with_config(
        governance_repo: Arc<dyn GovernanceRepository>,
        workspace_root: PathBuf,
        config: Option<spec_runtime::ValidatedConfig>,
    ) -> Self {
        let config = config.or_else(|| load_validated_config(&workspace_root));
        let platform_auth = security::PlatformAuthConfig::from_sources(config.as_ref());
        platform_auth.warn_if_misconfigured();

        // Initialize security configurations
        let cors_config = middleware::CorsConfig::from_sources(config.as_ref());
        let security_headers_config =
            middleware::SecurityHeadersConfig::from_sources(config.as_ref());

        // Create RepoContext for gov-http integration
        let repo_context = RepoContext::new(&workspace_root);

        Self {
            governance_repo,
            workspace_root,
            config,
            platform_auth,
            cors_config,
            security_headers_config,
            repo_context,
        }
    }
}

/// Create the application router (reusable for both main and tests)
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Router {
    let workspace_root = resolve_workspace_root();
    let config = load_validated_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config);
    build_router(app_state)
}

/// Create the application router with an explicit workspace root.
/// Useful for tests to avoid reliance on global environment variables.
pub fn app_with_workspace_root(
    governance_repo: Arc<dyn GovernanceRepository>,
    workspace_root: PathBuf,
) -> Router {
    let config = load_validated_config(&workspace_root);
    build_router(AppState::with_config(governance_repo, workspace_root, config))
}

/// Create an application router from an already-constructed state (e.g., when main has validated config).
pub fn app_with_state(app_state: AppState) -> Router {
    build_router(app_state)
}

fn build_router(app_state: AppState) -> Router {
    let auth_state = app_state.clone();
    let platform_state = app_state.clone();

    let platform_router = Router::new()
        .with_state(platform_state.clone())
        .merge(platform::router(platform_state.clone()))
        .route("/tasks/{id}/status", post(tasks::update_task_status))
        .layer(axum::middleware::from_fn_with_state(auth_state, middleware::platform_auth_guard))
        .with_state(platform_state.clone());

    let tasks_router =
        Router::new().with_state(app_state.clone()).route("/ui/tasks", get(tasks::tasks_ui));

    let agent_router = agent::router(app_state.clone());
    let todos_router = todos::router(app_state.clone());

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform_router)
        // Platform UI routes (at root level)
        .merge(platform::ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        .merge(todos_router)
        // Middleware layers (applied in reverse order - bottom to top)
        // Request ID middleware (outermost - applied first to request)
        .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        // Metrics middleware
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        // CORS middleware
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::cors_middleware))
        // Security headers (innermost - applied first to response)
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::security_headers_middleware,
        ))
        .with_state(app_state)
}

// ============================================================================
// Handlers - showing edge -> core path
// ============================================================================

// ============================================================================
// Template Core Handlers - Keep these in your service
// ============================================================================

/// Health check endpoint
///
/// Demonstrates:
/// - Accessing request ID from extensions
/// - Basic instrumentation
/// - Simple JSON response
#[instrument(skip(_request_id))]
async fn health(Extension(_request_id): Extension<RequestId>) -> impl IntoResponse {
    // Log with request_id automatically included from span
    info!("Health check requested");

    // METRICS STUB: Increment health check counter
    // metrics::counter!("health_checks_total").increment(1);

    Json(HealthResponse { status: "ok".to_string(), service: "service-api".to_string() })
}

/// Version information endpoint
#[instrument]
async fn version() -> impl IntoResponse {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

/// Echo endpoint - Used for testing error handling
///
/// Demonstrates:
/// - Validation errors with error codes
/// - Request ID propagation through error responses
/// - ErrorResponse envelope structure
#[instrument(skip(request_id, payload))]
async fn echo(
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<EchoRequest>,
) -> Result<Json<EchoResponse>, AppError> {
    info!("Echo request received");

    // Validation: message cannot be empty
    if payload.message.is_empty() {
        return Err(AppError::validation_error(ErrorCode::MissingField, "Message cannot be empty")
            .with_context("field", "message")
            .with_ac_id("AC-TPL-003") // Links to error envelope AC
            .with_request_id(request_id.as_str())); // AC-TPL-004: Propagate request ID
    }

    Ok(Json(EchoResponse { message: payload.message }))
}

// ============================================================================
// Your Domain Handlers Go Here
// ============================================================================
//
// Example structure for adding a domain handler:
//
// #[instrument(skip(request_id, payload), fields(entity_id = %payload.id))]
// async fn create_entity(
//     Extension(request_id): Extension<RequestId>,
//     Json(payload): Json<CreateEntityRequest>,
// ) -> Result<(StatusCode, Json<EntityResponse>), AppError> {
//     info!("Processing entity creation");
//
//     // Validation
//     if payload.name.is_empty() {
//         return Err(AppError::validation_error(ErrorCode::MissingField, "Name required")
//             .with_context("field", "name")
//             .with_ac_id("AC-XXX")
//             .with_request_id(request_id.as_str()));
//     }
//
//     // Call core domain logic
//     let entity = core::entities::create(payload)?;
//
//     info!(entity_id = %entity.id, "Entity created");
//     Ok((StatusCode::CREATED, Json(entity.into())))
// }
//
// See docs/tutorials/first-ac-change.md for a complete walkthrough.

// ============================================================================
// DTOs - Request/Response types for HTTP boundary
// ============================================================================

// Template Core DTOs
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Debug, Serialize)]
struct VersionInfo {
    version: String,
    #[serde(rename = "gitSha")]
    git_sha: String,
}

// Echo endpoint DTOs (used for testing error handling)
#[derive(Debug, Deserialize)]
struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    pub message: String,
}

// Your Domain DTOs Go Here
// Example:
// #[derive(Debug, Deserialize)]
// pub struct CreateEntityRequest {
//     pub name: String,
//     #[serde(rename = "someField")]
//     pub some_field: String,
// }

// ============================================================================
// Error handling - See errors.rs for comprehensive error handling
// ============================================================================
//
// The old inline AppError enum has been replaced with a full-featured
// error type in errors.rs that provides:
// - Machine-readable error codes
// - AC ID and Feature ID tracking
// - Structured logging with correlation
// - Proper JSON error responses with context
//
// See errors.rs for implementation details and examples.

// ============================================================================
// Architecture Notes:
//
// This demonstrates hexagonal/clean architecture:
//
// 1. HTTP layer (this file):
//    - Handles HTTP concerns (routing, serialization, status codes)
//    - Translates HTTP requests -> domain operations
//    - Translates domain errors -> HTTP responses
//
// 2. Domain layer (crates/core):
//    - Pure business logic, no HTTP knowledge
//    - Called BY adapters, never calls adapters
//
// 3. Model layer (crates/model):
//    - Domain entities and value objects
//    - Shared across adapters and core
//
// 4. Telemetry (crates/telemetry):
//    - Cross-cutting concern for observability
//    - Initialized once at startup
//
// Key pattern: The dependency arrow points INWARD
//   app-http -> core  ([OK] correct)
//   core -> app-http  ([X] never!)
// ============================================================================

pub fn resolve_workspace_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            tracing::warn!(
                "Failed to resolve workspace root from CARGO_MANIFEST_DIR, using current directory"
            );
            PathBuf::from(".")
        })
}

fn load_validated_config(workspace_root: &Path) -> Option<spec_runtime::ValidatedConfig> {
    let config_path = workspace_root.join("config/local.yaml");
    let schema_path = workspace_root.join("specs/config_schema.yaml");

    match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(cfg) => Some(cfg),
        Err(err) => {
            tracing::warn!(
                "Failed to validate config at {} against {}: {}",
                config_path.display(),
                schema_path.display(),
                err
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adapters_spec_fs::FsGovernanceRepository;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
    }

    /// @AC-TPL-001: Health endpoint returns 200 with status 'ok'
    #[tokio::test]
    async fn test_health_returns_ok() {
        let workspace_root = test_workspace_root();
        let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
        let app = app_with_workspace_root(repo, workspace_root);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"].as_str().unwrap(), "ok");
        assert!(json.get("service").is_some(), "Response should have 'service' field");
    }

    /// @AC-TPL-002: Version endpoint returns build information
    #[tokio::test]
    async fn test_version_returns_build_info() {
        let workspace_root = test_workspace_root();
        let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
        let app = app_with_workspace_root(repo, workspace_root);

        let response = app
            .oneshot(Request::builder().uri("/version").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("version").is_some(), "Response should have 'version' field");
        assert!(json.get("gitSha").is_some(), "Response should have 'gitSha' field");
    }
}
