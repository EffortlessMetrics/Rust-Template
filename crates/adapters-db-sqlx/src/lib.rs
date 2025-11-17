use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::env;
use tracing::{error, info};
use uuid::Uuid;

use model::{Task, TaskStatus};
use business_core::ports::TaskRepository;

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
}

#[async_trait::async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn create_task(&self, title: String) -> Result<Task> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, status, created_at) 
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(&title)
        .bind("PENDING")
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        let task = Task { id: id.to_string(), title, status: TaskStatus::Pending };

        info!(task_id = %task.id, "Created task");
        Ok(task)
    }

    async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, status, created_at 
            FROM tasks 
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(if let Some(row) = row {
            let id: String = row.get("id");
            let title: String = row.get("title");
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "PENDING" => TaskStatus::Pending,
                "COMPLETED" => TaskStatus::Completed,
                _ => TaskStatus::Pending,
            };

            Some(Task { id, title, status })
        } else {
            None
        })
    }

    async fn list_tasks(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, title, status, created_at 
            FROM tasks 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let tasks = rows
            .into_iter()
            .map(|row| Task { id: row.id, title: row.title, status: row.status })
            .collect();

        Ok(tasks)
    }

    async fn update_status(&self, id: &str, status: TaskStatus) -> Result<()> {
        let status_str = match status {
            TaskStatus::Pending => "PENDING",
            TaskStatus::Completed => "COMPLETED",
        };

        sqlx::query(
            r#"
            UPDATE tasks 
            SET status = $1 
            WHERE id = $2
            "#,
        )
        .bind(status_str)
        .bind(id)
        .execute(&self.pool)
        .await?;

        info!(task_id = %id, status = %status_str, "Updated task status");
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    title: String,
    status: TaskStatus,
    created_at: DateTime<Utc>,
}
