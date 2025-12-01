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

pub mod ports {
    use model::Task;

    /// Port for task persistence
    #[async_trait::async_trait]
    pub trait TaskRepository: Send + Sync {
        async fn save(&self, task: &Task) -> Result<(), String>;
        async fn find_by_id(&self, id: &str) -> Result<Option<Task>, String>;
        async fn find_all(&self) -> Result<Vec<Task>, String>;
        async fn update_status(
            &self,
            id: &str,
            status: model::TaskStatus,
        ) -> Result<Option<Task>, String>;
    }
}

pub mod use_cases {
    use super::ports::TaskRepository;
    use model::{Task, TaskStatus};

    /// Create a new task
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

    pub async fn get_task(repo: &dyn TaskRepository, id: String) -> Result<Option<Task>, String> {
        repo.find_by_id(&id).await
    }

    pub async fn list_tasks(repo: &dyn TaskRepository) -> Result<Vec<Task>, String> {
        repo.find_all().await
    }

    pub async fn update_task_status(
        repo: &dyn TaskRepository,
        id: String,
        status: TaskStatus,
    ) -> Result<Option<Task>, String> {
        repo.update_status(&id, status).await
    }
}

pub mod governance {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TaskStatus {
        Todo,
        InProgress,
        Review,
        Done,
    }

    impl TaskStatus {
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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Task {
        pub id: TaskId,
        pub title: String,
        pub status: TaskStatus,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct TaskId(pub String);

    #[derive(Debug, thiserror::Error)]
    pub enum GovernanceError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Serialization error: {0}")]
        Serialization(String),
        #[error("Task not found: {0:?}")]
        TaskNotFound(TaskId),
        #[error("Lock error: {0}")]
        Lock(String),
        #[error("Invalid transition from {from:?} to {to:?}")]
        InvalidTransition { from: TaskStatus, to: TaskStatus },
    }

    pub trait GovernanceRepository: Send + Sync {
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError>;
        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError>;
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

    pub struct TaskService<R: GovernanceRepository> {
        repo: R,
    }

    impl<R: GovernanceRepository> TaskService<R> {
        pub fn new(repo: R) -> Self {
            Self { repo }
        }

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
