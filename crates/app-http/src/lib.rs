//! HTTP application layer for the Rust-as-Spec platform.
//!
//! This crate acts as a facade for the HTTP layer, re-exporting functionality
//! from the focused http-* crates. It provides backward compatibility for existing
//! imports while delegating to the specialized crates.
//!
//! # Architecture
//!
//! The crate follows the facade pattern for dependency isolation:
//!
//! - **Facade Layer (this crate)**: Provides backward-compatible API by re-exporting
//!   from http-* crates.
//! - **Specialized HTTP Crates**: Each http-* crate handles a specific domain:
//!   - `http-core`: Shared foundation (app state traits, common handlers)
//!   - `http-platform`: Platform endpoints (`/platform/*`, UI routes)
//!   - `http-tasks`: Task management endpoints
//!   - `http-todos`: Todo management endpoints
//!   - `http-agents`: Agent hints endpoints
//!   - `http-metrics`: Prometheus metrics middleware and endpoint
//!   - `http-middleware`: Cross-cutting middleware
//!
//! Dependencies point inward: http-* → http-core → platform-contract (correct).
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
//! - **Platform Authentication**: JWT-based auth for `/platform/*` endpoints
//! - **Security Headers**: Comprehensive security header configuration
//! - **CORS**: Configurable origin validation and credentials handling
//! - **Request ID Propagation**: Correlation IDs for request tracing and audit trails
//!
//! ## Error Handling
//!
//! Error types are re-exported from `http-errors` crate:
//!
//! - **Machine-readable error codes** (`ErrorCode` enum)
//! - **AC/Feature ID tracking** for governance alignment
//! - **Structured logging** with request correlation
//! - **JSON error envelopes** with consistent format and context
//!
//! # Integration with Other Crates
//!
//! ## `http-core`
//!
//! Provides shared HTTP foundation including app state traits, common handlers,
//! and shutdown signal handling.
//!
//! ## `http-platform`
//!
//! Provides `/platform/*` endpoints for governance introspection and UI routes.
//!
//! ## `http-tasks`
//!
//! Provides task management endpoints.
//!
//! ## `http-todos`
//!
//! Provides todo management endpoints.
//!
//! ## `http-agents`
//!
//! Provides agent hints endpoints.
//!
//! ## `http-metrics`
//!
//! Provides request metrics middleware and `/metrics` handler.
//!
//! ## `http-middleware`
//!
//! Provides cross-cutting middleware (CORS, security headers, request ID).
//!
//! ## `business-core`
//!
//! Provides domain logic accessed by handlers.
//!
//! ## `spec-runtime`
//!
//! Validates configuration files at startup.
//!
//! ## `telemetry`
//!
//! Initialized at application startup for structured logging and metrics.
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
//! # Key Features
//!
//! - **Dependency Isolation**: Each http-* crate has minimal, focused dependencies
//! - **Backward Compatibility**: Existing imports continue to work via re-exports
//! - **Governance-first**: All endpoints integrate with the platform's governance system
//! - **Observability**: Request IDs, structured logs, metrics, distributed tracing
//! - **Security**: JWT auth, security headers, CORS, defense-in-depth
//! - **Type-safe Config**: Schema-validated YAML configuration
//! - **AC Traceability**: Error responses link to Acceptance Criteria IDs

use axum::{
    Router,
    extract::{Extension, Json},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

// ============================================================================
// Re-exports from http-* crates
// ============================================================================

// Re-export from http-core
pub use http_core::{
    AppState as CoreAppState, base_router, resolve_workspace_root, shutdown_signal,
};

// Re-export from http-platform
pub use http_platform::{
    // Re-exported gov-http types
    CoverageDetail,
    CoverageResponse,
    CoverageSummary,
    DebugInfo,
    DocHealthSummary,
    DocInfoWithHealth,
    DocsIndexResponse,
    ForkEntry,
    ForkSummary,
    ForksListResponse,
    FrictionContext,
    FrictionEntry,
    FrictionListResponse,
    // Re-exported IDP types
    IdpSnapshot,
    PlatformState as HttpPlatformState,
    Question,
    QuestionContext,
    QuestionFilters,
    QuestionSummary,
    QuestionsListResponse,
    SuggestNextQuery,
    TaskDocsOut,
    TaskFilters,
    TaskGraphQuery,
    TaskGraphResponse,
    TaskOut,
    TasksResponse,
    router as platform_router,
    ui_router,
};

// Re-export from http-tasks
pub use http_tasks::{TasksState, router as tasks_router, tasks_ui, update_task_status};

// Re-export from http-todos
pub use http_todos::{CreateTodoRequest, TodosStateTrait, router as todos_router};

// Re-export from http-agents
pub use http_agents::{
    AgentHint, AgentHintReason, AgentHintsResponse, AgentsState, HintsFilters, RecommendedStep,
    router as agents_router,
};

// Re-export from http-metrics
pub use http_metrics::{metrics_handler, metrics_middleware};

// Re-export from app-http internal modules (backward compatibility)
pub use errors::{AppError, ErrorCode, ErrorSummary, get_error_summary};
pub use middleware::{
    CorsConfig, REQUEST_ID_HEADER, RequestId, SecurityHeadersConfig, cors_middleware,
    platform_auth_guard, request_id_middleware, security_headers_middleware,
};

// ============================================================================
// Public modules (kept for backward compatibility)
// ============================================================================

// Note: Most modules have been moved to http-* crates.
// These are kept as thin wrappers for backward compatibility.

// Public modules (kept for backward compatibility)
pub mod errors;
pub mod metrics;
pub mod middleware;
pub mod security;
pub mod shutdown;

// Compatibility modules for legacy paths
pub mod platform {
    pub use http_platform::*;
}

// Re-export commonly used types (backward compatibility)
pub use security::PlatformAuthConfig;

use business_core::governance::GovernanceRepository;
use gov_model::RepoContext;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ============================================================================
// Application State (Facade)
// ============================================================================

/// Application state combining all required state for the HTTP layer.
///
/// This state implements all the required traits from the http-* crates
/// to enable seamless integration.
#[derive(Clone)]
pub struct AppState {
    /// Governance repository
    pub governance_repo: Arc<dyn GovernanceRepository>,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Validated configuration
    pub config: Option<spec_runtime::ValidatedConfig>,
    /// Platform authentication configuration
    pub platform_auth: PlatformAuthConfig,
    /// CORS configuration
    pub cors_config: CorsConfig,
    /// Security headers configuration
    pub security_headers_config: crate::middleware::security_headers::CachedSecurityHeaders,
    /// Repository context for gov-http integration
    pub repo_context: RepoContext,
}

// Implement http-core AppState trait
impl http_core::AppState for AppState {
    fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    fn governance_repo(&self) -> Arc<dyn business_core::governance::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }

    fn repo_context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn config(&self) -> Option<&spec_runtime::ValidatedConfig> {
        self.config.as_ref()
    }
}

