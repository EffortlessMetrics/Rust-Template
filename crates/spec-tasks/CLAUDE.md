# spec-tasks – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** gov-model, serde, serde_yaml

## Purpose

Work item tracking and sequencing from specs. Parses `tasks.yaml` to load work items and their dependencies.

## Key Exports

- Task loading functions
- Task dependency tracking
- Work item sequencing

## When to Modify

- Adding new task fields
- Extending dependency logic

## Key Files

- `specs/tasks.yaml` – Work items definition

## Consumers

`spec-runtime`, `http-tasks`

## See Also

- `specs/tasks.yaml` for file format
- `crates/gov-model/` for TaskStatus types
