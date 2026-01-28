# spec-tasks

Task and hint management.

## Purpose

This crate provides:
- Task types and structures
- Hint types and structures
- Task resolution logic
- Hint scoring

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger for validation

## Design Principles

- **Minimal deps**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
- **View layer**: Depends on spec-ledger for validation
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types
- `TasksSpec`: Tasks specification
- `Task`: A work item/task
- `TaskDocs`: Documentation references for a task
- `SuggestedSequence`: Suggested sequence of actions for a task
- `StepStatus`: Step status in suggested sequence
- `Action`: An action in a suggested sequence
- `ReferentialWarning`: Referential warning for task validation
- `TaskGraph`: Task dependency graph
- `TaskNode`: Node in task graph
- `TaskEdge`: Edge in task graph
- `BlockingRelationship`: Blocking relationship between tasks

### Functions
- `load_tasks(path)`: Load tasks specification from YAML file
- `validate_task_references(tasks, ledger)`: Validate task AC/REQ references
- `build_task_graph(tasks_spec)`: Build a task dependency graph

## Example

```rust
use spec_tasks::{load_tasks, validate_task_references};

let tasks = load_tasks(Path::new("specs/tasks.yaml"))?;
let warnings = validate_task_references(&tasks, &ledger)?;

for warning in warnings {
    eprintln!("Warning: {}", warning.message);
}
```
