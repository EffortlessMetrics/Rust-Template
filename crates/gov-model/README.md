# gov-model

Governance domain model types for Rust-as-Spec platform.

## What It Is

`gov-model` contains pure domain types for the platform's governance system. It defines the foundational types that other kernel crates depend on for governance semantics.

This crate has zero dependencies on HTTP, database, or other infrastructure—it is pure domain logic.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | Core governance types: `Task`, `TaskStatus`, `TaskService`, `GovernanceRepository` |
| `context.rs` | Repository context: `RepoContext`, `SpecLayout` (workspace paths) |

### What It Is Not

- **Not business logic**: Application logic lives in `business-core`
- **Not data access**: Repository implementations live in adapters
- **Not HTTP layer**: Request/response handling lives in `app-http`

## Core Types

### TaskStatus

Production governance task status with enforced state transitions:

```rust
pub enum TaskStatus {
    Todo,       // Task is not yet started
    InProgress, // Task is actively being worked on
    Review,     // Task is complete and awaiting review
    Done,        // Task is fully complete and approved
}
```

#### State Transitions

Allowed transitions:
- `Todo` → `InProgress`
- `InProgress` → `Review` (or back to `Todo`)
- `Review` → `Done` (or back to `InProgress`)

```rust
use gov_model::TaskStatus;

let current = TaskStatus::Todo;
let next = TaskStatus::InProgress;

if current.can_transition_to(&next) {
    println!("Valid transition");
}
```

#### Parsing

Status values parse case-insensitively with common aliases:

```rust
use std::str::FromStr;

// All parse to TaskStatus::Todo
"todo".parse::<TaskStatus>()?;
"open".parse::<TaskStatus>()?;
"Todo".parse::<TaskStatus>()?;
"TODO".parse::<TaskStatus>()?;

// All parse to TaskStatus::Done
"done".parse::<TaskStatus>()?;
"closed".parse::<TaskStatus>()?;
"completed".parse::<TaskStatus>()?;
```

Display output uses canonical form: `"Todo"`, `"InProgress"`, `"Review"`, `"Done"`.

### Task

A governance task with ID, title, and status:

```rust
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
}

pub struct TaskId(pub String);
```

### GovernanceRepository

Repository trait for governance task persistence:

```rust
pub trait GovernanceRepository: Send + Sync {
    fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError>;
    fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError>;
    fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError>;
    fn is_healthy(&self) -> bool { true } // Default implementation
}
```

### TaskService

Service for managing task lifecycle and enforcing transition rules:

```rust
use gov_model::{TaskService, TaskStatus, TaskId};

let service = TaskService::new(repo);

// Move task through workflow (enforces valid transitions)
service.move_task(&task_id, TaskStatus::InProgress)?;
service.move_task(&task_id, TaskStatus::Review)?;
service.move_task(&task_id, TaskStatus::Done)?;

// List all tasks
let tasks = service.list_tasks()?;
```

### GovernanceError

Errors that can occur during governance operations:

```rust
pub enum GovernanceError {
    Io(std::io::Error),
    Serialization(String),
    TaskNotFound(TaskId),
    Lock(String),
    InvalidTransition { from: TaskStatus, to: TaskStatus },
}
```

## Repository Context

### RepoContext

Provides workspace paths and configuration for kernel crates:

```rust
use gov_model::{RepoContext, SpecLayout};

let context = RepoContext::new("/path/to/workspace")
    .with_service_id("my-service");

// Access standard paths
let specs_dir = context.specs_dir();
let config_dir = context.config_dir();
let docs_dir = context.docs_dir();
```

### SpecLayout

Customizable directory layout for spec files:

```rust
let layout = SpecLayout {
    specs_dir: "specs".to_string(),
    config_dir: "config".to_string(),
    policy_dir: "policy".to_string(),
    docs_dir: "docs".to_string(),
};

let context = RepoContext::new(workspace_root).with_layout(layout);
```

## Usage Examples

### Basic Task Workflow

```rust
use gov_model::{Task, TaskId, TaskStatus, TaskService};

// Create a task (typically via repository)
let task = Task {
    id: TaskId("TASK-001".to_string()),
    title: "Implement feature".to_string(),
    status: TaskStatus::Todo,
};

// Move through workflow
let service = TaskService::new(repo);
service.move_task(&task.id, TaskStatus::InProgress)?;
service.move_task(&task.id, TaskStatus::Review)?;
service.move_task(&task.id, TaskStatus::Done)?;
```

### Status Parsing and Validation

```rust
use gov_model::TaskStatus;
use std::str::FromStr;

// Parse from user input
let status: TaskStatus = "in-progress".parse()?;

// Check if terminal
if status.is_done() {
    println!("Task is complete");
}

// Validate transition
if TaskStatus::Todo.can_transition_to(&status) {
    println!("Valid transition");
} else {
    println!("Invalid transition");
}
```

### Custom Repository Implementation

```rust
use gov_model::{GovernanceRepository, Task, TaskId, GovernanceError};

struct MyRepository {
    // Your storage here
}

impl GovernanceRepository for MyRepository {
    fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
        // Your implementation
    }

    fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
        // Your implementation
    }

    fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError> {
        // Your implementation
    }
}
```

## Design Principles

1. **Pure Domain**: No infrastructure dependencies
2. **Explicit State Machine**: All transitions are validated
3. **Flexible Parsing**: Accepts common status aliases
4. **Repository Pattern**: Defines ports, not implementations

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| `business-core` | Re-exports governance types for backward compatibility |
| `gov-xtask-core` | Uses `RepoContext` for workspace path resolution |
| `gov-receipts` | Uses governance types for receipt generation |
| Adapters | Implement `GovernanceRepository` trait |

## See Also

- [`business-core/README.md`](../business-core/README.md) - Business logic that re-exports these types
- [`gov-xtask-core/README.md`](../gov-xtask-core/README.md) - Uses `RepoContext`
- [`gov-receipts/README.md`](../gov-receipts/README.md) - Receipt generation using governance types
