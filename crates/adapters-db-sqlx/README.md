# adapters-db-sqlx

PostgreSQL database adapter using SQLx for persistent task storage.

## What It Is

`adapters-db-sqlx` implements the [`ExampleTaskRepository`] port from `business-core` using SQLx and PostgreSQL. It provides async task persistence with UUID-based IDs, connection pooling, and embedded database migrations.

This crate follows the hexagonal architecture pattern: it implements a repository interface defined in the business logic layer, keeping the domain code database-agnostic.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | Repository implementation, migration runner, connection pool management |
| `migrations/` | Embedded SQL migration files for database schema |

### What It Is Not

- **Not a standalone service**: This is a library crate, not a binary
- **Not business logic**: Domain rules live in `business-core`
- **Not HTTP layer**: HTTP handlers live in `app-http`

## Quick Start

### Setting Up the Database

```bash
# Start PostgreSQL using docker-compose
docker compose up -d postgres

# Run migrations
cargo run -p adapters-db-sqlx --bin run_migrations
```

Or run migrations programmatically:

```rust
use adapters_db_sqlx::{PostgresTaskRepository, run_migrations};
use sqlx::PgPool;

# async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     let pool = PgPool::connect(&database_url).await?;
#     run_migrations(&pool).await?;
#     Ok(())
# }
```

### Using the Repository

```rust
use adapters_db_sqlx::PostgresTaskRepository;
use business_core::use_cases::{create_example_task, list_example_tasks};

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     // Create repository from DATABASE_URL environment variable
#     let repo = PostgresTaskRepository::new().await?;
#
#     // Create a task via business logic
#     let task = create_example_task(&repo, "My first task".to_string()).await?;
#     println!("Created task: {}", task.id);
#
#     // List all tasks
#     let tasks = list_example_tasks(&repo).await?;
#     println!("Total tasks: {}", tasks.len());
#
#     Ok(())
# }
```

### Environment Variables

| Variable | Required | Description | Example |
|----------|-----------|-------------|----------|
| `DATABASE_URL` | Yes | PostgreSQL connection string | `postgres://user:pass@localhost:5432/mydb` |

## Database Schema

The repository expects a `tasks` table with the following structure:

```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

Valid status values: `PENDING`, `IN_PROGRESS`, `COMPLETED`

## Migrations

Migrations are embedded at compile time from the `migrations/` directory using SQLx's `migrate!` macro. This ensures the application always has access to its schema definitions.

To add a new migration:

1. Create a new SQL file in `migrations/` with timestamp prefix:

   ```
   migrations/20250101000000_new_column.sql
   ```

2. The migration will be automatically embedded and applied by [`run_migrations()`]

## Connection Pooling

The repository uses [`PgPool`] for efficient connection management:

- Connections are acquired from the pool for each operation
- Connections are automatically returned when the operation completes
- Pool size can be configured via the connection string

```rust
// Use an existing pool for custom configuration
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

let repo = PostgresTaskRepository::new_with_pool(pool);
```

## Architecture

The crate implements the repository pattern:

```
Business Logic (business-core)
    ↓ depends on
Repository Port (ExampleTaskRepository trait)
    ↓ implemented by
PostgresTaskRepository (this crate)
    ↓ uses
PostgreSQL Database
```

**Dependency flow**: `app-http` → `business-core` → `adapters-db-sqlx` → `model`

## Testing

Integration tests use testcontainers for real PostgreSQL instances:

```bash
# Run integration tests
cargo test -p adapters-db-sqlx --test integration
```

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| `app-http` | Provides database-backed task persistence |
| Tests | Integration tests for repository behavior |

## See Also

- [`business-core/README.md`](../business-core/README.md) - Repository port definition
- [`model/README.md`](../model/README.md) - Domain types
- [`app-http/README.md`](../app-http/README.md) - HTTP layer that uses this adapter
