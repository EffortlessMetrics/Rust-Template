use adapters_db_sqlx::PostgresTaskRepository;
use business_core::ports::ExampleTaskRepository;
use model::{ExampleTask, ExampleTaskStatus};
use sqlx::PgPool;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

/// Integration test for PostgresTaskRepository
///
/// This test requires Docker to be running to spin up a PostgreSQL container.
/// It's marked with #[ignore] by default to keep CI fast.
///
/// Run with: cargo test -p adapters-db-sqlx -- --ignored
#[tokio::test]
#[ignore = "Requires Docker for testcontainers; run with cargo test -- --ignored"]
async fn test_postgres_repository_roundtrip() {
    // Start PostgreSQL container with proper wait strategy (wait for ready message instead of sleep)
    let postgres_image = GenericImage::new("postgres", "16-alpine")
        .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
        .with_env_var("POSTGRES_DB", "test_db")
        .with_env_var("POSTGRES_USER", "test_user")
        .with_env_var("POSTGRES_PASSWORD", "test_pass");

    let container = postgres_image.start().await.expect("Failed to start container");
    let port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");

    // Build connection string
    let database_url = format!("postgres://test_user:test_pass@localhost:{}/test_db", port);

    // Connect to database
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to test database");

    // Run migrations (create tasks table)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            status VARCHAR(20) NOT NULL CHECK (status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED')),
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create tasks table");

    // Create repository with the pool
    let repo = PostgresTaskRepository::new_with_pool(pool.clone());

    // Test 1: Create a task (manually, as we're testing the repository layer directly)
    let task_title = "Integration test task".to_string();
    let task = ExampleTask {
        id: uuid::Uuid::new_v4().to_string(),
        title: task_title.clone(),
        status: ExampleTaskStatus::Pending,
        created_at: chrono::Utc::now(),
    };
    let task_id = task.id.clone();

    repo.save(&task).await.expect("Failed to save task");

    // Test 2: Find task by ID
    let fetched_task =
        repo.find_by_id(&task_id).await.expect("Failed to find task").expect("Task not found");

    assert_eq!(fetched_task.id, task_id);
    assert_eq!(fetched_task.title, task_title);
    assert_eq!(fetched_task.status, ExampleTaskStatus::Pending);

    // Test 3: Update task status
    let updated_task = repo
        .update_status(&task_id, ExampleTaskStatus::InProgress)
        .await
        .expect("Failed to update task status")
        .expect("Updated task not found");

    assert_eq!(updated_task.status, ExampleTaskStatus::InProgress);

    // Test 4: Find all tasks
    let all_tasks = repo.find_all().await.expect("Failed to find all tasks");

    assert_eq!(all_tasks.len(), 1);
    assert_eq!(all_tasks[0].id, task_id);
    assert_eq!(all_tasks[0].status, ExampleTaskStatus::InProgress);

    // Test 5: Update to completed
    let completed_task = repo
        .update_status(&task_id, ExampleTaskStatus::Completed)
        .await
        .expect("Failed to update to completed")
        .expect("Completed task not found");

    assert_eq!(completed_task.status, ExampleTaskStatus::Completed);

    // Cleanup: close pool
    pool.close().await;

    println!("✓ All PostgresTaskRepository integration tests passed");
}
