// Domain models and entities
//
// This crate contains shared domain types that can be used across:
// - Core business logic (crates/core)
// - HTTP adapters (crates/app-http)
// - Database adapters (crates/app-db, if you add one)
// - Event adapters (crates/app-events, if you add one)
//
// Keep this crate lightweight - only pure data structures and value objects.
// Business logic belongs in crates/core, not here.
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}