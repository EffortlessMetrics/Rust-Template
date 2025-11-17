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
    pub trait TaskRepository {
        fn save(&self, task: &Task) -> Result<(), String>;
        fn find_by_id(&self, id: &str) -> Result<Option<Task>, String>;
        fn find_all(&self) -> Result<Vec<Task>, String>;
        fn update_status(
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
    pub fn create_task(repo: &impl TaskRepository, title: String) -> Result<Task, String> {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        repo.save(&task)?;
        Ok(task)
    }

    pub fn get_task(repo: &impl TaskRepository, id: String) -> Result<Option<Task>, String> {
        repo.find_by_id(&id)
    }

    pub fn list_tasks(repo: &impl TaskRepository) -> Result<Vec<Task>, String> {
        repo.find_all()
    }

    pub fn update_task_status(
        repo: &impl TaskRepository,
        id: String,
        status: TaskStatus,
    ) -> Result<Option<Task>, String> {
        repo.update_status(&id, status)
    }
}
