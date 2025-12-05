//! Ports (interfaces) for task persistence and storage.
//!
//! This module defines the boundary traits that adapters must implement.

use model::Task;

/// Port for task persistence.
///
/// Adapters (e.g., filesystem, database) implement this trait to provide
/// concrete task storage and retrieval.
#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    /// Save a new task to storage.
    async fn save(&self, task: &Task) -> Result<(), String>;

    /// Find a task by its unique ID.
    async fn find_by_id(&self, id: &str) -> Result<Option<Task>, String>;

    /// Retrieve all tasks from storage.
    async fn find_all(&self) -> Result<Vec<Task>, String>;

    /// Update a task's status by ID.
    async fn update_status(
        &self,
        id: &str,
        status: model::TaskStatus,
    ) -> Result<Option<Task>, String>;
}
