#[cfg(feature = "integration-db")]
mod integration_tests {
    use adapters_db_sqlx::PostgresTaskRepository;
    use model::{Task, TaskStatus};
    use testcontainers::{clients::Cli, images::postgres::Postgres};

    #[tokio::test]
    async fn test_task_repository_crud() -> anyhow::Result<()> {
        let docker = Cli::default();
        let postgres = docker.run(Postgres::default());
        let port = postgres.get_host_port_ipv4(5432);

        let database_url = format!("postgres://postgres:postgres@localhost:{}", port);

        // Run migrations
        let _ = sqlx::migrate!("./migrations").async_run(&database_url.parse()?).await?;

        let repo = PostgresTaskRepository::new_with_url(&database_url).await?;

        // Create
        let task = repo.create_task("Test task".to_string()).await?;
        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);

        // Get
        let fetched = repo.get_task(&task.id).await?;
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, task.id);

        // List
        let tasks = repo.list_tasks().await?;
        assert!(!tasks.is_empty());

        // Update
        repo.update_status(&task.id, TaskStatus::Completed).await?;

        let updated = repo.get_task(&task.id).await?.unwrap();
        assert_eq!(updated.status, TaskStatus::Completed);

        Ok(())
    }
}

// Helper for tests
#[cfg(feature = "integration-db")]
impl PostgresTaskRepository {
    pub async fn new_with_url(database_url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}
