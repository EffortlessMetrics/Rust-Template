//! Platform HTTP router for governance endpoints.
//!
//! This crate provides a reusable Axum router that mounts all `/platform/*`
//! endpoints for governance introspection. Services compose this router
//! into their HTTP layer via `.nest()`.
//!
//! # Example
//!
//! ```ignore
//! use gov_http::{platform_router, PlatformState};
//! use axum::Router;
//!
//! let state = MyPlatformState::new();
//! let app = Router::new()
//!     .nest("/platform", platform_router(state));
//! ```

pub mod handlers;
pub mod state;

pub use state::PlatformState;

use axum::{Router, routing::get};

/// Build the platform router with all governance endpoints.
///
/// Mount this at `/platform` in your service's router.
pub fn platform_router<S>(_state: S) -> Router
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/status", get(handlers::get_status))
        .route("/health", get(handlers::health))
}
