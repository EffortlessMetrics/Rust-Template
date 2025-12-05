//! Use cases (application logic) for task operations.
//!
//! This module contains the business logic functions that orchestrate
//! task creation, retrieval, and status updates using the repository port.

use crate::ports::TaskRepository;
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