// Implement gov-http PlatformState trait (for governance endpoints)
impl gov_http::PlatformState for AppState {
    fn context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn governance_repo(&self) -> Arc<dyn gov_model::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

// Implement http-platform PlatformState trait
impl HttpPlatformState for AppState {
    fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    fn config(&self) -> Option<&spec_runtime::ValidatedConfig> {
        self.config.as_ref()
    }

    fn platform_auth(&self) -> &dyn http_platform::PlatformAuthConfig {
        &self.platform_auth
    }
}

// Implement http-tasks TasksState trait
impl TasksState for AppState {
    fn governance_repo(
        &self,
    ) -> std::sync::Arc<dyn business_core::governance::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

// Implement http-todos TodosStateTrait trait
impl TodosStateTrait for AppState {
    fn todos_state(&self) -> http_todos::TodosState {
        http_todos::TodosState::new()
    }
}

// Implement http-agents AgentsState trait
impl AgentsState for AppState {
    fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    fn governance_repo(
        &self,
    ) -> std::sync::Arc<dyn business_core::governance::GovernanceRepository> {
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
        let config = config.or_else(|| load_valid_config(&workspace_root));
        let platform_auth = security::PlatformAuthConfig::try_from_sources(config.as_ref())?;
        platform_auth.warn_if_misconfigured();

        // Initialize security configurations
        let cors_config = CorsConfig::from_sources(config.as_ref());
        let security_headers_config_raw = SecurityHeadersConfig::from_sources(config.as_ref());
        let security_headers_config = crate::middleware::security_headers::CachedSecurityHeaders::new(&security_headers_config_raw);

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

/// Create the application router (reusable for both main and tests).
///
/// # Errors
///
/// Returns an error if platform auth configuration is invalid (e.g., invalid PLATFORM_AUTH_MODE).
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Result<Router, String> {
    let workspace_root = resolve_workspace_root();
    let config = load_valid_config(&workspace_root);
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
    let config = load_valid_config(&workspace_root);
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
        .merge(platform_router(platform_state.clone()))
        .route("/tasks/{id}/status", post(update_task_status::<AppState>))
        .layer(axum::middleware::from_fn_with_state(auth_state, platform_auth_guard))
        .with_state(platform_state.clone());

    let tasks_router =
        Router::new().with_state(app_state.clone()).route("/ui/tasks", get(tasks_ui::<AppState>));

    let agent_router = agents_router(app_state.clone());
    let todos_router = todos_router(app_state.clone());

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform_router)
        // Platform UI routes (at root level)
        .merge(ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        .merge(todos_router)
        // Middleware layers (applied in reverse order - bottom to top)
        // Request ID middleware (outermost - applied first to request)
        .layer(axum::middleware::from_fn(request_id_middleware))
        // Metrics middleware
        .layer(axum::middleware::from_fn(metrics_middleware))
        // CORS middleware
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), cors_middleware))
        // Security headers (innermost - applied first to response)
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), security_headers_middleware))
        .with_state(app_state)
}

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
// Error handling
// ============================================================================
//
// Error types are now re-exported from http-errors crate.
// The http-errors crate provides:
// - Machine-readable error codes
// - AC ID and Feature ID tracking
// - Structured logging with correlation
// - Proper JSON error responses with context
//
// See http-errors crate for implementation details and examples.

// ============================================================================
// Architecture Notes:
//
// This demonstrates the facade pattern for dependency isolation:
//
// 1. Facade layer (this crate):
//    - Provides backward-compatible API by re-exporting from http-* crates
//    - Delegates to specialized crates for domain-specific functionality
//
// 2. Specialized HTTP crates (http-*):
//    - http-core: Shared foundation (app state traits, common handlers)
//    - http-platform: Platform endpoints (`/platform/*`, UI routes)
//    - http-tasks: Task management endpoints
//    - http-todos: Todo management endpoints
//    - http-agents: Agent hints endpoints
//    - http-middleware: Cross-cutting middleware
//
// 3. Domain layer (business-core):
//    - Pure business logic, no HTTP knowledge
//    - Called BY adapters, never calls adapters
//
// 4. Model layer (gov-model):
//    - Domain entities and value objects
//    - Shared across adapters and core
//
// 5. Telemetry (telemetry):
//    - Cross-cutting concern for observability
//    - Initialized once at startup
//
// Key pattern: The dependency arrow points INWARD
//   http-* crates -> http-core, platform-contract (correct)
//   app-http (facade) -> http-* crates (correct)
// ============================================================================

fn load_valid_config(workspace_root: &Path) -> Option<spec_runtime::ValidatedConfig> {
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
