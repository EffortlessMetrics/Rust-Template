# adapters-spec-fs – CLAUDE.md

**Tier:** Adapter (Layer 4)
**Publish:** No (internal)
**Dependencies:** business-core, model, serde, tokio

## Purpose

Filesystem adapter for spec and business object storage. Implements repository traits using file-based persistence (YAML/JSON files).

## Key Exports

- Repository trait implementations (filesystem-backed)
- File-based storage utilities

## When to Modify

- Adding new filesystem operations
- Implementing new repository traits for file storage
- Changing serialization format

## When NOT to Modify

- Changing repository trait definitions (those go in business-core)

## Architectural Notes

- **Ports & Adapters**: Implements traits from business-core
- **File-based**: Good for local development, testing
- **YAML/JSON**: Human-readable storage format

## Consumers

Local development, testing, `xtask`

## See Also

- `crates/business-core/` for repository trait definitions
- `crates/adapters-db-sqlx/` for database alternative
