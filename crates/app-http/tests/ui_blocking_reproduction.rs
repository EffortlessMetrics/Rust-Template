use app_http::app_with_workspace_root;
use axum::{body::Body, http::Request};
use std::{fs, sync::Arc, time::Duration};
use tempfile::tempdir;
use tower::ServiceExt;

fn setup_full_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Create minimal valid spec files to avoid errors

    // spec_ledger.yaml
    fs::write(
        specs_dir.join("spec_ledger.yaml"),
        r#"
metadata:
  schema_version: "1.0.0"
  template_version: "1.0.0"
  last_updated: "2024-01-01T00:00:00Z"
  description: "Test Ledger"
stories: []
"#,
    )
    .unwrap();

    // devex_flows.yaml
    fs::write(
        specs_dir.join("devex_flows.yaml"),
        r#"
flows: {}
"#,
    )
    .unwrap();

    // doc_index.yaml
    fs::write(
        specs_dir.join("doc_index.yaml"),
        r#"
docs: []
"#,
    )
    .unwrap();

    // tasks.yaml
    fs::write(
        specs_dir.join("tasks.yaml"),
        r#"
schema_version: "1.0.0"
template_version: "1.0.0"
tasks: []
"#,
    )
    .unwrap();

    // service_metadata.yaml
    fs::write(
        specs_dir.join("service_metadata.yaml"),
        r#"
name: "test-service"
description: "Test Service"
owner: "test"
"#,
    )
    .unwrap();

    (temp, spec_root)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn dashboard_does_not_block_executor() {
    let (_temp, spec_root) = setup_full_workspace();

    // Timer task
    let timer_task = tokio::spawn(async {
        let start = std::time::Instant::now();
        // Sleep 10 times 10ms. If executor is blocked, this will take longer.
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        start.elapsed()
    });

    // Spawn concurrent dashboard requests
    let mut handles = Vec::new();
    for _ in 0..10 {
        // Increase count to ensure enough work
        let spec_root = spec_root.clone();
        handles.push(tokio::spawn(async move {
            let repo =
                Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
            let app = app_with_workspace_root(repo, spec_root).expect("valid config");

            let request = Request::builder()
                .method("GET")
                .uri("/") // Dashboard
                .body(Body::empty())
                .expect("failed to build request");

            app.oneshot(request).await
        }));
    }

    // Wait for requests
    tokio::time::timeout(Duration::from_secs(10), async {
        for handle in handles {
            let _ = handle.await;
        }
    })
    .await
    .expect("requests timed out");

    // Check timer
    let timer_elapsed = tokio::time::timeout(Duration::from_secs(5), timer_task)
        .await
        .expect("timer timed out")
        .expect("timer panicked");

    // If blocking I/O happens on main thread, timer will be delayed significantly.
    // With 1 thread, every FS call blocks the thread.

    println!("Timer elapsed: {:?}", timer_elapsed);

    assert!(
        timer_elapsed < Duration::from_millis(500),
        "Timer took {:?} - expected < 500ms. Executor was blocked by sync I/O.",
        timer_elapsed
    );
}
