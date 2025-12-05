//! Governance domain model for task management.
//!
//! This module contains the core domain types for governance workflows,
//! including task states, transitions, and the governance repository trait.

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
