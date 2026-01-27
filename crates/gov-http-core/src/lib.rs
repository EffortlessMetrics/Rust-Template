//! Shared foundation for gov-http subrouter crates.
//!
//! This crate provides common router glue, extractors, error mapping,
//! and shared types used by all gov-http-* subrouter crates.
//!
//! # Purpose
//!
//! - Shared error types and error mapping traits
//! - Common extractors (request ID, auth context)
//! - Router composition helpers
//! - Shared middleware integration
//! - Common response builders
//!
//! # Design Principles
//!
//! - Minimal dependencies - only essential HTTP and model types
//! - No domain-specific handlers - those belong in subrouter crates
//! - Trait-based state requirements for flexibility
//! - Re-export compatible types from http-errors

pub mod error;
pub mod extractors;
pub mod state;

pub use error::{ErrorResponse, PlatformError};
pub use extractors::RequestId;
pub use state::PlatformState;

use axum::Router;

/// Build a router with health check endpoints.
///
/// This provides a minimal router with `/health` and `/status` endpoints
/// that can be composed with other routers.
pub fn health_router() -> Router {
    Router::new()
        .route("/health", axum::routing::get(handlers::health))
        .route("/status", axum::routing::get(handlers::get_status))
}

mod handlers {
    use axum::Json;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct HealthResponse {
        pub status: String,
    }

    #[derive(Serialize)]
    pub struct StatusResponse {
        pub governance: GovernanceStatusResponse,
    }

    #[derive(Serialize)]
    pub struct GovernanceStatusResponse {
        pub healthy: bool,
    }

    pub async fn health() -> axum::response::Json<HealthResponse> {
        Json(HealthResponse { status: "ok".to_string() })
    }

    pub async fn get_status() -> axum::response::Json<StatusResponse> {
        Json(StatusResponse { governance: GovernanceStatusResponse { healthy: true } })
    }
}
