// Core business logic goes here
//
// This crate should contain:
// - Domain entities and business rules
// - Use case / application service logic
// - Port definitions (traits for adapters to implement)
//
// Architecture principles:
// - No dependencies on HTTP, database, or other adapters
// - Adapters (app-http, app-db, etc.) call core, never the reverse
// - Core defines ports (traits), adapters implement them
//

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
    }

    pub trait GovernanceRepository: Send + Sync {
        fn set_task_status(
            &self,
            task_id: &TaskId,
            status: TaskStatus,
        ) -> Result<(), GovernanceError>;
    }
}
