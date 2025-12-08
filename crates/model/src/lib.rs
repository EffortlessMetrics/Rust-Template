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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
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
