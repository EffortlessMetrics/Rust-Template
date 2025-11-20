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
    fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError> {
        tasks_state::update_task_status(&self.state_file_path, task_id.clone(), status)
            .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))
    }
}
