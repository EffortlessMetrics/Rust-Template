use adapters_spec_fs::tasks_state;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use business_core::governance::{TaskId, TaskStatus};
use spec_runtime::tasks::{Task, TaskDocs, TasksSpec};
use std::{fs, sync::Arc};
use tempfile::tempdir;
use testing::process::EnvVarGuard;
use tower::ServiceExt;

const AUTH_ENV_VARS: &[&str] =
    &["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN", "PLATFORM_JWT_SECRET"];

fn write_tasks_files(spec_root: &std::path::Path, task_id: &str, status: TaskStatus) {
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    let state_path = specs_dir.join("tasks_state.yaml");
    tasks_state::update_task_status(&state_path, TaskId(task_id.to_string()), status)
        .expect("failed to write tasks_state.yaml");

    let tasks_yaml = specs_dir.join("tasks.yaml");
    let tasks = TasksSpec {
        schema_version: "1.0.0".to_string(),
        template_version: "0.1.0".to_string(),
        tasks: vec![Task {
            id: task_id.to_string(),
            title: "Auth Test Task".to_string(),
            requirement: "REQ-TPL-AUTH".to_string(),
            acs: vec![],
            status: "Todo".to_string(),
            owner: None,
            labels: vec![],
            docs: Some(TaskDocs { design: vec![], plan: vec![] }),
            summary: "Auth test task summary".to_string(),
            recommended_flows: vec![],
            depends_on: vec![],
        }],
    };

    let content = serde_yaml::to_string(&tasks).expect("failed to serialize tasks.yaml");
    fs::write(tasks_yaml, content).expect("failed to write tasks.yaml");
}

fn auth_env_guard() -> EnvVarGuard {
    let guard = EnvVarGuard::new(AUTH_ENV_VARS);
    for key in AUTH_ENV_VARS {
        guard.remove(key);
    }
    guard
}

#[tokio::test]
async fn authorization_bearer_header_takes_precedence_over_legacy_platform_token() {
    let env_guard = auth_env_guard();
    env_guard.set("PLATFORM_AUTH_MODE", "basic");
    env_guard.set("PLATFORM_AUTH_TOKEN", "legacy-token");

    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-AUTH-IT-001", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-AUTH-IT-001/status")
        .header("content-type", "application/json")
        .header("authorization", "Bearer invalid.jwt.token")
        .header("x-platform-token", "legacy-token")
        .body(Body::from(r#"{"status":"InProgress"}"#))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-AUTH-IT-001".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::Todo));
}

#[tokio::test]
async fn falls_back_to_legacy_platform_token_when_authorization_scheme_is_not_bearer() {
    let env_guard = auth_env_guard();
    env_guard.set("PLATFORM_AUTH_MODE", "basic");
    env_guard.set("PLATFORM_AUTH_TOKEN", "legacy-token");

    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-AUTH-IT-002", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-AUTH-IT-002/status")
        .header("content-type", "application/json")
        .header("authorization", "Basic dXNlcjpwYXNz")
        .header("x-platform-token", "legacy-token")
        .body(Body::from(r#"{"status":"InProgress"}"#))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-AUTH-IT-002".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}
