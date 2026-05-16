//! HTTP application layer for the Rust-as-Spec platform.
//!
//! This crate acts as a facade for the HTTP layer, re-exporting functionality
//! from the focused http-* crates. It provides backward compatibility for existing
//! imports while delegating to the specialized crates.
//!
//! # Architecture
//!
//! The crate follows the facade pattern for dependency isolation:
//!
//! - **Facade Layer (this crate)**: Provides backward-compatible API by re-exporting
//!   from http-* crates.
//! - **Specialized HTTP Crates**: Each http-* crate handles a specific domain:
//!   - `http-core`: Shared foundation (app state traits, common handlers)
//!   - `http-platform`: Platform endpoints (`/platform/*`, UI routes)
//!   - `http-tasks`: Task management endpoints
//!   - `http-todos`: Todo management endpoints
//!   - `http-agents`: Agent hints endpoints
//!   - `http-metrics`: Prometheus metrics middleware and endpoint
//!   - `http-middleware`: Cross-cutting middleware
//!
//! Dependencies point inward: http-* → http-core → platform-contract (correct).
//!
//! # Main Components
//!
//! ## Router and Handlers
//!
//! The application provides several categories of endpoints:
//!
//! - **Core Template Endpoints**: `/health`, `/version`, `/metrics` - operational endpoints
//! - **Platform Introspection**: `/platform/*` - governance graph, specs, tasks, docs
//! - **Platform UI**: `/ui/*` - web interfaces for tasks and governance visualization
//! - **Domain Endpoints**: `/api/todos`, `/api/agent/*` - business logic endpoints
//!
//! ## Middleware Stack
//!
//! Applied in reverse order (bottom-to-top in code):
//!
//! 1. **Request ID** (`middleware::request_id_middleware`): Generates or propagates correlation IDs
//! 2. **Metrics** (`metrics::metrics_middleware`): Tracks request latency and counts
//! 3. **CORS** (`middleware::cors_middleware`): Configurable cross-origin resource sharing
//! 4. **Security Headers** (`middleware::security_headers_middleware`): CSP, X-Frame-Options, HSTS
//!
//! ## Security Features
//!
//! - **Platform Authentication**: JWT-based auth for `/platform/*` endpoints
//! - **Security Headers**: Comprehensive security header configuration
//! - **CORS**: Configurable origin validation and credentials handling
//! - **Request ID Propagation**: Correlation IDs for request tracing and audit trails
//!
//! ## Error Handling
//!
//! Error types are re-exported from `http-errors` crate:
//!
//! - **Machine-readable error codes** (`ErrorCode` enum)
//! - **AC/Feature ID tracking** for governance alignment
//! - **Structured logging** with request correlation
//! - **JSON error envelopes** with consistent format and context
//!
//! # Integration with Other Crates
//!
//! ## `http-core`
//!
//! Provides shared HTTP foundation including app state traits, common handlers,
//! and shutdown signal handling.
//!
//! ## `http-platform`
//!
//! Provides `/platform/*` endpoints for governance introspection and UI routes.
//!
//! ## `http-tasks`
//!
//! Provides task management endpoints.
//!
//! ## `http-todos`
//!
//! Provides todo management endpoints.
//!
//! ## `http-agents`
//!
//! Provides agent hints endpoints.
//!
//! ## `http-metrics`
//!
//! Provides request metrics middleware and `/metrics` handler.
//!
//! ## `http-middleware`
//!
//! Provides cross-cutting middleware (CORS, security headers, request ID).
//!
//! ## `business-core`
//!
//! Provides domain logic accessed by handlers.
//!
//! ## `spec-runtime`
//!
//! Validates configuration files at startup.
//!
//! ## `telemetry`
//!
//! Initialized at application startup for structured logging and metrics.
//!
//! # Usage
//!
//! ## Creating an Application
//!
//! ```rust,no_run
//! use app_http::app;
//! use adapters_spec_fs::FsGovernanceRepository;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let repo = Arc::new(FsGovernanceRepository::new("/workspace/root".into()));
//! let router = app(repo).expect("invalid platform auth configuration");
//!
//! // Serve with axum
//! let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
//! axum::serve(listener, router).await.unwrap();
//! # }
//! ```
//!
//! # Key Features
//!
//! - **Dependency Isolation**: Each http-* crate has minimal, focused dependencies
//! - **Backward Compatibility**: Existing imports continue to work via re-exports
//! - **Governance-first**: All endpoints integrate with the platform's governance system
//! - **Observability**: Request IDs, structured logs, metrics, distributed tracing
//! - **Security**: JWT auth, security headers, CORS, defense-in-depth
//! - **Type-safe Config**: Schema-validated YAML configuration
//! - **AC Traceability**: Error responses link to Acceptance Criteria IDs
// ============================================================================
// Re-exports from http-* crates
// ============================================================================

// Re-export from http-core
pub use http_core::{
    AppState as CoreAppState, base_router, resolve_workspace_root, shutdown_signal,
};

// Re-export from http-platform
pub use http_platform::{
    // Re-exported gov-http types
    CoverageDetail,
    CoverageResponse,
    CoverageSummary,
    DebugInfo,
    DocHealthSummary,
    DocInfoWithHealth,
    DocsIndexResponse,
    ForkEntry,
    ForkSummary,
    ForksListResponse,
    FrictionContext,
    FrictionEntry,
    FrictionListResponse,
    // Re-exported IDP types
    IdpSnapshot,
    PlatformState as HttpPlatformState,
    Question,
    QuestionContext,
    QuestionFilters,
    QuestionSummary,
    QuestionsListResponse,
    SuggestNextQuery,
    TaskDocsOut,
    TaskFilters,
    TaskGraphQuery,
    TaskGraphResponse,
    TaskOut,
    TasksResponse,
    router as platform_router,
    ui_router,
};

// Re-export from http-tasks
pub use http_tasks::{TasksState, router as tasks_router, tasks_ui, update_task_status};

// Re-export from http-todos
pub use http_todos::{CreateTodoRequest, TodosStateTrait, router as todos_router};

// Re-export from http-agents
pub use http_agents::{
    AgentHint, AgentHintReason, AgentHintsResponse, AgentsState, HintsFilters, RecommendedStep,
    router as agents_router,
};

// Re-export from http-metrics
pub use http_metrics::{metrics_handler, metrics_middleware};

// Re-export from app-http internal modules (backward compatibility)
pub use errors::{AppError, ErrorCode, ErrorSummary, get_error_summary};
pub use middleware::{
    CorsConfig, REQUEST_ID_HEADER, RequestId, SecurityHeadersConfig, cors_middleware,
    platform_auth_guard, request_id_middleware, security_headers_middleware,
};

// ============================================================================
// Public modules (kept for backward compatibility)
// ============================================================================

// Note: Most modules have been moved to http-* crates.
// These are kept as thin wrappers for backward compatibility.

// Public modules (kept for backward compatibility)
pub mod errors;
pub mod metrics;
pub mod middleware;
pub mod security;
pub mod shutdown;

// Compatibility modules for legacy paths
pub mod platform {
    pub use http_platform::*;
}

// Re-export commonly used types (backward compatibility)
pub use security::PlatformAuthConfig;

// ============================================================================
// Focused internal modules
// ============================================================================

mod config;
mod handlers;
mod router;
mod state;

pub use router::{app, app_with_state, app_with_workspace_root};
pub use state::AppState;

pub(crate) use config::load_valid_config;
pub(crate) use handlers::{echo, health, version};
