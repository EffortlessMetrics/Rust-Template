use adapters_grpc::spawn;
use adapters_grpc::task::v1::task_service_client::TaskServiceClient;
use adapters_grpc::task::v1::{CreateTaskRequest, GetTaskRequest, ListTasksRequest};
use async_trait::async_trait;
use business_core::ports::{ExampleTaskRepository, RepositoryError};
use model::{ExampleTask, ExampleTaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory example task repository for testing.
///
/// Uses `tokio::sync::RwLock` instead of `std::sync::Mutex` because:
/// - Async-aware: lock acquisition yields to the executor instead of blocking
/// - Read-heavy workloads benefit from concurrent read access
/// - Proper pattern for async trait implementations
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
    async fn save(&self, task: &ExampleTask) -> Result<(), RepositoryError> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, RepositoryError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<ExampleTask>, RepositoryError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn update_status(
        &self,
        id: &str,
        status: ExampleTaskStatus,
    ) -> Result<Option<ExampleTask>, RepositoryError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = status;
            Ok(Some(task.clone()))
        } else {
            Ok(None)
        }
    }
}

/// gRPC adapter smoke test
///
/// This test verifies the gRPC service works end-to-end with an in-memory repository.
/// It's marked with #[ignore] by default to keep CI fast.
///
/// Run with: cargo test -p adapters-grpc --test smoke -- --ignored
#[tokio::test]
#[ignore = "gRPC smoke test; run with cargo test -- --ignored"]
async fn test_grpc_service_create_task() {
    // Create in-memory repository
    let repo: Arc<dyn ExampleTaskRepository> = Arc::new(InMemoryExampleTaskRepository::new());

    // Start gRPC server on random port
    // Start gRPC server on random port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release port so server can bind to it
    let server_repo = repo.clone();

    // Spawn server in background task
    let server_handle = tokio::spawn(async move { spawn(port, server_repo).await });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Create gRPC client
    let addr = format!("http://[::1]:{}", port);
    let mut client =
        TaskServiceClient::connect(addr).await.expect("Failed to connect to gRPC server");

    // Test 1: Create a task
    let create_request = CreateTaskRequest { title: "gRPC smoke test task".to_string() };

    let create_response =
        client.create_task(create_request).await.expect("Failed to create task").into_inner();

    let created_task = create_response.task.expect("No task in create response");
    assert_eq!(created_task.title, "gRPC smoke test task");
    assert_eq!(created_task.status, "Pending");

    let task_id = created_task.id.clone();

    // Test 2: Get the task
    let get_request = GetTaskRequest { id: task_id.clone() };

    let get_response = client.get_task(get_request).await.expect("Failed to get task").into_inner();
    let fetched_task = get_response.task.expect("No task in get response");

    assert_eq!(fetched_task.id, task_id);
    assert_eq!(fetched_task.title, "gRPC smoke test task");

    // Test 3: List tasks
    let list_request = ListTasksRequest {};

    let list_response =
        client.list_tasks(list_request).await.expect("Failed to list tasks").into_inner();

    assert_eq!(list_response.tasks.len(), 1);
    assert_eq!(list_response.tasks[0].id, task_id);

    // Cleanup: abort server
    server_handle.abort();

    println!("✓ gRPC adapter smoke test passed");
}
