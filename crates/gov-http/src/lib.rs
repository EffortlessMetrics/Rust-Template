//! Platform HTTP router for governance endpoints.
//!
//! This crate provides a reusable Axum router that mounts all `/platform/*`
//! endpoints for governance introspection. Services compose this router
//! into their HTTP layer via `.nest()`.
//!
//! # Example
//!
//! ```ignore
//! use gov_http::{platform_router, DefaultPlatformState};
//! use gov_model::RepoContext;
//! use axum::Router;
//! use std::sync::Arc;
//!
//! let ctx = RepoContext::new("/workspace");
//! let repo = my_governance_repo();
//! let state = Arc::new(DefaultPlatformState::new(ctx, repo));
//!
//! let app = Router::new()
//!     .nest("/platform", platform_router(state));
//! ```

pub mod error;
pub mod handlers;
pub mod state;

pub use error::PlatformError;
pub use handlers::{DocHealthSummary, DocInfoWithHealth, DocsIndexResponse};
pub use state::{DefaultPlatformState, PlatformState};

use axum::{Router, routing::get};
use std::sync::Arc;

/// Build the platform router with all governance endpoints.
///
/// Mount this at `/platform` in your service's router.
///
/// # Contract Anchor Endpoints
///
/// These are the externally consumed surfaces that define the "governed service cell":
/// - `/schema` - All platform schemas
/// - `/schema/{name}` - Specific schema by name
/// - `/docs/index` - Documentation inventory with health validation
/// - `/ui/contract` - UI contract specification
///
/// # Core Endpoints
///
/// - `/health` - Health check
/// - `/status` - Governance status (simplified)
pub fn platform_router<S>(state: Arc<S>) -> Router
where
    S: PlatformState + 'static,
{
    Router::new()
        // Core endpoints
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
        // Contract anchor endpoints
        .route("/schema", get(handlers::get_schema))
        .route("/schema/{name}", get(handlers::get_schema_by_name))
        .route("/docs/index", get(handlers::get_docs_index::<S>))
        .route("/ui/contract", get(handlers::get_ui_contract::<S>))
        .with_state(state)
}

/// Build a minimal platform router with only health/status endpoints.
///
/// Use this when you only need basic health checks without full governance.
pub fn minimal_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
}
