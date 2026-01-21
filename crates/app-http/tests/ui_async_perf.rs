//! Performance regression tests for UI endpoints.
//!
//! These tests verify that UI endpoints (Dashboard, Graph, Flows) do not block
//! the async executor with synchronous file I/O.

use app_http::app_with_workspace_root;
use axum::{body::Body, http::Request};
use std::{fs, sync::Arc, time::Duration};
use tempfile::tempdir;
use tower::ServiceExt;

/// Helper to set up a minimal test workspace
fn setup_test_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Create minimal valid spec files to avoid errors
    fs::write(
        specs_dir.join("spec_ledger.yaml"),
        "metadata:\n  template_version: 1.0.0\nstories: []",
    ).unwrap();
    fs::write(
        specs_dir.join("devex_flows.yaml"),
        "flows: {}",
    ).unwrap();
    fs::write(
        specs_dir.join("doc_index.yaml"),
        "docs: []",
    ).unwrap();
    fs::write(
        specs_dir.join("tasks.yaml"),
        "tasks: []",
    ).unwrap();
     fs::write(
        specs_dir.join("service_metadata.yaml"),
        "service_id: test-service",
    ).unwrap();

    (temp, spec_root)
}

/// Verify that the Dashboard endpoint doesn't block the executor.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn dashboard_does_not_block_executor() {
    let (_temp, spec_root) = setup_test_workspace();

    // Timer task to detect starvation
    let timer_task = tokio::spawn(async {
        let start = std::time::Instant::now();
        // Sleep in small intervals
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        start.elapsed()
    });

    // Spawn concurrent requests to the dashboard
    let mut handles = Vec::new();
    for _ in 0..10 {
        let spec_root = spec_root.clone();
        handles.push(tokio::spawn(async move {
            let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
            let app = app_with_workspace_root(repo, spec_root).expect("valid config");

            let request = Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .expect("failed to build request");

            let response = app.oneshot(request).await.expect("request failed");
            assert_eq!(response.status(), 200);
        }));
    }

    // Wait for requests
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        for handle in handles {
            handle.await.unwrap();
        }
    })
    .await
    .expect("dashboard requests timed out");

    // Check timer
    let timer_elapsed = tokio::time::timeout(Duration::from_secs(2), timer_task)
        .await
        .expect("timer timed out")
        .expect("timer panicked");

    // If blocking I/O is happening on the main thread, the timer will be delayed significantly.
    // With 10 requests doing file I/O on the single worker thread, the 100ms timer
    // will likely take much longer if they block.
    // We use a strict threshold to catch blocking behavior.
    assert!(
        timer_elapsed < Duration::from_millis(500),
        "Timer took {:?} - expected ~100ms. Executor blocked by synchronous I/O in dashboard?",
        timer_elapsed
    );
}
