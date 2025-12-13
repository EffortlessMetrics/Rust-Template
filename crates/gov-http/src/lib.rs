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
pub mod forks;
pub mod friction;
pub mod handlers;
pub mod questions;
pub mod state;

pub use error::PlatformError;
pub use forks::{ForkEntry, ForkSummary, ForksListResponse, Maintainer};
pub use friction::{FrictionContext, FrictionEntry, FrictionListResponse, Resolution};
pub use handlers::{
    CoverageDetail, CoverageResponse, CoverageSummary, DocHealthSummary, DocInfoWithHealth,
    DocsIndexResponse, SuggestNextQuery, TaskDocsOut, TaskFilters, TaskGraphQuery,
    TaskGraphResponse, TaskOut, TasksResponse,
};
pub use questions::{
    Question, QuestionContext, QuestionFilters, QuestionOption, QuestionResolution,
    QuestionSummary, QuestionsListResponse, Recommendation,
};
pub use state::{DefaultPlatformState, PlatformState};

use axum::{Router, routing::get};

/// Build the platform router with all governance endpoints (with state provided).
///
/// Mount this at `/platform` in your service's router.
/// This version provides state directly and returns a stateless router.
///
/// # Contract Anchor Endpoints
///
/// These are the externally consumed surfaces that define the "governed service cell":
/// - `/schema` - All platform schemas
/// - `/schema/{name}` - Specific schema by name
/// - `/docs/index` - Documentation inventory with health validation
/// - `/ui/contract` - UI contract specification
///
/// # Governance Introspection Endpoints
///
/// - `/graph` - Full governance graph (stories → REQs → ACs → tests → docs)
/// - `/devex/flows` - Developer experience flows and commands
/// - `/coverage` - AC coverage from BDD test results
///
/// # Core Endpoints
///
/// - `/health` - Health check
/// - `/status` - Governance status (simplified)
pub fn platform_router<S>(state: S) -> Router
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    platform_routes().with_state(state)
}

/// Build the platform router without state (for composition with other routers).
///
/// Use this when you need to merge the platform router with other routers
/// that share the same state type. Call `.with_state(state)` on the final
/// merged router.
///
/// # Example
///
/// ```ignore
/// let gov_routes = gov_http::platform_routes::<AppState>();
/// let app_routes = my_app_routes::<AppState>();
/// let combined = gov_routes.merge(app_routes).with_state(state);
/// ```
pub fn platform_routes<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
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
        // Governance introspection endpoints
        .route("/graph", get(handlers::get_graph::<S>))
        .route("/devex/flows", get(handlers::get_devex_flows::<S>))
        .route("/coverage", get(handlers::get_coverage::<S>))
        // Tasks endpoints
        .route("/tasks", get(handlers::get_tasks::<S>))
        .route("/tasks/suggest-next", get(handlers::get_suggest_next::<S>))
        .route("/tasks/graph", get(handlers::get_task_graph::<S>))
}

/// Build the platform router excluding the simplified `/status` endpoint.
///
/// Use this when your service provides its own richer `/status` endpoint
/// but wants to reuse all other governance endpoints from gov-http.
///
/// # Example
///
/// ```ignore
/// let gov_routes = gov_http::platform_routes_no_status::<AppState>();
/// let app = Router::new()
///     .merge(gov_routes)
///     .route("/status", get(my_rich_status_handler))
///     .with_state(state);
/// ```
pub fn platform_routes_no_status<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new()
        // Core endpoints (excluding /status)
        .route("/health", get(handlers::health))
        // Contract anchor endpoints
        .route("/schema", get(handlers::get_schema))
        .route("/schema/{name}", get(handlers::get_schema_by_name))
        .route("/docs/index", get(handlers::get_docs_index::<S>))
        .route("/ui/contract", get(handlers::get_ui_contract::<S>))
        // Governance introspection endpoints
        .route("/graph", get(handlers::get_graph::<S>))
        .route("/devex/flows", get(handlers::get_devex_flows::<S>))
        .route("/coverage", get(handlers::get_coverage::<S>))
        // Tasks endpoints
        .route("/tasks", get(handlers::get_tasks::<S>))
        .route("/tasks/suggest-next", get(handlers::get_suggest_next::<S>))
        .route("/tasks/graph", get(handlers::get_task_graph::<S>))
        // Governance artifact endpoints
        .merge(friction::router::<S>())
        .merge(questions::router::<S>())
        .merge(forks::router::<S>())
}

/// Build a minimal platform router with only health/status endpoints.
///
/// Use this when you only need basic health checks without full governance.
pub fn minimal_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
}
