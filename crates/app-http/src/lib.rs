use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

/// Create the application router (reusable for both main and tests)
pub fn app() -> Router {
    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        // Example domain endpoints - adapt or replace
        .route("/refunds", post(create_refund))
        .layer(TraceLayer::new_for_http())
}

// ============================================================================
// Handlers - showing edge → core path
// ============================================================================

// ============================================================================
// Template Core Handlers - Keep these in your service
// ============================================================================

/// Health check endpoint
#[instrument]
async fn health() -> impl IntoResponse {
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

// ============================================================================
// Example Domain Handlers - Adapt or replace with your domain
// ============================================================================

/// Create refund endpoint - demonstrates edge → core path
#[instrument(skip(payload))]
async fn create_refund(
    Json(payload): Json<CreateRefundRequest>,
) -> Result<(StatusCode, Json<CreateRefundResponse>), AppError> {
    info!(
        order_id = %payload.order_id,
        amount_cents = payload.amount_cents,
        "Creating refund"
    );

    // Validate request
    if payload.amount_cents == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".to_string()));
    }

    // Call core domain logic
    // This shows the separation: HTTP adapters call domain, not vice versa
    if !core::refund_ok() {
        return Err(AppError::InternalError("Refund processing unavailable".to_string()));
    }

    // In a real system, this would:
    // 1. Validate order exists
    // 2. Check refund eligibility
    // 3. Create refund entity via core::refunds::create()
    // 4. Publish refund.created event
    // For now, simulate success
    let refund = model::Refund { id: format!("REF-{}", uuid::Uuid::new_v4()) };

    info!(refund_id = %refund.id, "Refund created");

    Ok((
        StatusCode::CREATED,
        Json(CreateRefundResponse {
            refund_id: refund.id,
            order_id: payload.order_id,
            amount_cents: payload.amount_cents,
            status: "pending".to_string(),
        }),
    ))
}

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

// Example Domain DTOs
#[derive(Debug, Deserialize)]
pub struct CreateRefundRequest {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "amountCents")]
    pub amount_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct CreateRefundResponse {
    #[serde(rename = "refundId")]
    pub refund_id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "amountCents")]
    pub amount_cents: u64,
    pub status: String,
}

// ============================================================================
// Error handling - converting domain errors to HTTP responses
// ============================================================================

#[derive(Debug)]
enum AppError {
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

// ============================================================================
// Architecture Notes:
//
// This demonstrates hexagonal/clean architecture:
//
// 1. HTTP layer (this file):
//    - Handles HTTP concerns (routing, serialization, status codes)
//    - Translates HTTP requests → domain operations
//    - Translates domain errors → HTTP responses
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
//   app-http → core  (✓ correct)
//   core → app-http  (✗ never!)
// ============================================================================
