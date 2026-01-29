# business-core – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** model, gov-model, async-trait

## Purpose

Core business logic and domain services. Defines repository traits (ports) that adapters implement. Contains application-level business rules separate from governance.

## Key Exports

- `ExampleTaskRepository` – Trait for task persistence
- Service interfaces for domain operations
- Business rule implementations

## When to Modify

- Adding new repository traits
- Implementing business rules
- Adding service interfaces

## When NOT to Modify

- Adding database/HTTP code (those go in adapters)
- Adding governance logic (that goes in gov-model or ac-kernel)

## Architectural Notes

- **Ports & Adapters**: Defines traits, not implementations
- **Async-first**: Repository traits are async
- **Re-exports**: Some types re-exported for backward compatibility

## Consumers

`adapters-db-sqlx`, `adapters-spec-fs`, `app-http`, `http-tasks`

## See Also

- `crates/model/` for domain entities
- `crates/adapters-db-sqlx/` for database implementation
- `crates/adapters-spec-fs/` for filesystem implementation
