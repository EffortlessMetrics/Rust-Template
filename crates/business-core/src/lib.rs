//! Core business logic for the Rust-as-Spec platform cell.
//!
//! This crate defines the domain model and governance traits used by the
//! rest of the workspace, including:
//! - `TaskRepository` – persistence boundary for task state
//! - `Task` types (from `model` crate) – core task lifecycle
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

/// Ports (interfaces) for task persistence and storage.
///
/// This module defines the boundary traits that adapters must implement.
pub mod ports {
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
}

/// Use cases (application logic) for task operations.
///
/// This module contains the business logic functions that orchestrate
/// task creation, retrieval, and status updates using the repository port.
pub mod use_cases {
    use super::ports::TaskRepository;
    use model::{Task, TaskStatus};

    /// Create a new task with a given title.
    ///
    /// Generates a new UUID, sets the status to Pending, and saves the task.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository save operation fails.
    pub async fn create_task(repo: &dyn TaskRepository, title: String) -> Result<Task, String> {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        repo.save(&task).await?;
        Ok(task)
    }

    /// Retrieve a task by its unique ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn get_task(repo: &dyn TaskRepository, id: String) -> Result<Option<Task>, String> {
        repo.find_by_id(&id).await
    }

    /// List all tasks from the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn list_tasks(repo: &dyn TaskRepository) -> Result<Vec<Task>, String> {
        repo.find_all().await
    }

    /// Update a task's status by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository operation fails.
    pub async fn update_task_status(
        repo: &dyn TaskRepository,
        id: String,
        status: TaskStatus,
    ) -> Result<Option<Task>, String> {
        repo.update_status(&id, status).await
    }
}

/// Governance domain model for task management.
///
/// This module contains the core domain types for governance workflows,
/// including task states, transitions, and the governance repository trait.
pub mod governance {
    use serde::{Deserialize, Serialize};

    /// Task status in the governance workflow.
    ///
    /// Defines the allowed states and valid transitions for governance tasks.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TaskStatus {
        /// Task is not yet started.
        Todo,
        /// Task is actively being worked on.
        InProgress,
        /// Task is complete and awaiting review.
        Review,
        /// Task is fully complete and approved.
        Done,
    }

    impl TaskStatus {
        /// Check if a transition from the current status to a new status is valid.
        ///
        /// Allowed transitions:
        /// - Todo → InProgress
        /// - InProgress → Review (or back to Todo)
        /// - Review → Done (or back to InProgress)
        pub fn can_transition_to(&self, next: &TaskStatus) -> bool {
            use TaskStatus::*;
            match (self, next) {
                (Todo, InProgress) => true,
                (InProgress, Review) => true,
                (Review, Done) => true,
                (Review, InProgress) => true, // Backwards allowed
                (InProgress, Todo) => true,   // Backwards allowed
                _ => false,
            }
        }
    }

    /// A governance task with ID, title, and status.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Task {
        /// Unique task identifier.
        pub id: TaskId,
        /// Task title or description.
        pub title: String,
        /// Current workflow status.
        pub status: TaskStatus,
    }

    /// Unique identifier for a governance task.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct TaskId(pub String);

    /// Errors that can occur during governance operations.
    #[derive(Debug, thiserror::Error)]
    pub enum GovernanceError {
        /// IO error during repository operations.
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        /// Serialization/deserialization error.
        #[error("Serialization error: {0}")]
        Serialization(String),
        /// Task not found in the repository.
        #[error("Task not found: {0:?}")]
        TaskNotFound(TaskId),
        /// Lock acquisition failure.
        #[error("Lock error: {0}")]
        Lock(String),
        /// Invalid state transition attempted.
        #[error("Invalid transition from {from:?} to {to:?}")]
        InvalidTransition {
            /// Source status.
            from: TaskStatus,
            /// Target status.
            to: TaskStatus,
        },
    }

    /// Repository trait for governance task persistence.
    ///
    /// Adapters implement this trait to provide concrete storage mechanisms.
    pub trait GovernanceRepository: Send + Sync {
        /// Load a task by its ID.
        ///
        /// # Errors
        ///
        /// Returns [`GovernanceError::TaskNotFound`] if the task does not exist.
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError>;

        /// Retrieve all tasks from storage.
        ///
        /// # Errors
        ///
        /// Returns an error if the repository operation fails.
        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError>;

        /// Update a task's status.
        ///
        /// # Errors
        ///
        /// Returns an error if the task is not found or the update fails.
        fn set_task_status(
            &self,
            task_id: &TaskId,
            status: TaskStatus,
        ) -> Result<(), GovernanceError>;
    }

    impl GovernanceRepository for std::sync::Arc<dyn GovernanceRepository> {
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
            (**self).load_task(task_id)
        }

        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
            (**self).find_all_tasks()
        }

        fn set_task_status(
            &self,
            task_id: &TaskId,
            status: TaskStatus,
        ) -> Result<(), GovernanceError> {
            (**self).set_task_status(task_id, status)
        }
    }

    /// Service for managing task lifecycle and transitions.
    ///
    /// Orchestrates task operations and enforces state transition rules.
    pub struct TaskService<R: GovernanceRepository> {
        repo: R,
    }

    impl<R: GovernanceRepository> TaskService<R> {
        /// Create a new task service with the given repository.
        pub fn new(repo: R) -> Self {
            Self { repo }
        }

        /// Move a task to a new status, enforcing transition rules.
        ///
        /// # Errors
        ///
        /// Returns [`GovernanceError::InvalidTransition`] if the transition is not allowed.
        /// Returns [`GovernanceError::TaskNotFound`] if the task does not exist.
        pub fn move_task(
            &self,
            id: &TaskId,
            new_status: TaskStatus,
        ) -> Result<(), GovernanceError> {
            let mut task = self.repo.load_task(id)?;
            if !task.status.can_transition_to(&new_status) {
                return Err(GovernanceError::InvalidTransition {
                    from: task.status,
                    to: new_status,
                });
            }
            task.status = new_status.clone();
            self.repo.set_task_status(id, new_status)?;
            Ok(())
        }

        /// List all tasks from the repository.
        ///
        /// # Errors
        ///
        /// Returns an error if the repository operation fails.
        pub fn list_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
            self.repo.find_all_tasks()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_allowed_transitions() {
            assert!(TaskStatus::Todo.can_transition_to(&TaskStatus::InProgress));
            assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Review));
            assert!(TaskStatus::Review.can_transition_to(&TaskStatus::Done));
            assert!(TaskStatus::Review.can_transition_to(&TaskStatus::InProgress));
        }

        #[test]
        fn test_forbidden_transitions() {
            assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::Todo));
            assert!(!TaskStatus::Todo.can_transition_to(&TaskStatus::Done));
        }
    }
}
