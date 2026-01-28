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
//!
//! # Architecture
//!
//! This crate acts as a facade that re-exports types from subrouter crates:
//! - `gov-http-core` - Shared foundation (errors, state, extractors)
//! - `gov-http-forks` - Forks endpoints
//! - `gov-http-friction` - Friction endpoints
//! - `gov-http-questions` - Questions endpoints
//! - `gov-http-issues` - Issues endpoints
//!
//! The core handlers (health, schema, docs, tasks) remain in this crate.

// Re-export from subrouter crates for backward compatibility
pub use gov_http_core::state::DefaultPlatformState;
pub use gov_http_core::{PlatformError, PlatformState, RequestId};
pub use gov_http_forks::{ForkEntry, ForkSummary, ForksListResponse, Maintainer};
pub use gov_http_friction::{FrictionContext, FrictionEntry, FrictionListResponse, Resolution};
pub use gov_http_issues::{
    Issue, IssueFilters, IssueKind, IssueStatus, IssuesResponse, IssuesSummary, KindCounts,
    Pagination, StatusCounts,
};
pub use gov_http_questions::{
    Question, QuestionContext, QuestionFilters, QuestionOption, QuestionResolution,
    QuestionSummary, QuestionsListResponse, Recommendation,
};

// Core handlers remain in this crate
pub mod handlers;
pub use handlers::{
    CoverageDetail, CoverageResponse, CoverageSummary, DocHealthSummary, DocInfoWithHealth,
    DocsIndexResponse, SuggestNextQuery, TaskDocsOut, TaskFilters, TaskGraphQuery,
    TaskGraphResponse, TaskOut, TasksResponse,
};

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
/// - `/openapi` - OpenAPI spec (YAML)
/// - `/openapi.yaml` - OpenAPI spec alias (YAML)
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
///
/// # Governance Artifact Endpoints
///
/// - `/forks/*` - Forks registry (via gov-http-forks)
/// - `/friction/*` - Friction log (via gov-http-friction)
/// - `/questions/*` - Questions (via gov-http-questions)
/// - `/issues` - Unified issues (via gov-http-issues)
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
        .route("/openapi", get(handlers::get_openapi::<S>))
        .route("/openapi.yaml", get(handlers::get_openapi::<S>))
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
        // Governance artifact endpoints (from subrouter crates)
        .merge(gov_http_forks::router::<S>())
        .merge(gov_http_friction::router::<S>())
        .merge(gov_http_questions::router::<S>())
        .merge(gov_http_issues::router::<S>())
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
        .route("/openapi", get(handlers::get_openapi::<S>))
        .route("/openapi.yaml", get(handlers::get_openapi::<S>))
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
        // Governance artifact endpoints (from subrouter crates)
        .merge(gov_http_forks::router::<S>())
        .merge(gov_http_friction::router::<S>())
        .merge(gov_http_questions::router::<S>())
        .merge(gov_http_issues::router::<S>())
}

/// Build a minimal platform router with only health/status endpoints.
///
/// Use this when you only need basic health checks without full governance.
pub fn minimal_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
}
