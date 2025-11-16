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
    use crate::model::Task;

    /// Port for task persistence
    pub trait TaskRepository {
        fn save(&self, task: &Task) -> Result<(), String>;
    }
}

pub mod use_cases {
    use super::ports::TaskRepository;
    use crate::model::{Task, TaskStatus};

    /// Create a new task
    pub fn create_task(repo: &impl TaskRepository, title: String) -> Result<Task, String> {
        let task = Task {
            id: "placeholder-id".to_string(),
            title,
            status: TaskStatus::Pending,
        };
        repo.save(&task)?;
        Ok(task)
    }
}