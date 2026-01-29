# spec-schema

Schema generation (OpenAPI, JSON schema).

## Purpose

This crate provides:
- Schema generation functions
- OpenAPI generation
- JSON schema generation
- Schema validation helpers

**This is where jsonschema lives - don't leak it elsewhere.**

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger for schema generation
- **serde_json**: JSON schema generation

## Design Principles

- **Heavy deps isolated**: This crate contains jsonschema and other heavy dependencies
- **Foundation**: Depends on spec-types and spec-ledger
- **No axum**: HTTP/web dependencies are isolated to higher-level crates

## Public API

### Types

- `PlatformSchemas`: Complete platform schema information
- `SchemaInfo`: Information about a specific schema
- `EndpointSchema`: API endpoint schema information

### Functions

- `get_all_schemas()`: Get all platform schemas
- `get_schema_by_name(name)`: Get schema by name

## Example

```rust
use spec_schema::{get_all_schemas, get_schema_by_name};

let schemas = get_all_schemas();
let ledger_schema = get_schema_by_name("spec_ledger").unwrap();

println!("Schema: {} - {}", ledger_schema.name, ledger_schema.description);
```
