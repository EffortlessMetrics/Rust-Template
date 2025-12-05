<!-- doclint:disable orphan-version -->
# Adapter Architecture

This document explains the hexagonal/ports-and-adapters architecture implemented in this Rust template, with concrete examples from the database, gRPC, and HTTP adapters.

## Table of Contents

1. [Overview](#overview)
2. [Adapter Types](#adapter-types)
3. [How to Add a New Adapter](#how-to-add-a-new-adapter)
4. [Multi-Service Architecture](#multi-service-architecture)
5. [Testing Strategy](#testing-strategy)
6. [Governance Integration](#governance-integration)

---

## Overview

### What is Hexagonal Architecture?

Hexagonal architecture (also called "ports and adapters") is a pattern that isolates core business logic from external dependencies. The core domain is at the center (the "hexagon"), and adapters connect it to the outside world.

```
┌─────────────────────────────────────────────────────────────┐
│                      External Systems                       │
│  HTTP Clients  │  Postgres  │  gRPC Clients  │  Events     │
└────────┬────────────┬─────────────┬────────────────┬─────────┘
         │            │             │                │
         ▼            ▼             ▼                ▼
    ┌────────┐  ┌──────────┐  ┌─────────┐   ┌──────────────┐
    │app-http│  │adapters- │  │adapters-│   │adapters-     │
    │        │  │db-sqlx   │  │grpc     │   │events (TBD)  │
    └────┬───┘  └─────┬────┘  └────┬────┘   └──────┬───────┘
         │            │             │               │
         └────────────┴─────────────┴───────────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │      CORE       │
                   │  (Hexagon)      │
                   │                 │
                   │  • use_cases    │
                   │  • ports        │
                   │  • domain logic │
                   └─────────────────┘
                            │
                            ▼
                      ┌──────────┐
                      │  MODEL   │
                      │ entities │
                      └──────────┘
```

### Why Adapters Isolate Technology Choices

**Key Principle:** Dependencies point INWARD. Core never depends on adapters.

This creates several benefits:

1. **Testability:** Test core logic without spinning up databases or HTTP servers
2. **Flexibility:** Swap Postgres for MongoDB without changing business logic
3. **Clarity:** Business rules live in one place (`core/`), not scattered across infrastructure code
4. **Maintainability:** Upgrade Axum to a newer version without touching domain logic

**The Dependency Rule:**

```rust
// ✓ CORRECT: Adapter depends on core
use core::ports::TaskRepository;
use core::use_cases;

pub struct PostgresTaskRepository { ... }
impl TaskRepository for PostgresTaskRepository { ... }
```

```rust
// ✗ WRONG: Core depends on adapter
use adapters_db_sqlx::PostgresTaskRepository;  // ✗ Never!

pub fn create_task() {
    let repo = PostgresTaskRepository::new();  // ✗ Core knows about Postgres!
}
```

### Ports vs Adapters

- **Ports:** Traits/interfaces defined in `core/` that specify WHAT the domain needs
- **Adapters:** Concrete implementations that specify HOW to do it with a specific technology

Example port definition:

```rust
// crates/core/src/ports.rs
pub trait TaskRepository {
    fn save(&self, task: &Task) -> Result<(), String>;
    fn find_by_id(&self, id: &str) -> Result<Option<Task>, String>;
    fn find_all(&self) -> Result<Vec<Task>, String>;
    fn update_status(&self, id: &str, status: TaskStatus) -> Result<Option<Task>, String>;
}
```

The port says "the domain needs a way to save/find tasks" but doesn't specify database, in-memory, file system, etc.

---

## Adapter Types

### HTTP Adapter (app-http)

**Purpose:** Expose domain operations via REST endpoints.

**Location:** `/home/steven/code/Rust/Rust-Template/crates/app-http/`

**Technology:** Axum web framework with Tower middleware

**Responsibilities:**
- Route HTTP requests to handlers
- Deserialize JSON payloads into domain types
- Call `core::use_cases` functions
- Serialize domain types back to JSON responses
- HTTP-specific concerns: status codes, headers, request IDs

**Key Files:**
- `src/lib.rs`: Router setup, handlers, DTOs
- `src/main.rs`: Server initialization
- `src/errors.rs`: HTTP error responses
- `src/middleware/`: Request ID, tracing

**Example Handler:**

```rust
// crates/app-http/src/lib.rs
#[instrument(skip(request_id, payload))]
async fn echo(
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<EchoRequest>,
) -> Result<Json<EchoResponse>, AppError> {
    // 1. Validate HTTP input
    if payload.message.is_empty() {
        return Err(AppError::validation_error(
            ErrorCode::MissingField,
            "Message cannot be empty"
        ));
    }

    // 2. Call domain (in real app: core::use_cases::process_message)
    // 3. Return HTTP response
    Ok(Json(EchoResponse { message: payload.message }))
}
```

**Dependencies:**

```toml
# crates/app-http/Cargo.toml
[dependencies]
core = { path = "../core" }        # ✓ Adapter → Core
model = { path = "../model" }      # ✓ Adapter → Model
telemetry = { path = "../telemetry" }
axum = "0.8.7"
tokio = { version = "1.48.0", features = ["full"] }
```

### Database Adapter (adapters-db-sqlx)

**Purpose:** Persist domain entities in Postgres via sqlx.

**Location:** `/home/steven/code/Rust/Rust-Template/crates/adapters-db-sqlx/`

**Technology:** sqlx (compile-time checked SQL), Postgres

**Responsibilities:**
- Implement `TaskRepository` port from core
- Execute SQL queries
- Map database rows to domain models
- Handle connection pooling

**Key Files:**
- `src/lib.rs`: Repository implementation
- `tests/integration.rs`: Testcontainers-based tests
- `migrations/`: SQL schema migrations (if using sqlx migrations)

**Example Implementation:**

```rust
// crates/adapters-db-sqlx/src/lib.rs
use core::ports::TaskRepository;
use core::model::{Task, TaskStatus};
use sqlx::PgPool;

pub struct PostgresTaskRepository {
    pool: PgPool,
}

#[async_trait::async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn create_task(&self, title: String) -> Result<Task> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, status, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(&title)
        .bind("PENDING")
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(Task {
            id: id.to_string(),
            title,
            status: TaskStatus::Pending,
        })
    }

    // ... other methods
}
```

**Schema Management:**

Database schema lives in `/home/steven/code/Rust/Rust-Template/specs/db/schema.hcl`:

```hcl
table "tasks" {
  schema = schema.public

  column "id" {
    type = uuid
    primary_key = true
  }

  column "title" {
    type = varchar(255)
    null = false
  }

  column "status" {
    type = varchar(20)
    check = "status IN ('PENDING', 'COMPLETED')"
  }
}
```

**Dependencies:**

```toml
# crates/adapters-db-sqlx/Cargo.toml
[dependencies]
core = { path = "../core" }        # ✓ Adapter → Core
sqlx = { version = "0.8.6", features = ["postgres", "uuid", "chrono"] }
tokio = { version = "1.48.0", features = ["full"] }

[dev-dependencies]
testcontainers = "0.25.2"          # For integration tests
```

### gRPC Adapter (adapters-grpc)

**Purpose:** Expose domain operations via gRPC/Protobuf endpoints.

**Location:** `/home/steven/code/Rust/Rust-Template/crates/adapters-grpc/`

**Technology:** Tonic (gRPC framework), prost (Protobuf)

**Responsibilities:**
- Implement gRPC service traits generated from `.proto` files
- Convert Protobuf messages to/from domain models
- Call `core::use_cases` functions
- Handle gRPC-specific concerns: status codes, metadata

**Key Files:**
- `src/lib.rs`: Service implementation
- `build.rs`: Compile `.proto` files at build time
- `/specs/proto/task/v1/task.proto`: Protobuf schema

**Example Implementation:**

```rust
// crates/adapters-grpc/src/lib.rs
use tonic::{Request, Response, Status};
use core::use_cases;
use core::ports::TaskRepository;

pub struct TaskServiceImpl {
    repo: Arc<dyn TaskRepository>,
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let title = request.into_inner().title;

        // Call core use case
        let task = use_cases::create_task(&*self.repo, title)
            .map_err(|e| Status::internal(e))?;

        // Convert domain model to Protobuf
        let proto_task = model_task_to_proto(&task);
        Ok(Response::new(CreateTaskResponse { task: Some(proto_task) }))
    }
}
```

**Proto Schema:**

```protobuf
// specs/proto/task/v1/task.proto
syntax = "proto3";
package task.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/uuid.proto";

message Task {
  google.protobuf.UUID id = 1;
  string title = 2;
  string status = 3;
  google.protobuf.Timestamp created_at = 4;
}

service TaskService {
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc GetTask(GetTaskRequest) returns (Task);
  rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);
}
```

**Build Script:**

```rust
// crates/adapters-grpc/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../../specs/proto/task/v1/task.proto")?;
    Ok(())
}
```

### Events Adapter (adapters-events)

**Purpose:** Publish domain events to message brokers (NATS, Kafka, etc.).

**Status:** Not yet implemented, but follows same pattern.

**Planned Structure:**

```rust
// crates/adapters-events/src/lib.rs (hypothetical)
use core::ports::EventPublisher;
use core::model::DomainEvent;

pub struct NatsEventPublisher {
    client: nats::Client,
}

impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<()> {
        let subject = format!("tasks.{}", event.event_type());
        let payload = serde_json::to_vec(&event)?;
        self.client.publish(&subject, payload).await?;
        Ok(())
    }
}
```

**Event Schema:**

Events would be validated against schemas in `/home/steven/code/Rust/Rust-Template/specs/events/`:

```yaml
# specs/events/subjects.yaml
subjects:
  - name: tasks-created
    schema: json-schema/tasks-created.json
  - name: tasks-completed
    schema: json-schema/tasks-completed.json
```

---

## How to Add a New Adapter

This section walks through adding a new adapter (e.g., Redis cache) step-by-step.

### Step 1: Define Port in Core

First, determine what your domain needs. Define a trait in `core/src/ports.rs`:

```rust
// crates/core/src/ports.rs
pub trait Cache {
    async fn get(&self, key: &str) -> Result<Option<String>, String>;
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), String>;
    async fn delete(&self, key: &str) -> Result<(), String>;
}
```

**Key Point:** The port describes WHAT you need (get/set/delete), not HOW (Redis vs Memcached).

### Step 2: Create Adapter Crate

```bash
cd crates
cargo new adapters-cache-redis --lib
```

Edit `Cargo.toml` at workspace root:

```toml
# Cargo.toml
[workspace]
members = [
  "crates/*",
  "crates/adapters-db-sqlx",
  "crates/adapters-grpc",
  "crates/adapters-cache-redis",  # ← Add this
]
```

### Step 3: Implement Adapter

```rust
// crates/adapters-cache-redis/src/lib.rs
use core::ports::Cache;
use redis::AsyncCommands;
use std::time::Duration;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        let value: Option<String> = conn.get(key)
            .await
            .map_err(|e| e.to_string())?;

        Ok(value)
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), String> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        conn.set_ex(key, value, ttl.as_secs())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        conn.del(key)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

Add dependencies:

```toml
# crates/adapters-cache-redis/Cargo.toml
[package]
name = "adapters-cache-redis"
version = "0.1.0"
edition.workspace = true

[dependencies]
core = { path = "../core" }
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1.48", features = ["full"] }
async-trait = "0.1"

[dev-dependencies]
testcontainers = "0.25"
```

### Step 4: Wire in main.rs

```rust
// crates/app-http/src/main.rs
use adapters_cache_redis::RedisCache;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init();

    // Initialize adapters
    let cache = RedisCache::new("redis://localhost:6379").await?;
    let cache: Arc<dyn core::ports::Cache> = Arc::new(cache);

    // Pass to handlers via Extension
    let app = app_http::app()
        .layer(Extension(cache));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

Use in handlers:

```rust
// crates/app-http/src/lib.rs
async fn get_cached_task(
    Extension(cache): Extension<Arc<dyn Cache>>,
    Path(id): Path<String>,
) -> Result<Json<Task>, AppError> {
    let cache_key = format!("task:{}", id);

    if let Some(cached) = cache.get(&cache_key).await? {
        let task: Task = serde_json::from_str(&cached)?;
        return Ok(Json(task));
    }

    // ... fetch from database, cache result
}
```

### Step 5: Add Integration Tests

```rust
// crates/adapters-cache-redis/tests/integration.rs
#[cfg(feature = "integration-cache")]
mod integration_tests {
    use adapters_cache_redis::RedisCache;
    use core::ports::Cache;
    use testcontainers::{clients::Cli, images::redis::Redis};
    use std::time::Duration;

    #[tokio::test]
    async fn test_redis_cache_operations() -> anyhow::Result<()> {
        let docker = Cli::default();
        let redis = docker.run(Redis::default());
        let port = redis.get_host_port_ipv4(6379);

        let redis_url = format!("redis://localhost:{}", port);
        let cache = RedisCache::new(&redis_url).await?;

        // Test set & get
        cache.set("key1", "value1", Duration::from_secs(60)).await?;
        let value = cache.get("key1").await?;
        assert_eq!(value, Some("value1".to_string()));

        // Test delete
        cache.delete("key1").await?;
        let value = cache.get("key1").await?;
        assert_eq!(value, None);

        Ok(())
    }
}
```

Enable feature flag:

```toml
# crates/adapters-cache-redis/Cargo.toml
[features]
integration-cache = []
```

Run tests:

```bash
cargo test -p adapters-cache-redis --features integration-cache
```

### Step 6: Document the Adapter

Update this file (`docs/explanation/adapters.md`) with your new adapter in the "Adapter Types" section.

---

## Multi-Service Architecture

### Current State

The template is structured as a **monolithic application** with multiple adapters (HTTP, gRPC, DB). All adapters run in one process, sharing the same core domain.

### Future: Multi-Service Pattern

For larger systems, you can split into multiple services, each with its own adapters but sharing core domain logic.

**Example Structure:**

```
services/
├── task-api/           # HTTP service
│   ├── src/main.rs     # HTTP adapter only
│   └── Cargo.toml
├── task-worker/        # Background worker service
│   ├── src/main.rs     # Event consumer adapter
│   └── Cargo.toml
└── shared-core/        # Shared business logic
    ├── core/
    ├── model/
    └── Cargo.toml
```

**task-api service:**

```rust
// services/task-api/src/main.rs
use shared_core::core;
use adapters_db_sqlx::PostgresTaskRepository;
use adapters_cache_redis::RedisCache;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = PostgresTaskRepository::new().await?;
    let cache = RedisCache::new("redis://localhost").await?;

    let app = Router::new()
        .route("/tasks", post(create_task))
        .layer(Extension(Arc::new(db)))
        .layer(Extension(Arc::new(cache)));

    // ... serve HTTP
}
```

**task-worker service:**

```rust
// services/task-worker/src/main.rs
use shared_core::core;
use adapters_events::NatsSubscriber;
use adapters_db_sqlx::PostgresTaskRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = PostgresTaskRepository::new().await?;
    let subscriber = NatsSubscriber::new("nats://localhost").await?;

    subscriber.subscribe("tasks.created", |event| {
        // Process event using core::use_cases
        core::use_cases::process_task_created(event, &db)?;
        Ok(())
    }).await?;

    // ... run event loop
}
```

### When to Split Services

**Keep monolithic when:**
- Single team owns all code
- Deployment complexity is a concern
- System fits on one machine
- Debugging across boundaries is expensive

**Split into services when:**
- Different teams own different components
- Scaling needs differ (API needs horizontal scaling, worker doesn't)
- Independent deployment cycles are valuable
- Failure isolation is critical (worker crash shouldn't take down API)

**Shared-Core Pattern:**

Both services depend on the same `core` crate:

```toml
# services/task-api/Cargo.toml
[dependencies]
core = { path = "../../crates/core" }
adapters-db-sqlx = { path = "../../crates/adapters-db-sqlx" }

# services/task-worker/Cargo.toml
[dependencies]
core = { path = "../../crates/core" }
adapters-db-sqlx = { path = "../../crates/adapters-db-sqlx" }
adapters-events = { path = "../../crates/adapters-events" }
```

This ensures business rules are consistent across services.

---

## Testing Strategy

### Unit Tests in Core (No Adapters)

Core business logic should be tested WITHOUT any adapters. Use simple in-memory stubs.

```rust
// crates/core/src/use_cases.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::TaskRepository;
    use crate::model::{Task, TaskStatus};
    use std::sync::Mutex;
    use std::collections::HashMap;

    // In-memory stub - no database needed!
    struct InMemoryTaskRepository {
        tasks: Mutex<HashMap<String, Task>>,
    }

    impl TaskRepository for InMemoryTaskRepository {
        fn save(&self, task: &Task) -> Result<(), String> {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task.id.clone(), task.clone());
            Ok(())
        }

        fn find_by_id(&self, id: &str) -> Result<Option<Task>, String> {
            let tasks = self.tasks.lock().unwrap();
            Ok(tasks.get(id).cloned())
        }
    }

    #[test]
    fn test_create_task() {
        let repo = InMemoryTaskRepository {
            tasks: Mutex::new(HashMap::new())
        };

        let task = create_task(&repo, "Test task".to_string()).unwrap();

        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);

        // Verify it was saved
        let saved = repo.find_by_id(&task.id).unwrap();
        assert!(saved.is_some());
    }
}
```

**Key Point:** Core tests are fast (no I/O), deterministic (no database state), and isolated.

### Integration Tests Per Adapter

Each adapter has its own integration tests that verify it correctly implements the port.

**Database Adapter Tests (using Testcontainers):**

Location: `crates/adapters-db-sqlx/tests/integration.rs`

The DB adapter integration test verifies the full repository CRUD cycle:
1. Spins up a PostgreSQL container via testcontainers
2. Creates test database and tables
3. Tests save, find_by_id, update_status, and find_all operations
4. Automatically cleans up containers after test completes

**Why Testcontainers?**
- Spins up real Postgres in Docker
- Isolated (each test gets fresh container)
- Automatic cleanup
- Tests against real database behavior, not mocks

**Running Integration Tests:**

The integration tests are marked with `#[ignore]` by default to keep CI fast and avoid requiring Docker in all environments.

To run the DB adapter integration test:

```bash
# Requires Docker running
cargo test -p adapters-db-sqlx -- --ignored

# Or run all ignored tests in the workspace
cargo test --workspace -- --ignored
```

**Requirements:**
- Docker must be running
- Testcontainers will automatically pull `postgres:16-alpine` image

**What the test validates:**
- Task creation and persistence
- Finding task by ID
- Status updates (Pending → InProgress → Completed)
- Listing all tasks
- Proper UUID and timestamp handling

**gRPC Adapter Smoke Test:**

Location: `crates/adapters-grpc/tests/smoke.rs`

The gRPC adapter smoke test verifies the service works end-to-end:
1. Creates an in-memory TaskRepository (no DB dependency)
2. Starts a gRPC server on a test port
3. Creates a tonic client
4. Tests CreateTask, GetTask, and ListTasks RPCs
5. Automatically cleans up server after test completes

**Running gRPC Smoke Test:**

The smoke test is marked with `#[ignore]` by default.

To run the gRPC adapter smoke test:

```bash
# Run smoke test
cargo test -p adapters-grpc --test smoke -- --ignored

# Or run all ignored tests in the workspace
cargo test --workspace -- --ignored
```

**Requirements:**
- No external dependencies (uses in-memory repository)
- Tests full gRPC request/response cycle

**What the test validates:**
- Task creation via gRPC CreateTask RPC
- Task retrieval via gRPC GetTask RPC
- Task listing via gRPC ListTasks RPC
- Protobuf serialization/deserialization
- Service implementation correctness

### In-Memory Stubs for Events

For event publishers, use in-memory implementations in tests:

```rust
// crates/adapters-events/src/lib.rs
#[cfg(test)]
pub struct InMemoryEventPublisher {
    pub events: Arc<Mutex<Vec<DomainEvent>>>,
}

impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_event_publishing() {
        let publisher = InMemoryEventPublisher {
            events: Arc::new(Mutex::new(Vec::new())),
        };

        let event = DomainEvent::TaskCreated { id: "123".to_string() };
        publisher.publish(&event).await.unwrap();

        let events = publisher.events.lock().unwrap();
        assert_eq!(events.len(), 1);
    }
}
```

### Test Pyramid

```
              ┌──────────────────┐
              │  Acceptance Tests │  (BDD, full system)
              │   (Cucumber)      │
              └──────────────────┘
            ┌──────────────────────┐
            │ Integration Tests    │  (Adapters + real infra)
            │  (Testcontainers)    │
            └──────────────────────┘
        ┌──────────────────────────────┐
        │     Unit Tests               │  (Core + stubs)
        │   (In-memory repos)          │
        └──────────────────────────────┘
```

**Guidelines:**
- **Most tests:** Unit tests in core (fast, isolated)
- **Some tests:** Integration tests per adapter (verify real behavior)
- **Few tests:** Acceptance tests (end-to-end, slow but high confidence)

---

## Governance Integration

### How Policies Apply to Adapters

Adapters are subject to the same governance as core code:

1. **Acceptance Criteria:** New adapter features need ACs in `specs/spec_ledger.yaml`
2. **Tests:** ACs must have mapped tests (enforced by `policy/ledger.rego`)
3. **Schema Management:** External schemas (DB, Proto, Events) versioned in `specs/`
4. **PII Handling:** If adapters touch PII, must be declared in `specs/privacy.yaml`

**Example AC for Database Adapter:**

```yaml
# specs/spec_ledger.yaml
stories:
  - id: US-042
    title: "Task Persistence"
    requirements:
      - id: REQ-042-01
        text: "Tasks must be stored in Postgres"
        acceptance_criteria:
          - id: AC-DB-001
            text: "Repository saves task with UUID"
            tests:
              - type: integration
                location: "crates/adapters-db-sqlx/tests/integration.rs::test_task_repository_crud"
```

### Schema Management

All external schemas are versioned in `specs/`:

#### Database Schemas

```
specs/db/
├── schema.hcl                    # Atlas HCL schema
└── atlas/
    └── migrations/
        └── 20250101000000_init.sql
```

**Workflow:**
1. Edit `specs/db/schema.hcl`
2. Generate migration: `atlas migrate diff --env dev`
3. Apply migration: `atlas migrate apply --env prod`
4. Adapters use schema via sqlx compile-time checks

#### Protobuf Schemas

```
specs/proto/
└── task/
    └── v1/
        └── task.proto
```

**Workflow:**
1. Edit `.proto` file in `specs/proto/`
2. Adapter's `build.rs` compiles it: `tonic_build::compile_protos("specs/proto/task/v1/task.proto")`
3. Generated Rust code is checked into `target/` (gitignored) or committed

**Example:**

```rust
// crates/adapters-grpc/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../../specs/proto/task/v1/task.proto")?;
    Ok(())
}
```

#### Event Schema Validation

```
specs/events/
├── subjects.yaml                 # Event subject registry
└── json-schema/
    └── tasks-created.json        # JSON Schema for event
```

**Planned Workflow:**
1. Define event schema in `specs/events/json-schema/`
2. Register in `subjects.yaml`
3. Adapter validates events before publishing:

```rust
// crates/adapters-events/src/lib.rs (hypothetical)
impl NatsEventPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<()> {
        // Validate against schema
        let schema = load_schema(&event.event_type())?;
        schema.validate(&event)?;

        // Publish
        self.client.publish(event.subject(), event.payload()).await?;
        Ok(())
    }
}
```

### Policy Checks

Relevant policies:

- **`policy/ledger.rego`:** Ensures ACs have tests (applies to adapter tests)
- **`policy/privacy.rego`:** Validates PII handling (applies if adapters touch PII)
- **`policy/features.rego`:** Feature flags reference valid ACs

**Example Policy Violation:**

```bash
$ cargo xtask check --policy

FAIL: AC-DB-001 has no mapped tests
  Expected: tests section with location
  Found: none
  Fix: Add integration test to crates/adapters-db-sqlx/tests/
```

### Traceability

Full traceability from requirement to adapter:

```
US-042: Task Persistence
  └── REQ-042-01: Store in Postgres
      └── AC-DB-001: Repository saves with UUID
          └── Test: adapters-db-sqlx/tests/integration.rs::test_task_repository_crud
              └── Implementation: adapters-db-sqlx/src/lib.rs::PostgresTaskRepository::save
```

Query with:

```bash
cargo xtask ac-status --id AC-DB-001
```

---

## Dependency Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     EXECUTION FLOW                      │
│                                                         │
│  HTTP Request → app-http → core::use_cases → port      │
│                                 ↓                        │
│                            adapter (impl port)          │
│                                 ↓                        │
│                            Postgres/Redis/NATS          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                    DEPENDENCY FLOW                      │
│                                                         │
│  app-http ────────────→ core ←──────── adapters-db-sqlx│
│      │                   │                      │       │
│      └─────→ model ←─────┘                      │       │
│                                                  │       │
│  adapters-grpc ───────→ core ←──────────────────┘       │
│      │                   │                              │
│      └─────→ model ←─────┘                              │
└─────────────────────────────────────────────────────────┘

Key:
  ──→  Dependency (imports crate)
  ←──  Implements port from
```

---

## Request Path Through Layers

### Example: Create Task via HTTP

```
1. HTTP POST /tasks {"title": "Buy milk"}
   │
   ▼
2. app-http/src/lib.rs::create_task()
   │ • Deserialize JSON
   │ • Validate HTTP input
   │ • Extract request_id
   ▼
3. core/use_cases::create_task(repo, title)
   │ • Generate UUID
   │ • Create Task entity
   │ • Apply business rules
   ▼
4. repo.save(&task)  [trait call - polymorphic]
   │
   ▼
5. adapters-db-sqlx/src/lib.rs::PostgresTaskRepository::save()
   │ • Execute INSERT query
   │ • Map result to domain model
   ▼
6. Postgres database
   │
   ◀ Return up the stack
   │
7. app-http serializes Task to JSON
   │
   ▼
8. HTTP 201 {"id": "...", "title": "Buy milk", "status": "pending"}
```

### Example: Same Task via gRPC

```
1. gRPC CreateTask{title: "Buy milk"}
   │
   ▼
2. adapters-grpc/src/lib.rs::TaskServiceImpl::create_task()
   │ • Deserialize Protobuf
   │ • Extract title
   ▼
3. core/use_cases::create_task(repo, title)  [SAME AS HTTP!]
   │ • Generate UUID
   │ • Create Task entity
   ▼
4. repo.save(&task)
   │
   ▼
5. PostgresTaskRepository::save()
   │ • Execute INSERT
   ▼
6. Postgres
   │
   ◀ Return
   │
7. adapters-grpc converts Task → ProtoTask
   │
   ▼
8. gRPC response {task: {...}}
```

**Key Insight:** Steps 3-6 are IDENTICAL. Only steps 1-2 and 7-8 differ between HTTP and gRPC. This is the power of hexagonal architecture.

---

## Summary

### Key Takeaways

1. **Ports define interfaces**, adapters provide implementations
2. **Dependencies point inward** to core domain
3. **Core never depends** on HTTP, databases, or other infrastructure
4. **Each adapter is isolated** and can be swapped independently
5. **Testing is layered:** unit tests (core), integration tests (adapters), acceptance tests (full system)
6. **Schemas are versioned** in `specs/` and enforced by governance policies

### Quick Reference

| Adapter          | Purpose                 | Technology | Port Interface      |
|------------------|-------------------------|------------|---------------------|
| app-http         | REST API                | Axum       | N/A (entry point)   |
| adapters-db-sqlx | Database persistence    | sqlx       | TaskRepository      |
| adapters-grpc    | gRPC API                | Tonic      | N/A (entry point)   |
| adapters-events  | Event publishing (TBD)  | NATS       | EventPublisher      |

### Next Steps

- **Add a new adapter:** Follow [How to Add a New Adapter](#how-to-add-a-new-adapter)
- **Understand governance:** Read `/home/steven/code/Rust/Rust-Template/docs/explanation/architecture.md`
- **Write integration tests:** See examples in `crates/adapters-db-sqlx/tests/integration.rs`
- **Explore multi-service:** Plan service boundaries based on team/scaling needs

### Related Documentation

- [Architecture Overview](/home/steven/code/Rust/Rust-Template/docs/explanation/architecture.md)
- [First AC Change Tutorial](/home/steven/code/Rust/Rust-Template/docs/tutorials/) (TBD)
- [Testing Strategy](/home/steven/code/Rust/Rust-Template/docs/how-to/) (TBD)
- [Governance Model](/home/steven/code/Rust/Rust-Template/docs/explanation/TEMPLATE-CONTRACTS.md)
