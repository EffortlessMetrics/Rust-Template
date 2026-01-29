# model – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** Yes
**Dependencies:** Minimal (serde, uuid, chrono)

## Purpose

Shared domain models and data structures for the application layer. Defines core business entities separate from governance concerns.

## Key Exports

- `ExampleTask` – Example domain entity demonstrating CRUD lifecycle
- Domain types for business operations

## When to Modify

- Adding new domain entities
- Extending existing entity fields
- Adding domain validation logic

## When NOT to Modify

- Adding HTTP/persistence concerns (those go in adapters)
- Adding governance-specific types (those go in gov-model)

## Architectural Notes

- **Domain layer**: Pure business entities
- **CRUD lifecycle**: Entities follow create/read/update patterns
- **Separate from governance**: Business domain vs governance domain split

## Consumers

`business-core`, `adapters-db-sqlx`, `adapters-spec-fs`, `http-tasks`, `http-todos`

## See Also

- `crates/gov-model/` for governance-specific domain types
- `crates/business-core/` for business logic using these models
