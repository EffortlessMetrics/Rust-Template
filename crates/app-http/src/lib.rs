use axum::{
    Router,
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

// Public modules
pub mod errors;
pub mod middleware;

// Re-export commonly used types
pub use errors::{AppError, ErrorCode};
pub use middleware::{REQUEST_ID_HEADER, RequestId};

/// Create the application router (reusable for both main and tests)
pub fn app() -> Router {
    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        // Example domain endpoints - adapt or replace
        .route("/refunds", post(create_refund))
        // Middleware layers (applied in reverse order - bottom to top)
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
// Handlers - showing edge → core path
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

// ============================================================================
// Example Domain Handlers - Adapt or replace with your domain
// ============================================================================

/// Create refund endpoint - demonstrates edge → core path with full observability
///
/// This handler demonstrates:
/// - Request ID correlation via Extension
/// - Structured logging with business context
/// - Enhanced error handling with error codes and context
/// - AC/Feature ID tracking for product correlation
/// - Instrumentation with custom fields
/// - Proper validation and error responses
/// - Metrics integration points (stubbed)
#[instrument(
    skip(request_id, payload),
    fields(
        order_id = %payload.order_id,
        amount_cents = payload.amount_cents,
        // request_id is automatically included from the span
    )
)]
async fn create_refund(
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateRefundRequest>,
) -> Result<(StatusCode, Json<CreateRefundResponse>), AppError> {
    info!("Processing refund creation request");

    // METRICS STUB: Start timer for refund creation latency
    // let _timer = metrics::histogram!("refund_creation_duration_seconds").start_timer();

    // Validation: Amount must be positive
    if payload.amount_cents == 0 {
        // METRICS STUB: Increment validation error counter
        // metrics::counter!("refund_validation_errors_total", "field" => "amount").increment(1);

        return Err(
            AppError::validation_error(ErrorCode::InvalidAmount, "Amount must be greater than 0")
                .with_context("field", "amount_cents")
                .with_context("value", payload.amount_cents)
                .with_ac_id("AC-REFUND-001") // Links to acceptance criteria
                .with_feature_id("FT-REFUND-CREATION") // Links to feature
                .with_request_id(request_id.as_str()), // AC-TPL-004: Propagate request ID
        );
    }

    // Validation: Order ID format (simple example)
    if payload.order_id.is_empty() {
        return Err(AppError::validation_error(ErrorCode::MissingField, "Order ID is required")
            .with_context("field", "order_id")
            .with_ac_id("AC-REFUND-001")
            .with_request_id(request_id.as_str())); // AC-TPL-004
    }

    // Call core domain logic
    // This shows the separation: HTTP adapters call domain, not vice versa
    if !core::refund_ok() {
        // Log internal error with context
        tracing::error!(
            order_id = %payload.order_id,
            "Refund service unavailable"
        );

        // METRICS STUB: Increment service unavailable counter
        // metrics::counter!("refund_service_errors_total", "type" => "unavailable").increment(1);

        return Err(AppError::internal_error("Refund processing unavailable")
            .with_context("order_id", &payload.order_id)
            .with_context("service", "core::refunds")
            .with_request_id(request_id.as_str())); // AC-TPL-004
    }

    // In a real system, this would:
    // 1. Validate order exists via core::orders::get(&payload.order_id)
    // 2. Check refund eligibility via core::refunds::is_eligible()
    // 3. Create refund entity via core::refunds::create()
    // 4. Publish refund.created event via event bus
    //
    // For now, simulate success
    let refund = model::Refund { id: format!("REF-{}", uuid::Uuid::new_v4()) };

    // Log successful creation with structured data
    info!(
        refund_id = %refund.id,
        order_id = %payload.order_id,
        amount_cents = payload.amount_cents,
        "Refund created successfully"
    );

    // METRICS STUB: Increment successful refund counter
    // metrics::counter!("refunds_created_total", "status" => "success").increment(1);
    // metrics::histogram!("refund_amount_cents").record(payload.amount_cents as f64);

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
