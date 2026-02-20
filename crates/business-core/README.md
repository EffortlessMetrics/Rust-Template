# business-core

Core business logic and domain services for the Rust-as-Spec platform.

## What It Is

`business-core` contains the pure business logic layer of the application. It defines:

- **Ports** (traits): Repository interfaces that adapters must implement
- **Use cases**: Application logic functions that orchestrate domain operations
- **Governance re-exports**: Production governance types from `gov-model`

This crate follows clean architecture principles: it has no dependencies on HTTP, database, or other infrastructure concerns.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `ports` | Repository trait definitions (e.g., `ExampleTaskRepository`) |
| `use_cases` | Business logic functions (e.g., `create_example_task`, `list_example_tasks`) |
| `governance` | Re-exports of governance domain types from `gov-model` |

### What It Is Not

- **Not an HTTP layer**: Request/response handling lives in `app-http`
- **Not data access**: Database implementations live in `adapters-db-sqlx`
- **Not domain types**: Core types live in `model` and `gov-model`

## Architecture

The crate follows hexagonal/clean architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                  Adapters Layer                        │
│  ┌──────────────┐         ┌──────────────────┐  │
│  │  app-http   │         │ adapters-db-sqlx│  │
│  │  (HTTP)      │         │   (PostgreSQL)   │  │
│  └──────┬───────┘         └────────┬─────────┘  │
│         │                           │               │
└─────────┼───────────────────────────┼───────────────┘
          │                           │
          │ depends on                │ implements
┌─────────┼───────────────────────────┼───────────────┐
│         │                           │               │
│  ┌──────▼──────┐         ┌───────▼──────────┐  │
│  │ business-core│         │  Ports (traits)  │  │
│  │  (this crate)│◄────────│  Repository      │  │
│  └──────┬──────┘         └──────────────────┘  │
│         │                                   │
│         │ uses                             │
│  ┌──────▼──────┐                           │
│  │    model     │                           │
│  │  (types)    │                           │
│  └─────────────┘                           │
└─────────────────────────────────────────────────────┘
```

**Dependency flow**: `app-http` → `business-core` → `model`/`gov-model`

## Ports (Repository Traits)

The `ports` module defines interfaces that adapters implement:

### ExampleTaskRepository

```rust
#[async_trait]
pub trait ExampleTaskRepository: Send + Sync {
    async fn save(&self, task: &ExampleTask) -> Result<(), String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ExampleTask>, String>;
    async fn find_all(&self) -> Result<Vec<ExampleTask>, String>;
    async fn update_status(
        &self,
        id: &str,
        status: ExampleTaskStatus,
    ) -> Result<Option<ExampleTask>, String>;
}
```

## Use Cases

The `use_cases` module provides business logic functions:

### Creating a Task

```rust
use business_core::use_cases::create_example_task;
use adapters_db_sqlx::PostgresTaskRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = PostgresTaskRepository::new().await?;

    let task = create_example_task(&repo, "My task".to_string()).await?;
    println!("Created task: {} with status: {:?}", task.id, task.status);

    Ok(())
}
```

### Listing Tasks

```rust
use business_core::use_cases::list_example_tasks;

let tasks = list_example_tasks(&repo).await?;
for task in tasks {
    println!("{}: {}", task.id, task.title);
}
```

### Updating Task Status

```rust
use business_core::use_cases::update_example_task_status;
use model::ExampleTaskStatus;

let updated = update_example_task_status(
    &repo,
    task_id.to_string(),
    ExampleTaskStatus::Completed,
).await?;
```

## Governance Types

The `governance` module re-exports production governance types from `gov-model`:

```rust
use business_core::governance::{Task, TaskStatus, TaskService};

// Use governance types directly
let status = TaskStatus::Todo;
let service = TaskService::new(repo);
```

For new code, prefer importing directly from `gov-model` crate.

## Design Principles

1. **Infrastructure Independence**: No dependencies on HTTP, database, or frameworks
2. **Port-Adapter Pattern**: Core defines interfaces, adapters implement them
3. **Pure Business Logic**: Functions contain only domain rules, no I/O
4. **Async-First**: All repository operations are async

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| `app-http` | HTTP handlers call use cases for business operations |
| `adapters-db-sqlx` | Implements `ExampleTaskRepository` port |
| Tests | Unit tests for business logic behavior |

## See Also

- [`model/README.md`](../model/README.md) - Domain types (ExampleTask, ExampleTaskStatus)
- [`gov-model/README.md`](../gov-model/README.md) - Governance domain model
- [`adapters-db-sqlx/README.md`](../adapters-db-sqlx/README.md) - PostgreSQL adapter implementation
- [`app-http/README.md`](../app-http/README.md) - HTTP layer that calls this crate

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.89.0.
