//! Shared types for the Rust-as-Spec platform cell.
//!
//! This crate contains data structures that are shared across the workspace
//! (core business logic, HTTP handlers, database drivers, UI, and governance logic).
//!
//! The goal is to keep cross-crate types centralized, so that changes to the
//! platform model are visible and reviewable in one place.
//!
//! ## Design principles
//!
//! - Lightweight: only pure data structures and value objects (no business logic)
//! - Reusable: can be serialized for HTTP/gRPC/storage without adaptation
//! - Stable: changes here ripple across the entire workspace
//!

/// Example CRUD task model for demonstrating basic REST operations.
///
/// This is a simple example domain model with a basic three-state lifecycle
/// (Pending → InProgress → Completed). It is separate from the production
/// governance model in `business_core::governance`, which uses a different
/// TaskStatus enum with richer workflow states (Todo/InProgress/Review/Done).
///
/// **Note:** This type is intentionally named `ExampleTask` (not just `Task`)
/// to avoid confusion with `gov_model::Task` used in production governance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleTask {
    pub id: String,
    pub title: String,
    pub status: ExampleTaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Basic task status for example CRUD operations.
///
/// This enum is used by the example `ExampleTask` model above. For production
/// governance workflows, see `gov_model::TaskStatus` instead.
///
/// **Note:** This type is intentionally named `ExampleTaskStatus` (not just
/// `TaskStatus`) to avoid confusion with `gov_model::TaskStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExampleTaskStatus {
    Pending,
    InProgress,
    Completed,
}

/// Todo item for the MYSERV domain
///
/// Example domain model demonstrating AC-MYSERV-001:
/// GET /todos returns a JSON array of todos with id and title fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
}
