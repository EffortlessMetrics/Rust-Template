use crate::AppState;
use axum::{Router, routing::get};

mod config;
mod debug;
mod idp;
mod status;
mod ui;

pub(crate) use config::config_summary;

// Re-export gov-http types for backwards compatibility with downstream consumers
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

/// Platform API routes (mounted at /platform)
///
/// Uses gov-http's `platform_routes_no_status` for governance-generic endpoints,
/// then adds service-specific endpoints like rich `/status` and debug info.
pub fn router(state: AppState) -> Router<AppState> {
    // Start with gov-http routes (includes friction, questions, forks)
    gov_http::platform_routes_no_status::<AppState>()
        // Service-specific endpoints
        .route("/debug/info", get(debug::debug_info))
        .route("/status", get(status::get_status))
        // Service-specific sub-routers
        .merge(idp::router())
        .with_state(state)
}

/// UI routes (mounted at root)
pub fn ui_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(ui::dashboard))
        .route("/ui", get(ui::dashboard))
        .route("/ui/graph", get(ui::graph_view))
        .route("/ui/flows", get(ui::flows_view))
        .route("/ui/coverage", get(ui::coverage_view))
        .with_state(state)
}
