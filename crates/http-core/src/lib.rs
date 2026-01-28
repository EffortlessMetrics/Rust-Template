//! HTTP core foundation for the Rust-as-Spec platform.
//!
//! This crate provides shared HTTP infrastructure including:
//! - App state traits for dependency injection
//! - Router composition helpers
//! - Shared handlers (health, version, metrics)
//! - Shutdown signal handling
//!
//! # Design Philosophy
//!
//! - **Minimal dependencies**: Only essential HTTP and platform deps
//! - **Shared foundation**: Common types used across all http-* crates
//! - **Trait-based state**: Use traits for state requirements, not concrete types
//!
//! # Example
//!
//! ```rust,ignore
//! use http_core::{AppState, router};
//!
//! let state = AppState::new(repo)?;
//! let app = router(state);
//! ```

use axum::{Json, Router, extract::Extension, response::IntoResponse, routing::get};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, instrument};

// Request ID type for request tracking
pub type RequestId = String;

// ============================================================================
// App State Traits
// ============================================================================

/// Platform state trait for governance integration.
///
/// This trait allows http-* crates to work with any state that implements
/// the required governance operations.
pub trait PlatformState {
    /// Get the repository context.
    fn context(&self) -> &gov_model::RepoContext;

    /// Get the governance repository.
    fn governance_repo(&self) -> Arc<dyn gov_model::GovernanceRepository>;
}

/// Core application state trait.
///
/// This trait defines the minimal interface required for HTTP handlers.
pub trait AppState: Clone + Send + Sync + 'static {
    /// Get the workspace root path.
    fn workspace_root(&self) -> &Path;

    /// Get the governance repository.
    fn governance_repo(&self) -> Arc<dyn business_core::governance::GovernanceRepository>;

    /// Get the repository context.
    fn repo_context(&self) -> &gov_model::RepoContext;

    /// Get the validated config (if available).
    fn config(&self) -> Option<&spec_runtime::ValidatedConfig>;
}

// ============================================================================
// Shared Handlers
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Status indicator
    pub status: String,
    /// Service name
    pub service: String,
}

/// Version information response.
#[derive(Debug, Serialize)]
pub struct VersionInfo {
    /// Version from Cargo.toml
    pub version: String,
    /// Git SHA from build environment
    #[serde(rename = "gitSha")]
    pub git_sha: String,
}

/// Health check endpoint.
///
/// Demonstrates:
/// - Accessing request ID from extensions
/// - Basic instrumentation
/// - Simple JSON response
#[instrument(skip(_request_id))]
pub async fn health(Extension(_request_id): Extension<RequestId>) -> impl IntoResponse {
    info!("Health check requested");

    Json(HealthResponse { status: "ok".to_string(), service: "service-api".to_string() })
}

/// Version information endpoint.
#[instrument]
pub async fn version() -> impl IntoResponse {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

// ============================================================================
// Router Composition Helpers
// ============================================================================

/// Create a base router with common endpoints.
///
/// This function creates a router with:
/// - `/health` - Health check endpoint
/// - `/version` - Version information endpoint
pub fn base_router() -> Router {
    Router::new().route("/health", get(health)).route("/version", get(version))
}

// ============================================================================
// Workspace Root Resolution
// ============================================================================

/// Resolve the workspace root from environment or CARGO_MANIFEST_DIR.
///
/// Checks SPEC_ROOT environment variable first, then resolves from
/// CARGO_MANIFEST_DIR (crates/app-http).
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

// ============================================================================
// Shutdown Signal Handling
// ============================================================================

/// Create a future that completes when a shutdown signal is received.
///
/// On Unix: Handles both SIGTERM and SIGINT (Ctrl-C)
/// On Windows: Handles Ctrl-C only
///
/// When signal is received, logs an informational message and returns,
/// allowing the server to begin graceful shutdown.
///
/// # Example
///
/// ```ignore
/// axum::serve(listener, app)
///     .with_graceful_shutdown(shutdown_signal())
///     .await?;
/// ```
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::warn!("Failed to install Ctrl+C handler: {}", err);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                tracing::warn!("Failed to install SIGTERM handler: {}", err);
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl-C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}

// ============================================================================
// Re-exports
// ============================================================================

// Re-export http-errors types for convenience
pub use http_errors::{ErrorCode, ErrorResponse, HttpError};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_workspace_root_from_env() {
        let guard = testing::process::EnvVarGuard::new(&["SPEC_ROOT"]);
        guard.set("SPEC_ROOT", "/test/path");

        let root = resolve_workspace_root();
        assert_eq!(root, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_health_response_serialization() {
        let response =
            HealthResponse { status: "ok".to_string(), service: "test-service".to_string() };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ok"));
        assert!(json.contains("test-service"));
    }

    #[test]
    fn test_version_info_serialization() {
        let info = VersionInfo { version: "1.0.0".to_string(), git_sha: "abc123".to_string() };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("1.0.0"));
        assert!(json.contains("abc123"));
        assert!(json.contains("gitSha"));
    }
}
