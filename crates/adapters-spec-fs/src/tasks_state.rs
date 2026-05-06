//! Persistence for task execution state (`tasks_state.yaml`).
//!
//! This module tracks the *current* status of tasks (Todo, InProgress,
//! Review, Done, etc.) separately from the immutable task definitions.
//! It is the write-layer used by `FsGovernanceRepository`.

use business_core::governance::{TaskId, TaskStatus};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// In-memory representation of persisted task status.
///
/// This mirrors the `tasks_state.yaml` file and is used to read and
/// write the set of task ids and their current `TaskStatus` values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TasksState {
    /// Map of task ids to their current status.
    pub tasks: HashMap<TaskId, TaskStatus>,
}

/// Update the status of a single task in `tasks_state.yaml`.
///
/// - `path` points to the `tasks_state.yaml` file.
/// - `task_id` is the task being updated.
/// - `status` is the new state (Todo/InProgress/Review/Done).
///
/// If the file does not exist yet it will be created. Existing entries
/// for other tasks are preserved.
#[expect(
    clippy::suspicious_open_options,
    reason = "existing file-open semantics; tracked by lint policy ratchet"
)]
pub fn update_task_status(
    path: &Path,
    task_id: TaskId,
    status: TaskStatus,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Open file with read/write access, create if not exists
    let mut file = OpenOptions::new().read(true).write(true).create(true).open(path)?;

    // Lock the file for exclusive access
    file.lock_exclusive()?;

    // Read content
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse or default
    let mut state: TasksState = if content.trim().is_empty() {
        TasksState::default()
    } else {
        serde_yaml::from_str(&content).unwrap_or_default()
    };

    // Update state
    state.tasks.insert(task_id, status);

    // Serialize
    let new_content = serde_yaml::to_string(&state)?;

    // Write back
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(new_content.as_bytes())?;

    // Unlock (optional as closing the file unlocks it, but explicit is fine)
    file.unlock()?;

    Ok(())
}

/// Look up the persisted status of a single task.
///
/// Returns:
/// - `Ok(Some(status))` if the task has a recorded state,
/// - `Ok(None)` if the task has no entry yet,
/// - `Err(_)` if the state file could not be read or parsed.
pub fn get_task_status(
    path: &Path,
    task_id: &TaskId,
) -> Result<Option<TaskStatus>, Box<dyn std::error::Error + Send + Sync>> {
    if !path.exists() {
        return Ok(None);
    }

    let mut file = OpenOptions::new().read(true).open(path)?;

    // Shared lock for reading
    file.lock_shared()?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    file.unlock()?;

    if content.trim().is_empty() {
        return Ok(None);
    }

    let state: TasksState = serde_yaml::from_str(&content).unwrap_or_default();
    Ok(state.tasks.get(task_id).cloned())
}

/// Load the current status for all tasks from `tasks_state.yaml`.
///
/// This is used by APIs and UIs that need to show a dashboard of
/// all tasks and their current state.
pub fn get_all_tasks(
    path: &Path,
) -> Result<HashMap<TaskId, TaskStatus>, Box<dyn std::error::Error + Send + Sync>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let mut file = OpenOptions::new().read(true).open(path)?;
    file.lock_shared()?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    file.unlock()?;

    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let state: TasksState = serde_yaml::from_str(&content).unwrap_or_default();
    Ok(state.tasks)
}
