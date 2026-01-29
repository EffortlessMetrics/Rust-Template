# gov-model – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** Yes
**Dependencies:** Minimal (serde, thiserror)

## Purpose

Foundation crate defining governance domain types. Nearly all other crates depend on this—it is the "bottom of the stack."

## Key Exports

- `TaskStatus` – Enum with enforced state machine (Todo → InProgress → Review → Done)
- `Task`, `TaskId` – Work item with status
- `TaskService` – Service for managing task lifecycle
- `GovernanceRepository` – Trait for task persistence (adapters implement this)
- `RepoContext`, `SpecLayout` – Workspace path resolution for kernel crates
- `GovernanceError` – Domain error types

## When to Modify

- Adding new governance domain concepts
- Extending `TaskStatus` state machine
- Adding new paths to `RepoContext`

## When NOT to Modify

- Adding HTTP/CLI concerns (those go in app-http, xtask)
- Adding business logic (that goes in business-core)
- Adding infrastructure dependencies (breaks the foundation contract)

## Architectural Notes

- **Pure domain**: Zero infrastructure dependencies
- **State machine**: All `TaskStatus` transitions are validated
- **Repository pattern**: Defines ports, not implementations

## Consumers

`spec-runtime`, `business-core`, `gov-xtask-core`, `gov-receipts`, all adapters, `app-http`, `xtask`

## See Also

- `README.md` in this crate for full API documentation
- `crates/business-core/` for application-level business logic
