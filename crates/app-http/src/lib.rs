//! HTTP application layer for the Rust-as-Spec platform.
//!
//! This crate implements the HTTP interface for the platform, serving as the primary
//! adapter between external HTTP clients and the core business logic. It provides a
//! complete web application with routing, middleware, security, and observability.
//!
//! # Architecture
//!
//! The crate follows hexagonal/clean architecture principles:
//!
//! - **HTTP Layer (this crate)**: Handles HTTP concerns (routing, serialization, status codes)
//!   and translates between HTTP requests/responses and domain operations.
//! - **Domain Layer** (`business-core`): Pure business logic with no HTTP knowledge.
//! - **Model Layer** (`gov-model`): Domain entities and value objects shared across layers.
//! - **Telemetry** (`telemetry`): Cross-cutting observability concerns.
//!
//! Dependencies point inward: `app-http` → `business-core` (correct), never the reverse.
//!
//! # Main Components
//!
//! ## Router and Handlers
//!
//! The application provides several categories of endpoints:
//!
//! - **Core Template Endpoints**: `/health`, `/version`, `/metrics` - operational endpoints
//! - **Platform Introspection**: `/platform/*` - governance graph, specs, tasks, docs
//! - **Platform UI**: `/ui/*` - web interfaces for tasks and governance visualization
//! - **Domain Endpoints**: `/api/todos`, `/api/agent/*` - business logic endpoints
//!
//! ## Middleware Stack
//!
//! Applied in reverse order (bottom-to-top in code):
//!
//! 1. **Request ID** (`middleware::request_id_middleware`): Generates or propagates correlation IDs
//! 2. **Metrics** (`metrics::metrics_middleware`): Tracks request latency and counts
//! 3. **CORS** (`middleware::cors_middleware`): Configurable cross-origin resource sharing
//! 4. **Security Headers** (`middleware::security_headers_middleware`): CSP, X-Frame-Options, HSTS
//!
//! ## Security Features
//!
//! - **Platform Authentication**: JWT-based auth for `/platform/*` endpoints via `platform_auth_guard`
//! - **Security Headers**: Comprehensive security header configuration (CSP, HSTS, X-Content-Type-Options)
//! - **CORS**: Configurable origin validation and credentials handling
//! - **Request ID Propagation**: Correlation IDs for request tracing and audit trails
//!
//! ## Error Handling
//!
//! The `errors` module provides a comprehensive error handling system:
//!
//! - **Machine-readable error codes** (`ErrorCode` enum)
//! - **AC/Feature ID tracking** for governance alignment
//! - **Structured logging** with request correlation
//! - **JSON error envelopes** with consistent format and context
//!
//! See [`errors::AppError`] for details.
//!
//! # Integration with Other Crates
//!
//! ## `business-core`
//!
//! Provides domain logic accessed by handlers. The HTTP layer calls into core services
//! and translates domain errors into HTTP responses.
//!
//! ## `gov-http`
//!
//! Provides `/platform/*` endpoints for governance introspection. This crate implements
//! the `gov_http::PlatformState` trait on `AppState` to enable integration.
//!
//! ## `gov-model`
//!
//! Provides governance data structures (`RepoContext`, `GovernanceRepository`) used
//! throughout the platform endpoints.
//!
//! ## `spec-runtime`
//!
//! Validates configuration files (`config/local.yaml` against `specs/config_schema.yaml`)
//! at startup, providing type-safe config access.
//!
//! ## `telemetry`
//!
//! Initialized at application startup to provide structured logging, tracing, and metrics.
//! Handlers use `#[instrument]` for automatic span creation and request correlation.
//!
//! # Usage
//!
//! ## Creating an Application
//!
//! ```rust,no_run
//! use app_http::app;
//! use adapters_spec_fs::FsGovernanceRepository;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let repo = Arc::new(FsGovernanceRepository::new("/workspace/root".into()));
//! let router = app(repo).expect("invalid platform auth configuration");
//!
//! // Serve with axum
//! let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
//! axum::serve(listener, router).await.unwrap();
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust,no_run
//! use app_http::{AppState, app_with_state};
//! use adapters_spec_fs::FsGovernanceRepository;
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # fn example() {
//! let workspace_root = PathBuf::from("/workspace/root");
//! let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
//! let state = AppState::with_config(repo, workspace_root, None)
//!     .expect("invalid platform auth configuration");
//! let router = app_with_state(state);
//! # }
//! ```
//!
//! # Key Features
//!
//! - **Governance-first**: All endpoints integrate with the platform's governance system
//! - **Observability**: Request IDs, structured logs, metrics, distributed tracing
//! - **Security**: JWT auth, security headers, CORS, defense-in-depth
//! - **Type-safe Config**: Schema-validated YAML configuration via `spec-runtime`
//! - **AC Traceability**: Error responses link to Acceptance Criteria IDs
//! - **Developer Experience**: Clear error messages, comprehensive docs, testing utilities

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
    /// Create a new AppState with default configuration.
    ///
    /// Uses the default workspace root resolution. For tests or custom configurations,
    /// prefer `with_config()` which allows explicit workspace root specification.
    pub fn new(governance_repo: Arc<dyn GovernanceRepository>) -> Result<Self, String> {
        let workspace_root = resolve_workspace_root();
        Self::with_config(governance_repo, workspace_root, None)
    }

    pub fn with_config(
        governance_repo: Arc<dyn GovernanceRepository>,
        workspace_root: PathBuf,
        config: Option<spec_runtime::ValidatedConfig>,
    ) -> Result<Self, String> {
        let config = config.or_else(|| load_validated_config(&workspace_root));
        let platform_auth = security::PlatformAuthConfig::try_from_sources(config.as_ref())?;
        platform_auth.warn_if_misconfigured();

        // Initialize security configurations
        let cors_config = middleware::CorsConfig::from_sources(config.as_ref());
        let security_headers_config =
            middleware::SecurityHeadersConfig::from_sources(config.as_ref());

        // Create RepoContext for gov-http integration
        let repo_context = RepoContext::new(&workspace_root);

        Ok(Self {
            governance_repo,
            workspace_root,
            config,
            platform_auth,
            cors_config,
            security_headers_config,
            repo_context,
        })
    }
}

/// Create the application router (reusable for both main and tests)
///
/// # Errors
///
/// Returns an error if platform auth configuration is invalid (e.g., invalid PLATFORM_AUTH_MODE).
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Result<Router, String> {
    let workspace_root = resolve_workspace_root();
    let config = load_validated_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config)?;
    Ok(build_router(app_state))
}

/// Create the application router with an explicit workspace root.
/// Useful for tests to avoid reliance on global environment variables.
///
/// # Errors
///
/// Returns an error if platform auth configuration is invalid (e.g., invalid PLATFORM_AUTH_MODE).
pub fn app_with_workspace_root(
    governance_repo: Arc<dyn GovernanceRepository>,
    workspace_root: PathBuf,
) -> Result<Router, String> {
    let config = load_validated_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config)?;
    Ok(build_router(app_state))
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
        let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

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
        let app = app_with_workspace_root(repo, workspace_root).expect("valid config");

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
