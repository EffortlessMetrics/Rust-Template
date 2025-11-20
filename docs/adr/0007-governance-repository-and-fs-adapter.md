# ADR-0007: Governance Repository and FS Adapter

## Status
Accepted

## Context
The v2.5.0 kernel was designed as a read-only runtime for specifications. The `spec-runtime` crate loaded YAML files at startup, and the application served them via introspection APIs. However, to enable "Autonomous Software Engineering" (v3.0.0), the platform must be able to modify its own state (e.g., updating task status, creating acceptance criteria) in response to agent actions.

We needed a mechanism to persist these state changes durably while maintaining Git as the ultimate source of truth and avoiding the destruction of human-authored comments in the source YAML files.

## Decision
We decided to introduce a **Governance Repository** pattern with a **File System Adapter** that separates human-authored definitions from machine-managed state.

1.  **GovernanceRepository Trait**: Defined in `business-core`, this trait abstracts the storage mechanism for governance entities (Tasks, Requirements, ACs). It decouples the domain logic from the persistence layer.
    ```rust
    pub trait GovernanceRepository: Send + Sync {
        fn set_task_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<(), GovernanceError>;
    }
    ```

2.  **adapters-spec-fs**: A new crate that implements `GovernanceRepository` using the local file system.
    *   It reads/writes to `specs/tasks_state.yaml` for machine-managed state (status updates).
    *   It uses `fs2` for file locking to ensure safe concurrent access (e.g., between the HTTP server and CLI commands).
    *   It uses `serde_yaml` for serialization.

3.  **State Separation**:
    *   `specs/tasks.yaml`: Human-authored definitions (Title, Description, Links).
    *   `specs/tasks_state.yaml`: Machine-managed state (Status, Assignee, Timestamps).
    *   The runtime merges these views (future work for Sprint 2).

## Consequences
### Positive
*   **GitOps Compatible**: All state changes are persisted to YAML, which can be committed to Git. This maintains a full audit trail and allows for rollbacks.
*   **Safe Concurrency**: File locking prevents race conditions between multiple agents or tools.
*   **Clean Architecture**: The core domain is isolated from file system details.
*   **Non-Destructive**: Machine updates do not overwrite human-authored YAML files, preserving comments and formatting in the definitions.

### Negative
*   **Complexity**: Requires merging two sources of truth (definitions + state) to get a complete view of a task.
*   **Performance**: File I/O with locking is slower than in-memory or database storage, but acceptable for the scale of governance metadata.

## Compliance
*   **REQ-TPL-GOV-WRITE-001**: The system can persist task status changes to machine-managed state.
