//! Concurrency tests for the in-memory repository implementation.
//!
//! These tests verify that the async-aware `tokio::sync::RwLock` implementation
//! handles concurrent operations correctly without deadlocks or data races.
//!
//! Run with: `cargo test -p adapters-grpc --test concurrency`

use async_trait::async_trait;
use business_core::ports::ExampleTaskRepository;
use chrono::Utc;
use model::{ExampleTask, ExampleTaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory example task repository for concurrency testing.
///
/// Uses `tokio::sync::RwLock` for async-aware locking:
/// - Lock acquisition yields to the executor instead of blocking the thread
/// - Concurrent reads are allowed (read-heavy workloads benefit)
/// - Writes are exclusive but don't block the executor while waiting
struct InMemoryExampleTaskRepository {
    tasks: RwLock<HashMap<String, ExampleTask>>,
}

impl InMemoryExampleTaskRepository {
    fn new() -> Self {
        Self { tasks: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl ExampleTaskRepository for InMemoryExampleTaskRepository {
    async fn save(&self, task: &ExampleTask) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, String> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<ExampleTask>, String> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn update_status(
        &self,
        id: &str,
        status: ExampleTaskStatus,
    ) -> Result<Option<ExampleTask>, String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = status;
            Ok(Some(task.clone()))
        } else {
            Ok(None)
        }
    }
}

/// Test concurrent write operations.
///
/// Spawns 20 concurrent tasks that each save a unique task to the repository,
/// then verifies all tasks were persisted correctly.
#[tokio::test]
async fn test_concurrent_task_operations() {
    let repo: Arc<dyn ExampleTaskRepository> = Arc::new(InMemoryExampleTaskRepository::new());
    let task_count = 20;

    // Spawn concurrent write operations
    let handles: Vec<_> = (0..task_count)
        .map(|i| {
            let repo = repo.clone();
            tokio::spawn(async move {
                let task = ExampleTask {
                    id: format!("task-{}", i),
                    title: format!("Concurrent Task {}", i),
                    status: ExampleTaskStatus::Pending,
                    created_at: Utc::now(),
                };
                repo.save(&task).await.expect("save should succeed");
            })
        })
        .collect();

    // Wait for all writes to complete
    for handle in handles {
        handle.await.expect("task should not panic");
    }

    // Verify all tasks were saved
    let all_tasks = repo.find_all().await.expect("find_all should succeed");
    assert_eq!(all_tasks.len(), task_count, "all {} tasks should be persisted", task_count);

    // Verify each task can be retrieved individually
    for i in 0..task_count {
        let task =
            repo.find_by_id(&format!("task-{}", i)).await.expect("find_by_id should succeed");
        assert!(task.is_some(), "task-{} should exist", i);
        assert_eq!(task.unwrap().title, format!("Concurrent Task {}", i));
    }
}

/// Test concurrent read and write operations.
///
/// Spawns a mix of concurrent readers and writers to verify:
/// - No deadlocks occur under contention
/// - No panics from concurrent access
/// - Data remains consistent (writes are visible to subsequent reads)
#[tokio::test]
async fn test_concurrent_read_write() {
    let repo: Arc<dyn ExampleTaskRepository> = Arc::new(InMemoryExampleTaskRepository::new());
    let operation_count: usize = 50;

    // Pre-populate some tasks
    for i in 0..10 {
        let task = ExampleTask {
            id: format!("initial-{}", i),
            title: format!("Initial Task {}", i),
            status: ExampleTaskStatus::Pending,
            created_at: Utc::now(),
        };
        repo.save(&task).await.expect("initial save should succeed");
    }

    // Spawn mixed read/write operations
    let handles: Vec<_> = (0..operation_count)
        .map(|i| {
            let repo = repo.clone();
            tokio::spawn(async move {
                match i % 4 {
                    // Write: save new task
                    0 => {
                        let task = ExampleTask {
                            id: format!("concurrent-{}", i),
                            title: format!("Concurrent Task {}", i),
                            status: ExampleTaskStatus::Pending,
                            created_at: Utc::now(),
                        };
                        repo.save(&task).await.expect("save should succeed");
                    }
                    // Read: find_all
                    1 => {
                        let _ = repo.find_all().await.expect("find_all should succeed");
                    }
                    // Read: find_by_id (existing)
                    2 => {
                        let _ = repo
                            .find_by_id(&format!("initial-{}", i % 10))
                            .await
                            .expect("find_by_id should succeed");
                    }
                    // Write: update_status
                    _ => {
                        let _ = repo
                            .update_status(
                                &format!("initial-{}", i % 10),
                                ExampleTaskStatus::InProgress,
                            )
                            .await
                            .expect("update_status should succeed");
                    }
                }
            })
        })
        .collect();

    // Wait for all operations to complete (no deadlock = success)
    for handle in handles {
        handle.await.expect("operation should not panic");
    }

    // Verify repository is still in consistent state
    let all_tasks = repo.find_all().await.expect("final find_all should succeed");

    // Should have initial 10 + count of operations where i % 4 == 0
    // For 0..50, i % 4 == 0 when i is: 0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48 = 13 values
    let expected_new_tasks = operation_count.div_ceil(4); // count of i where i % 4 == 0 for 0..n
    let expected_total = 10 + expected_new_tasks;
    assert_eq!(
        all_tasks.len(),
        expected_total,
        "repository should contain {} tasks (10 initial + {} concurrent writes)",
        expected_total,
        expected_new_tasks
    );
}
