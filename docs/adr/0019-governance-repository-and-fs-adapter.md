---
id: ADR-0019
title: Governance Repository and FS Adapter
doc_type: adr
status: accepted
audience: developers, architects
tags: [architecture, persistence, governance, file-system]
stories: [US-TPL-PLATFORM-V3]
requirements: [REQ-TPL-GOV-WRITE-001]
acs: [AC-TPL-GOV-WRITE-TASK-STATUS-200]
adrs: [ADR-0001]
last_updated: 2025-11-26
---
<!-- doclint:disable orphan-version -->
<!-- ADR: This document contains historical version references as part of the decision record. -->

# ADR-0019: Governance Repository and FS Adapter

## Status

Accepted

## Context

The v2.5.0 kernel was designed as a read-only runtime for specifications. The `spec-runtime` crate loaded YAML files at startup, and the application served them via introspection APIs. However, to enable "Autonomous Software Engineering" (v3.0.0), the platform must be able to modify its own state (e.g., updating task status, creating acceptance criteria) in response to agent actions.

We needed a mechanism to persist these state changes durably while maintaining Git as the ultimate source of truth and avoiding the destruction of human-authored comments in the source YAML files.

## Decision

We decided to introduce a **Governance Repository** pattern with a **File System Adapter** that separates human-authored definitions from machine-managed state.

1. **GovernanceRepository Trait**: Defined in `business-core`, this trait abstracts the storage mechanism for governance entities (Tasks, Requirements, ACs). It decouples the domain logic from the persistence layer.

    ```rust
    pub trait GovernanceRepository: Send + Sync {
        fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError>;
    }
    ```

2. **adapters-spec-fs**: A new crate that implements `GovernanceRepository` using the local file system.
    * It reads/writes to `specs/tasks_state.yaml` for machine-managed state (status updates).
    * It uses `fs2` for file locking to ensure safe concurrent access (e.g., between the HTTP server and CLI commands).
    * It uses `serde_yaml` for serialization.

3. **State Separation**:
    * `specs/tasks.yaml`: Human-authored definitions (Title, Description, Links).
    * `specs/tasks_state.yaml`: Machine-managed state (Status, Assignee, Timestamps).
    * The runtime merges these views (future work for Sprint 2).

## Consequences

### Positive

* **GitOps Compatible**: All state changes are persisted to YAML, which can be committed to Git. This maintains a full audit trail and allows for rollbacks.
* **Safe Concurrency**: File locking prevents race conditions between multiple agents or tools.
* **Clean Architecture**: The core domain is isolated from file system details.
* **Non-Destructive**: Machine updates do not overwrite human-authored YAML files, preserving comments and formatting in the definitions.

### Negative

* **Complexity**: Requires merging two sources of truth (definitions + state) to get a complete view of a task.
* **Performance**: File I/O with locking is slower than in-memory or database storage, but acceptable for the scale of governance metadata.

## Async I/O Pattern

The `adapters-spec-fs` crate uses **synchronous** `std::fs` operations with `fs2` file locks. This is intentional: synchronous file I/O with file locks is simpler and more predictable than async file I/O with coordination primitives.

However, this creates a hazard when called from async HTTP handlers: blocking I/O on Tokio worker threads causes **executor starvation** under concurrent load. To address this (see GitHub Issue #14):

1. **`GovernanceRepository` trait methods are synchronous** (`fn`, not `async fn`).
2. **Async handlers use `tokio::task::spawn_blocking`** to offload calls to the sync repository.

### Canonical Pattern

```rust
// In async HTTP handler (e.g., app-http/src/tasks.rs)
async fn update_task_status(...) -> Result<impl IntoResponse, AppError> {
    let repo = state.governance_repo.clone();
    let task_id = TaskId(id);
    let new_status = payload.status;

    // Offload blocking file I/O (fs2 locks + std::fs) to spawn_blocking
    // to avoid starving the Tokio executor under concurrent load.
    tokio::task::spawn_blocking(move || {
        let service = TaskService::new(repo);
        service.move_task(&task_id, new_status)
    })
    .await
    .map_err(|e| AppError::internal_error(format!("spawn_blocking join: {e}")))?
    .map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
```

### Regression Test

The `async_io_regression.rs` test suite verifies that blocking I/O is properly offloaded. It uses a single-thread Tokio runtime where executor starvation is immediately observable: if blocking I/O occurs on the worker thread, concurrent timer tasks will hang.

## Compliance

* **REQ-TPL-GOV-WRITE-001**: The system can persist task status changes to machine-managed state.
