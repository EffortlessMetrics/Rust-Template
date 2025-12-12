//! Platform endpoint handlers.

use axum::{Json, response::IntoResponse};
use serde::Serialize;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Platform health endpoint.
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok".to_string() })
}

/// Platform status response (simplified).
#[derive(Serialize)]
pub struct StatusResponse {
    pub governance: GovernanceStatusResponse,
}

#[derive(Serialize)]
pub struct GovernanceStatusResponse {
    pub healthy: bool,
}

/// Platform status endpoint.
pub async fn get_status() -> impl IntoResponse {
    Json(StatusResponse { governance: GovernanceStatusResponse { healthy: true } })
}
