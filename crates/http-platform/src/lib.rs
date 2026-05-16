//! HTTP handlers for `/platform/*` endpoints.
//!
//! This crate implements the platform API including:
//! - Platform status endpoint
//! - IDP snapshot endpoint
//! - UI routes (dashboard, graph, flows, coverage)
//! - Debug info endpoint
//!
//! # Design Philosophy
//!
//! - **Platform-focused**: Only platform-related handlers
//! - **Contract-based**: Uses `platform-contract` types for stable API
//! - **Gov-http integration**: Reuses gov-http handlers for governance endpoints
//!
//! # Example
//!
//! ```rust,ignore
//! use http_platform::router;
//!
//! let app = Router::new().nest("/platform", router(state));
//! ```

use axum::{Router, routing::get};
use spec_runtime::ValidatedConfig;

mod config;
mod debug;
mod idp;
mod status;
mod status_counts;
mod ui;

pub(crate) use config::config_summary;

// Re-export gov-http types for backwards compatibility
pub use gov_http::{
    // Coverage types
    CoverageDetail,
    CoverageResponse,
    CoverageSummary,
    // Docs types
    DocHealthSummary,
    DocInfoWithHealth,
    DocsIndexResponse,
    // Forks types
    ForkEntry,
    ForkSummary,
    ForksListResponse,
    // Friction types
    FrictionContext,
    FrictionEntry,
    FrictionListResponse,
    // Question types
    Question,
    QuestionContext,
    QuestionFilters,
    QuestionSummary,
    QuestionsListResponse,
    // Query types
    SuggestNextQuery,
    // Task types
    TaskDocsOut,
    TaskFilters,
    TaskGraphQuery,
    TaskGraphResponse,
    TaskOut,
    TasksResponse,
};

// Re-export endpoint response types
pub use debug::DebugInfo;
pub use idp::IdpSnapshot;

// ============================================================================
// State Trait
// ============================================================================

/// Platform state trait for handlers.
///
/// This trait defines the minimal interface required for platform handlers.
pub trait PlatformState: Clone + Send + Sync + 'static {
    /// Get workspace root path.
    fn workspace_root(&self) -> &std::path::Path;

    /// Get validated config (if available).
    fn config(&self) -> Option<&ValidatedConfig>;

    /// Get platform auth config.
    fn platform_auth(&self) -> &dyn PlatformAuthConfig;
}

/// Platform auth config trait.
pub trait PlatformAuthConfig {
    /// Get auth mode label.
    fn mode_label(&self) -> &str;

    /// Check if token is present.
    fn token_present(&self) -> bool;
}

impl PlatformAuthConfig for http_auth::PlatformAuthConfig {
    fn mode_label(&self) -> &str {
        http_auth::PlatformAuthConfig::mode_label(self)
    }

    fn token_present(&self) -> bool {
        http_auth::PlatformAuthConfig::token_present(self)
    }
}

// ============================================================================
// Platform Router
// ============================================================================

/// Create the platform API router (mounted at /platform).
///
/// Uses gov-http's `platform_routes_no_status` for governance-generic endpoints,
/// then adds service-specific endpoints like rich `/status` and debug info.
pub fn router<S>(state: S) -> Router<S>
where
    S: PlatformState + Clone + 'static + gov_http::PlatformState,
{
    gov_http::platform_routes_no_status::<S>()
        .merge(idp::router::<S>())
        // Service-specific endpoints
        .route("/debug/info", get(debug::debug_info::<S>))
        .route("/status", get(status::get_status::<S>))
        .with_state(state)
}

/// Create the UI routes (mounted at root).
///
/// Returns routes for:
/// - `/` - Dashboard
/// - `/ui` - Dashboard
/// - `/ui/graph` - Graph view
/// - `/ui/flows` - Flows view
/// - `/ui/coverage` - Coverage view
pub fn ui_router<S>(state: S) -> Router<S>
where
    S: PlatformState + Clone + 'static,
{
    Router::new()
        .route("/", get(ui::dashboard::<S>))
        .route("/ui", get(ui::dashboard::<S>))
        .route("/ui/graph", get(ui::graph_view::<S>))
        .route("/ui/flows", get(ui::flows_view::<S>))
        .route("/ui/coverage", get(ui::coverage_view::<S>))
        .with_state(state)
}
