//! Regression tests for async I/O correctness (Issue #14).
//!
//! These tests verify that blocking file I/O operations (fs2 locks, std::fs)
//! are properly offloaded via `spawn_blocking` and don't block the Tokio
//! executor's worker threads.
//!
//! The key insight is: with a single-thread runtime, if any blocking I/O
//! happens on the executor thread, concurrent timer tasks will be starved.
//! This test exploits that behavior to detect regressions.

use adapters_spec_fs::tasks_state;
use app_http::app_with_workspace_root;
use axum::{body::Body, http::Request};
use business_core::governance::{TaskId, TaskStatus};
use spec_runtime::tasks::{Task, TaskDocs, TasksSpec};
use std::{fs, sync::Arc, time::Duration};
use tempfile::tempdir;
use tower::ServiceExt;

/// Helper to set up a minimal test workspace with task files.
fn setup_test_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Seed tasks_state.yaml
    let state_path = specs_dir.join("tasks_state.yaml");
    tasks_state::update_task_status(
        &state_path,
        TaskId("TASK-ASYNC-001".to_string()),
        TaskStatus::Todo,
    )
    .expect("failed to write tasks_state.yaml");

    // Seed tasks.yaml
    let tasks_yaml = specs_dir.join("tasks.yaml");
    let tasks = TasksSpec {
        schema_version: "1.0.0".to_string(),
        template_version: "0.1.0".to_string(),
        tasks: vec![Task {
            id: "TASK-ASYNC-001".to_string(),
            title: "Async Test Task".to_string(),
            requirement: "REQ-TPL-ASYNC".to_string(),
            acs: vec![],
            status: "Todo".to_string(),
            owner: None,
            labels: vec![],
            docs: Some(TaskDocs { design: vec![], plan: vec![] }),
            summary: "Test task for async I/O regression".to_string(),
            recommended_flows: vec![],
            depends_on: vec![],
        }],
    };
    let content = serde_yaml::to_string(&tasks).expect("failed to serialize tasks.yaml");
    fs::write(tasks_yaml, content).expect("failed to write tasks.yaml");

    (temp, spec_root)
}

/// Regression test: concurrent task operations don't block the executor.
///
/// This test uses a single-thread runtime to make executor starvation obvious.
/// If blocking I/O occurs on the executor thread, the timer task will be starved
/// and the test will timeout.
///
/// The test structure:
/// 1. Spawn N concurrent update_task_status requests
/// 2. Concurrently run a timer that sleeps in small intervals
/// 3. Assert the timer completes within a reasonable time window
///
/// If spawn_blocking is removed, this test should hang or timeout.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn concurrent_task_updates_do_not_block_executor() {
    let (_temp, spec_root) = setup_test_workspace();

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone()).expect("valid config");

    // We need a shared service to handle multiple requests
    let app = Arc::new(tokio::sync::Mutex::new(app));

    // Number of concurrent operations
    const CONCURRENT_OPS: usize = 5;

    // Timer task: should complete quickly if executor isn't blocked
    let timer_task = tokio::spawn(async {
        let start = std::time::Instant::now();
        // Sleep in small intervals - this should complete quickly if executor is free
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        start.elapsed()
    });

    // Spawn concurrent task update requests
    // These all try to toggle the same task status back and forth
    let mut handles = Vec::new();
    for i in 0..CONCURRENT_OPS {
        let _app = Arc::clone(&app); // kept for future use if we want shared service
        let spec_root = spec_root.clone();
        handles.push(tokio::spawn(async move {
            // Rebuild app for each request (tower oneshot consumes)
            let repo =
                Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
            let app = app_with_workspace_root(repo, spec_root).expect("valid config");

            // Alternate between InProgress and Todo to exercise write locks
            let status = if i % 2 == 0 { "InProgress" } else { "Todo" };
            let body = format!(r#"{{ "status": "{}" }}"#, status);
            let request = Request::builder()
                .method("POST")
                .uri("/platform/tasks/TASK-ASYNC-001/status")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("failed to build request");

            let response = app.oneshot(request).await;
            // We expect some requests to fail due to invalid transitions, that's OK
            // The point is they complete without blocking the executor
            response.is_ok()
        }));
    }

    // Wait for all task updates with a timeout
    let results = tokio::time::timeout(Duration::from_secs(10), async {
        let mut completed = 0;
        for handle in handles {
            if handle.await.is_ok() {
                completed += 1;
            }
        }
        completed
    })
    .await
    .expect("task updates timed out - possible executor starvation");

    // Wait for timer task
    let timer_elapsed = tokio::time::timeout(Duration::from_secs(5), timer_task)
        .await
        .expect("timer timed out - executor was blocked")
        .expect("timer task panicked");

    // Timer should have completed in roughly 100ms (10 * 10ms sleeps)
    // Allow generous slack for CI variability, but flag if it took way too long
    // (which would indicate the executor was blocked)
    assert!(
        timer_elapsed < Duration::from_secs(2),
        "Timer took {:?} - expected ~100ms. Executor may have been blocked by sync I/O.",
        timer_elapsed
    );

    // At least some operations should have completed
    assert!(results > 0, "No task operations completed");
}

