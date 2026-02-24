use adapters_spec_fs::tasks_state;
use app_http::{app_with_workspace_root, platform::TasksResponse};
use axum::{body::Body, http::Request};
use business_core::governance::{TaskId, TaskStatus};
use http_body_util::BodyExt;
use spec_runtime::tasks::{Task, TaskDocs, TasksSpec};
use std::{fs, sync::Arc};
use tempfile::tempdir;
use tower::ServiceExt;

fn write_tasks_files(spec_root: &std::path::Path, task_id: &str, status: TaskStatus) {
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Seed tasks_state.yaml with the desired status
    let state_path = specs_dir.join("tasks_state.yaml");
    tasks_state::update_task_status(&state_path, TaskId(task_id.to_string()), status)
        .expect("failed to write tasks_state.yaml");

    // Seed tasks.yaml with a matching task definition
    let tasks_yaml = specs_dir.join("tasks.yaml");
    let tasks = TasksSpec {
        schema_version: "1.0.0".to_string(),
        template_version: "0.1.0".to_string(),
        tasks: vec![Task {
            id: task_id.to_string(),
            title: "Test Task".to_string(),
            requirement: "REQ-TPL-TEST".to_string(),
            acs: vec![],
            status: "Todo".to_string(),
            owner: None,
            labels: vec![],
            docs: Some(TaskDocs { design: vec![], plan: vec![] }),
            summary: "Test task summary".to_string(),
            recommended_flows: vec![],
            depends_on: vec![],
        }],
    };

    let content = serde_yaml::to_string(&tasks).expect("failed to serialize tasks.yaml");
    fs::write(tasks_yaml, content).expect("failed to write tasks.yaml");
}

#[tokio::test]
async fn update_task_status_endpoint_accepts_json_body() {
    // Use an isolated temp repo
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-001", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let body = r#"{ "status": "InProgress" }"#;
    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-001/status")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    if !status.is_success() {
        let body =
            BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    // Verify the status was persisted
    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-001".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}

#[tokio::test]
async fn update_task_status_endpoint_accepts_form_body() {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-002", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-002/status")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("status=InProgress"))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    if !status.is_success() {
        let body =
            BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-002".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}

#[tokio::test]
async fn update_task_status_endpoint_accepts_form_body_without_content_type() {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-002-NO-CT", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-002-NO-CT/status")
        .body(Body::from("status=InProgress"))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    if !status.is_success() {
        let body =
            BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-002-NO-CT".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}

#[tokio::test]
async fn tasks_endpoint_returns_persisted_status() {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-003", TaskStatus::Review);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let request = Request::builder()
        .method("GET")
        .uri("/platform/tasks")
        .body(Body::empty())
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    let body =
        BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();

    if !status.is_success() {
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let tasks: TasksResponse =
        serde_json::from_slice(&body).expect("failed to deserialize tasks response");

    assert_eq!(tasks.tasks.len(), 1);
    assert_eq!(tasks.tasks[0].status, "Review");
}
