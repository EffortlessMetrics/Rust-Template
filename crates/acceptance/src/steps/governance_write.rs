use crate::World;
use business_core::governance::{TaskId, TaskStatus};
use cucumber::{given, then, when};
use std::fs;

#[given(expr = "a task {string} exists")]
async fn task_exists(_world: &mut World, task_id: String) {
    // In a real scenario, we might create it. For now, we assume it's valid to operate on.
    // We ensure the repo is clean or has this task in a known state if needed.
    // For this sprint, we just ensure the ID is what we expect.
    assert!(!task_id.is_empty());
}

#[when(expr = "the system updates its status to {string}")]
async fn update_task_status(world: &mut World, status_str: String) {
    let task_id = TaskId("TASK-TPL-GOV-WRITE-001".to_string());
    let status = match status_str.as_str() {
        "Todo" => TaskStatus::Todo,
        "InProgress" => TaskStatus::InProgress,
        "Review" => TaskStatus::Review,
        "Done" => TaskStatus::Done,
        _ => panic!("Unknown status: {}", status_str),
    };

    // We need to access the repository from the app state or directly from the world if exposed.
    // Since World holds the app router, we can't easily grab the repo from it without exposing it.
    // However, we injected the repo into the app using a temp dir in World::default().
    // We can reconstruct the repo or, better, expose the temp dir path in World to verify the file.

    // To actually perform the update, we should ideally use the API (Sprint 3) or a direct call.
    // Since the API doesn't exist yet (Sprint 3), we will simulate the "system" updating it
    // by directly calling the repository. This requires us to access the repo or create a new instance pointing to the same temp dir.

    let specs_dir = world._temp_dir.path().to_path_buf();
    let repo = adapters_spec_fs::FsGovernanceRepository::new(specs_dir);

    use business_core::governance::GovernanceRepository;
    repo.set_task_status(&task_id, status).expect("Failed to update task status");
}

#[then(regex = r#"^specs/tasks_state\.yaml should contain that task with status "([^"]+)"$"#)]
async fn verify_task_status(world: &mut World, status_str: String) {
    let specs_dir = world._temp_dir.path().to_path_buf();
    let state_file = specs_dir.join("tasks_state.yaml");

    assert!(state_file.exists(), "tasks_state.yaml should exist");

    let content = fs::read_to_string(state_file).expect("Failed to read tasks_state.yaml");

    // Simple string check for now, or parse YAML if we want to be robust
    assert!(content.contains("TASK-TPL-GOV-WRITE-001"), "Task ID not found in state file");
    assert!(content.contains(status_str.as_str()), "Status not found in state file");
}