/// Regression test: concurrent read operations don't block the executor.
///
/// Similar to the write test, but exercises the read path (list_tasks).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn concurrent_task_reads_do_not_block_executor() {
    let (_temp, spec_root) = setup_test_workspace();

    // Timer task
    let timer_task = tokio::spawn(async {
        let start = std::time::Instant::now();
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        start.elapsed()
    });

    // Spawn concurrent read requests
    let mut handles = Vec::new();
    for _ in 0..5 {
        let spec_root = spec_root.clone();
        handles.push(tokio::spawn(async move {
            let repo =
                Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
            let app = app_with_workspace_root(repo, spec_root).expect("valid config");

            let request = Request::builder()
                .method("GET")
                .uri("/platform/tasks")
                .body(Body::empty())
                .expect("failed to build request");

            app.oneshot(request).await.is_ok()
        }));
    }

    // Wait for all reads
    let results = tokio::time::timeout(Duration::from_secs(10), async {
        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }
        completed
    })
    .await
    .expect("read operations timed out");

    // Wait for timer
    let timer_elapsed = tokio::time::timeout(Duration::from_secs(5), timer_task)
        .await
        .expect("timer timed out")
        .expect("timer task panicked");

    assert!(
        timer_elapsed < Duration::from_secs(2),
        "Timer took {:?} - expected ~100ms. Executor may have been blocked.",
        timer_elapsed
    );

    assert!(results > 0, "No read operations completed");
}

fn setup_snapshot_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Seed spec_ledger.yaml
    let ledger_path = specs_dir.join("spec_ledger.yaml");
    let ledger_content = r#"
metadata:
  template_version: "1.0.0"
stories: []
"#;
    fs::write(ledger_path, ledger_content).expect("failed to write spec_ledger.yaml");

    // Seed service_metadata.yaml
    let meta_path = specs_dir.join("service_metadata.yaml");
    let meta_content = r#"
service_id: "test-service"
display_name: "Test Service"
"#;
    fs::write(meta_path, meta_content).expect("failed to write service_metadata.yaml");

    (temp, spec_root)
}

/// Regression test: concurrent IDP snapshot reads don't block the executor.
///
/// This tests the /platform/idp/snapshot endpoint specifically, which performs
/// multiple file I/O operations (ledger, coverage, tasks, etc.).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn concurrent_idp_snapshot_reads_do_not_block_executor() {
    let (_temp, spec_root) = setup_snapshot_workspace();

    // Timer task
    let timer_task = tokio::spawn(async {
        let start = std::time::Instant::now();
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        start.elapsed()
    });

    // Spawn concurrent read requests
    let mut handles = Vec::new();
    for _ in 0..5 {
        let spec_root = spec_root.clone();
        handles.push(tokio::spawn(async move {
            let repo =
                Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
            let app = app_with_workspace_root(repo, spec_root).expect("valid config");

            let request = Request::builder()
                .method("GET")
                .uri("/platform/idp/snapshot")
                .body(Body::empty())
                .expect("failed to build request");

            app.oneshot(request).await.is_ok()
        }));
    }

    // Wait for all reads
    let results = tokio::time::timeout(Duration::from_secs(10), async {
        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }
        completed
    })
    .await
    .expect("snapshot read operations timed out");

    // Wait for timer
    let timer_elapsed = tokio::time::timeout(Duration::from_secs(5), timer_task)
        .await
        .expect("timer timed out")
        .expect("timer task panicked");

    assert!(
        timer_elapsed < Duration::from_secs(2),
        "Timer took {:?} - expected ~100ms. Executor may have been blocked.",
        timer_elapsed
    );

    assert!(results > 0, "No snapshot operations completed");
}
