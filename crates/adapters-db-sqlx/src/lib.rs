//! PostgreSQL database adapter using SQLx.
//!
//! This crate implements the TaskRepository port from business-core using
//! SQLx and PostgreSQL, providing async task persistence with UUID-based
//! IDs and transaction support.
//!
//! # Migrations
//!
//! Database migrations are embedded at compile time from the `migrations/`
//! directory. Use [`run_migrations`] to apply pending migrations.
//!
//! ```ignore
//! let pool = PgPool::connect(&database_url).await?;
//! adapters_db_sqlx::run_migrations(&pool).await?;
//! ```

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::migrate::Migrator;
use sqlx::{PgPool, Row};
use std::env;
use tracing::info;
use uuid::Uuid;

use business_core::ports::ExampleTaskRepository;
use model::{ExampleTask, ExampleTaskStatus};

/// Embedded migrations from the migrations/ directory.
///
/// Migrations are compiled into the binary at build time, ensuring
/// the application always has access to its schema definitions.
static MIGRATOR: Migrator = sqlx::migrate!();

/// Run all pending database migrations.
///
/// This function applies any unapplied migrations from the embedded
/// migration set. It's idempotent - already-applied migrations are skipped.
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool to run migrations on
///
/// # Errors
///
/// Returns an error if any migration fails to apply.
///
/// # Example
///
/// ```ignore
/// let pool = PgPool::connect(&database_url).await?;
/// run_migrations(&pool).await?;
/// ```
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    MIGRATOR.run(pool).await?;
    info!("Database migrations complete");
    Ok(())
}

/// Get the pool for running migrations (creates a new connection).
///
/// Useful when you need to run migrations without an existing repository.
pub async fn create_pool() -> Result<PgPool> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

/// PostgreSQL-backed implementation of the [`ExampleTaskRepository`] trait.
///
/// This struct provides persistent storage for [`ExampleTask`] entities using
/// PostgreSQL as the backing database. It uses SQLx for async database operations
/// and connection pooling.
///
/// # Architecture
///
/// `PostgresTaskRepository` implements the repository pattern, serving as the
/// concrete adapter for the `ExampleTaskRepository` port defined in `business-core`.
/// This separation allows the business logic to remain database-agnostic while
/// this adapter handles PostgreSQL-specific concerns.
///
/// # Connection Management
///
/// The repository uses a [`PgPool`] for connection pooling, which is managed
/// internally. Connections are acquired from the pool for each operation and
/// returned automatically when the operation completes.
///
/// # Example
///
/// ```ignore
/// use adapters_db_sqlx::PostgresTaskRepository;
///
/// // Create repository from DATABASE_URL environment variable
/// let repo = PostgresTaskRepository::new().await?;
///
/// // Or use an existing connection pool
/// let pool = PgPool::connect(&database_url).await?;
/// let repo = PostgresTaskRepository::new_with_pool(pool);
/// ```
///
/// # Database Schema
///
/// This repository expects a `tasks` table with the following columns:
/// - `id` (UUID, primary key)
/// - `title` (TEXT)
/// - `status` (TEXT: "PENDING", "IN_PROGRESS", or "COMPLETED")
/// - `created_at` (TIMESTAMP WITH TIME ZONE)
///
/// Use [`run_migrations`] to ensure the schema is up to date.
pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    /// Creates a new repository instance using the `DATABASE_URL` environment variable.
    ///
    /// This method reads the database connection string from the `DATABASE_URL`
    /// environment variable and establishes a connection pool to the PostgreSQL
    /// database.
    ///
    /// # Environment Variables
    ///
    /// - `DATABASE_URL` - Required. PostgreSQL connection string in the format:
    ///   `postgres://user:password@host:port/database`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `DATABASE_URL` environment variable is not set
    /// - The connection to the database fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Ensure DATABASE_URL is set before calling
    /// std::env::set_var("DATABASE_URL", "postgres://localhost/mydb");
    /// let repo = PostgresTaskRepository::new().await?;
    /// ```
    pub async fn new() -> Result<Self> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let pool = PgPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    /// Creates a new repository instance with an existing connection pool.
    ///
    /// This constructor is useful when you want to share a connection pool
    /// across multiple repositories or services, or when you need more control
    /// over pool configuration (e.g., in tests or when using custom pool settings).
    ///
    /// # Arguments
    ///
    /// * `pool` - An existing [`PgPool`] connection pool
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sqlx::postgres::PgPoolOptions;
    ///
    /// let pool = PgPoolOptions::new()
    ///     .max_connections(5)
    ///     .connect(&database_url)
    ///     .await?;
    ///
    /// let repo = PostgresTaskRepository::new_with_pool(pool);
    /// ```
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ExampleTaskRepository for PostgresTaskRepository {
    async fn save(&self, task: &ExampleTask) -> Result<(), String> {
        let id = Uuid::parse_str(&task.id).map_err(|e| e.to_string())?;
        let created_at = task.created_at;
        let status_str = match task.status {
            ExampleTaskStatus::Pending => "PENDING",
            ExampleTaskStatus::InProgress => "IN_PROGRESS",
            ExampleTaskStatus::Completed => "COMPLETED",
        };

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, status, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(&task.title)
        .bind(status_str)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        info!(task_id = %task.id, "Saved task");
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, String> {
        let uuid = Uuid::parse_str(id).map_err(|e| e.to_string())?;
        let row = sqlx::query(
            r#"
            SELECT id, title, status, created_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(if let Some(row) = row {
            let id: Uuid = row.get("id");
            let title: String = row.get("title");
            let status_str: String = row.get("status");
            let created_at: DateTime<Utc> = row.get("created_at");
            let status = match status_str.as_str() {
                "PENDING" => ExampleTaskStatus::Pending,
                "IN_PROGRESS" => ExampleTaskStatus::InProgress,
                "COMPLETED" => ExampleTaskStatus::Completed,
                _ => ExampleTaskStatus::Pending,
            };

            Some(ExampleTask { id: id.to_string(), title, status, created_at })
        } else {
            None
        })
    }

    async fn find_all(&self) -> Result<Vec<ExampleTask>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, status, created_at
            FROM tasks
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let tasks = rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let title: String = row.get("title");
                let status_str: String = row.get("status");
                let created_at: DateTime<Utc> = row.get("created_at");
                let status = match status_str.as_str() {
                    "PENDING" => ExampleTaskStatus::Pending,
                    "IN_PROGRESS" => ExampleTaskStatus::InProgress,
                    "COMPLETED" => ExampleTaskStatus::Completed,
                    _ => ExampleTaskStatus::Pending,
                };
                ExampleTask { id: id.to_string(), title, status, created_at }
            })
            .collect();

        Ok(tasks)
    }

    async fn update_status(
        &self,
        id: &str,
        status: ExampleTaskStatus,
    ) -> Result<Option<ExampleTask>, String> {
        let uuid = Uuid::parse_str(id).map_err(|e| e.to_string())?;
        let status_str = match status {
            ExampleTaskStatus::Pending => "PENDING",
            ExampleTaskStatus::InProgress => "IN_PROGRESS",
            ExampleTaskStatus::Completed => "COMPLETED",
        };

        sqlx::query(
            r#"
            UPDATE tasks
            SET status = $1
            WHERE id = $2
            "#,
        )
        .bind(status_str)
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        info!(task_id = %id, status = %status_str, "Updated task status");

        // Return the updated task
        self.find_by_id(id).await
    }
}
