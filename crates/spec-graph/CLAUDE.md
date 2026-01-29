# spec-graph – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** spec-types, spec-ledger, serde

## Purpose

Governance graph builder. Constructs queryable graph of relationships: stories → requirements → ACs → tests → docs.

## Key Exports

- `Graph` – The governance graph structure
- `Node`, `Edge` – Graph primitives
- Graph building functions

## Node Types

Story, Requirement, AC, Test, Doc, Command, Flow

## Edge Types

has_requirement, has_ac, has_test, documented_by, implements

## When to Modify

- Adding new node or edge types
- Extending graph queries
- Adding graph analysis functions

## When NOT to Modify

- Adding graph visualization (that goes in app-http or CLI)

## Consumers

`spec-runtime`, `gov-http` (for `/platform/graph`)

## See Also

- `crates/spec-runtime/` for graph loading via `build_graph()`
- `/platform/graph` endpoint for API access
