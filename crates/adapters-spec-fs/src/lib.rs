pub mod tasks_def;
pub mod tasks_state;

use business_core::governance::{GovernanceError, GovernanceRepository, TaskId, TaskStatus};
use std::path::PathBuf;

pub struct FsGovernanceRepository {
    state_file_path: PathBuf,
    tasks_file_path: PathBuf,
}

impl FsGovernanceRepository {
    pub fn new(specs_dir: PathBuf) -> Self {
        Self {
            state_file_path: specs_dir.join("tasks_state.yaml"),
            tasks_file_path: specs_dir.join("tasks.yaml"),
        }
    }
}

/// Parse a task status string from tasks.yaml to TaskStatus enum
fn parse_task_status(status_str: &str) -> Option<TaskStatus> {
    match status_str.to_lowercase().as_str() {
        "todo" | "open" => Some(TaskStatus::Todo),
        "in_progress" | "inprogress" | "in-progress" => Some(TaskStatus::InProgress),
        "review" => Some(TaskStatus::Review),
        "done" | "closed" | "complete" | "completed" => Some(TaskStatus::Done),
        _ => None,
    }
}

impl GovernanceRepository for FsGovernanceRepository {
    fn load_task(
        &self,
        task_id: &TaskId,
    ) -> Result<business_core::governance::Task, GovernanceError> {
        // Load status from tasks_state.yaml
        let status = tasks_state::get_task_status(&self.state_file_path, task_id)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?
            .unwrap_or(TaskStatus::Todo);

        // Load title from tasks.yaml
        let definitions = tasks_def::load_tasks_definitions(&self.tasks_file_path)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?;

        let title = definitions
            .get(task_id.0.as_str())
            .map(|def| def.title.clone())
            .unwrap_or_else(|| task_id.0.clone());

        Ok(business_core::governance::Task { id: task_id.clone(), title, status })
    }

    fn find_all_tasks(&self) -> Result<Vec<business_core::governance::Task>, GovernanceError> {
        // Load definitions from tasks.yaml (source of truth for task list)
        let definitions = tasks_def::load_tasks_definitions(&self.tasks_file_path)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?;

        // Load status overrides from tasks_state.yaml (if exists)
        let state_map = tasks_state::get_all_tasks(&self.state_file_path)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?;

        let tasks = definitions
            .into_iter()
            .map(|(task_id, def)| {
                let id = TaskId(task_id);

                // Use status from state file if exists, otherwise parse from definition
                let status = state_map
                    .get(&id)
                    .cloned()
                    .or_else(|| def.status.as_ref().and_then(|s| parse_task_status(s.as_str())))
                    .unwrap_or(TaskStatus::Todo);

                business_core::governance::Task { id, title: def.title, status }
            })
            .collect();

        Ok(tasks)
    }

    fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError> {
        tasks_state::update_task_status(&self.state_file_path, task_id.clone(), status)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))
    }
}
