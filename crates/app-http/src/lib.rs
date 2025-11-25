use axum::{
    Router,
    extract::{Extension, Json},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

// Public modules
pub mod agent;
pub mod errors;
pub mod metrics;
pub mod middleware;
pub mod platform;
pub mod tasks;

// Re-export commonly used types
pub use errors::{AppError, ErrorCode};
pub use middleware::{REQUEST_ID_HEADER, RequestId};

use business_core::governance::GovernanceRepository;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub governance_repo: Arc<dyn GovernanceRepository>,
    pub workspace_root: PathBuf,
}

impl AppState {
    fn new(governance_repo: Arc<dyn GovernanceRepository>) -> Self {
        Self { governance_repo, workspace_root: resolve_workspace_root() }
    }
}

/// Create the application router (reusable for both main and tests)
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Router {
    let app_state = AppState::new(governance_repo);
    build_router(app_state)
}

/// Create the application router with an explicit workspace root.
/// Useful for tests to avoid reliance on global environment variables.
pub fn app_with_workspace_root(
    governance_repo: Arc<dyn GovernanceRepository>,
    workspace_root: PathBuf,
) -> Router {
    build_router(AppState { governance_repo, workspace_root })
}

fn build_router(app_state: AppState) -> Router {
    let tasks_router = Router::new()
        .route("/platform/tasks/{id}/status", post(tasks::update_task_status))
        .route("/ui/tasks", get(tasks::tasks_ui))
        .with_state(app_state.clone());

    let agent_router = agent::router(app_state.clone());
    let platform_state = app_state.clone();

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform::router(platform_state.clone()))
        // Platform UI routes (at root level)
        .merge(platform::ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        // Middleware layers (applied in reverse order - bottom to top)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        .layer(
            // Configure TraceLayer to include request_id field
            TraceLayer::new_for_http().make_span_with(|request: &axum::extract::Request| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = tracing::field::Empty, // Will be filled by request_id middleware
                )
            }),
        )
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

fn resolve_workspace_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}
