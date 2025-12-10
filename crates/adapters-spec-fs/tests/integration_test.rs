//! Integration tests for the FsGovernanceRepository adapter.

use adapters_spec_fs::FsGovernanceRepository;
use business_core::governance::{GovernanceRepository, TaskId, TaskStatus};
use std::fs;

#[test]
fn test_set_task_status_persists() {
    // Setup temp dir
    let temp_dir = tempfile::tempdir().unwrap();
    let specs_dir = temp_dir.path().to_path_buf();

    // Initialize repo
    let repo = FsGovernanceRepository::new(specs_dir.clone());

    // Define task and status
    let task_id = TaskId("TASK-123".to_string());
    let status = TaskStatus::InProgress;

    // Set status
    repo.set_task_status(&task_id, status).unwrap();

    // Verify file exists
    let state_file = specs_dir.join("tasks_state.yaml");
    assert!(state_file.exists());

    // Verify content
    let content = fs::read_to_string(state_file).unwrap();
    assert!(content.contains("TASK-123"));
    assert!(content.contains("InProgress"));

    // Update status
    let new_status = TaskStatus::Done;
    repo.set_task_status(&task_id, new_status).unwrap();

    // Verify update
    let content = fs::read_to_string(specs_dir.join("tasks_state.yaml")).unwrap();
    assert!(content.contains("Done"));
}

#[test]
fn load_task_falls_back_to_tasks_yaml_status() {
    // Setup temp dir with tasks.yaml but no tasks_state.yaml entry
    let temp_dir = tempfile::tempdir().unwrap();
    let specs_dir = temp_dir.path().to_path_buf();

    let tasks_yaml = r#"
schema_version: "1.0"
template_version: "1.0"
tasks:
  - id: TASK-STATELESS
    title: "Task without state"
    requirement: REQ-TEST
    status: in_progress
    acs: []
    labels: []
    summary: "Example"
    recommended_flows: []
    docs:
      design: []
      plan: []
"#;

    fs::create_dir_all(&specs_dir).unwrap();
    fs::write(specs_dir.join("tasks.yaml"), tasks_yaml.trim()).unwrap();

    let repo = FsGovernanceRepository::new(specs_dir.clone());
    let task = repo.load_task(&TaskId("TASK-STATELESS".to_string())).unwrap();

    assert_eq!(task.status, TaskStatus::InProgress);
}
