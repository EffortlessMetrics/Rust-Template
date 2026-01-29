//! PostgreSQL database adapter using SQLx.
//!
//! This crate implements the TaskRepository port from business-core using
//! SQLx and PostgreSQL, providing async task persistence with UUID-based
//! IDs and transaction support.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::migrate::Migrator;
use sqlx::{PgPool, Row};
use std::env;
use tracing::info;
use uuid::Uuid;

use business_core::ports::{ExampleTaskRepository, RepositoryError};
use model::{ExampleTask, ExampleTaskStatus};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    MIGRATOR.run(pool).await?;
    info!("Database migrations complete");
    Ok(())
}

pub async fn create_pool() -> Result<PgPool> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    pub async fn new() -> Result<Self> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let pool = PgPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    pub fn new_with_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ExampleTaskRepository for PostgresTaskRepository {
    async fn save(&self, task: &ExampleTask) -> Result<(), RepositoryError> {
        let id = Uuid::parse_str(&task.id).map_err(|e| RepositoryError::Other(e.to_string()))?;
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
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        info!(task_id = %task.id, "Saved task");
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, RepositoryError> {
        let uuid = Uuid::parse_str(id).map_err(|e| RepositoryError::Other(e.to_string()))?;
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
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

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

    async fn find_all(&self) -> Result<Vec<ExampleTask>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, status, created_at
            FROM tasks
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

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
    ) -> Result<Option<ExampleTask>, RepositoryError> {
        let uuid = Uuid::parse_str(id).map_err(|e| RepositoryError::Other(e.to_string()))?;
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
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        info!(task_id = %id, status = %status_str, "Updated task status");

        // Return the updated task
        self.find_by_id(id).await
    }
}
