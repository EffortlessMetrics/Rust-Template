//! Core business logic for the Rust-as-Spec platform cell.
//!
//! This crate defines the domain model and governance traits used by the
//! rest of the workspace, including:
//! - `ExampleTaskRepository` – persistence boundary for example task state
//! - `ExampleTask` types (from `model` crate) – example CRUD lifecycle
//!
//! Adapters (HTTP handlers, database drivers, event systems) implement these
//! ports to provide concrete storage and transport. The kernel treats this crate
//! as the source of truth for governance semantics.
//!
//! ## Architecture
//!
//! - No dependencies on HTTP, database, or other infrastructure adapters
//! - Adapters (app-http, app-db, etc.) call core, never the reverse
//! - Core defines ports (traits), adapters implement them

/// Ports (interfaces) for example task persistence and storage.
///
/// This module defines the boundary traits that adapters must implement.
pub mod ports {
    use model::ExampleTask;

    /// Port for example task persistence.
    ///
    /// Adapters (e.g., filesystem, database) implement this trait to provide
    /// concrete task storage and retrieval.
    #[async_trait::async_trait]
    pub trait ExampleTaskRepository: Send + Sync {
        /// Save a new task to storage.
        async fn save(&self, task: &ExampleTask) -> Result<(), String>;

        /// Find a task by its unique ID.
        async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, String>;

        /// Retrieve all tasks from storage.
        async fn find_all(&self) -> Result<Vec<ExampleTask>, String>;

        /// Update a task's status by ID.
        async fn update_status(
            &self,
            id: &str,
            status: model::ExampleTaskStatus,
        ) -> Result<Option<ExampleTask>, String>;
    }
}

/// Use cases (application logic) for example task operations.
///
/// This module contains the business logic functions that orchestrate
/// task creation, retrieval, and status updates using the repository port.
pub mod use_cases {
    use super::ports::ExampleTaskRepository;
    use model::{ExampleTask, ExampleTaskStatus};

    /// Create a new example task with a given title.
    ///
    /// Generates a new UUID, sets the status to Pending, and saves the task.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository save operation fails.
    pub async fn create_example_task(
        repo: &dyn ExampleTaskRepository,
        title: String,
    ) -> Result<ExampleTask, String> {
        let task = ExampleTask {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            status: ExampleTaskStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        repo.save(&task).await?;
        Ok(task)
    }

    /// Retrieve an example task by its unique ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn get_example_task(
        repo: &dyn ExampleTaskRepository,
        id: String,
    ) -> Result<Option<ExampleTask>, String> {
        repo.find_by_id(&id).await
    }

    /// List all example tasks from the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn list_example_tasks(
        repo: &dyn ExampleTaskRepository,
    ) -> Result<Vec<ExampleTask>, String> {
        repo.find_all().await
    }

    /// Update an example task's status by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn update_example_task_status(
        repo: &dyn ExampleTaskRepository,
        id: String,
        status: ExampleTaskStatus,
    ) -> Result<Option<ExampleTask>, String> {
        repo.update_status(&id, status).await
    }
}

/// Governance domain model for task management.
///
/// This module re-exports types from the `gov-model` crate for backward compatibility.
/// For new code, prefer importing directly from `gov_model`.
///
/// This is the production governance model with a rich four-state workflow
/// (Todo → InProgress → Review → Done). It is separate from the example CRUD
/// model in the `model` crate, which uses a simpler three-state TaskStatus enum.
pub mod governance {
    // Re-export all governance types from gov-model crate
    pub use gov_model::{
        GovernanceError, GovernanceRepository, Task, TaskId, TaskService, TaskStatus,
        TaskStatusParseError,
    };
}
