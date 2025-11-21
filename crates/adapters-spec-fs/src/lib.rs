pub mod tasks_state;

use business_core::governance::{GovernanceError, GovernanceRepository, TaskId, TaskStatus};
use std::path::PathBuf;

pub struct FsGovernanceRepository {
    state_file_path: PathBuf,
}

impl FsGovernanceRepository {
    pub fn new(specs_dir: PathBuf) -> Self {
        Self { state_file_path: specs_dir.join("tasks_state.yaml") }
    }
}

impl GovernanceRepository for FsGovernanceRepository {
    fn load_task(
        &self,
        task_id: &TaskId,
    ) -> Result<business_core::governance::Task, GovernanceError> {
        let status = tasks_state::get_task_status(&self.state_file_path, task_id)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?
            .unwrap_or(TaskStatus::Todo);

        Ok(business_core::governance::Task {
            id: task_id.clone(),
            title: "Placeholder Title".to_string(), // TODO: Fetch from tasks.yaml in Sprint 3
            status,
        })
    }

    fn find_all_tasks(&self) -> Result<Vec<business_core::governance::Task>, GovernanceError> {
        let tasks_map = tasks_state::get_all_tasks(&self.state_file_path)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?;

        let tasks = tasks_map
            .into_iter()
            .map(|(id, status)| business_core::governance::Task {
                id,
                title: "Placeholder Title".to_string(),
                status,
            })
            .collect();

        Ok(tasks)
    }

    fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError> {
        tasks_state::update_task_status(&self.state_file_path, task_id.clone(), status)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))
    }
}
