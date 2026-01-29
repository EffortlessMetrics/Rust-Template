# adapters-db-sqlx – CLAUDE.md

**Tier:** Adapter (Layer 4)
**Publish:** Yes
**Dependencies:** business-core, model, sqlx, tokio

## Purpose

SQLx-based PostgreSQL adapter. Implements repository traits from business-core for database persistence.

## Key Exports

- Repository trait implementations
- Database connection management
- Migration support

## When to Modify

- Adding new database operations
- Implementing new repository traits
- Adding/modifying migrations

## When NOT to Modify

- Changing repository trait definitions (those go in business-core)
- Adding business logic (that goes in business-core)

## Architectural Notes

- **Ports & Adapters**: Implements traits from business-core
- **SQLx**: Compile-time checked SQL queries
- **PostgreSQL**: Primary database target
- **Migrations**: Embedded migrations in crate

## Key Files

- `migrations/` for database migrations

## Consumers

`app-http` (production database)

## See Also

- `crates/business-core/` for repository trait definitions
- `crates/adapters-spec-fs/` for filesystem alternative
