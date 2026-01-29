# spec-schema – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde, serde_yaml, jsonschema

## Purpose

Schema handling and validation for spec files. Validates YAML specs against their JSON schemas.

## Key Exports

- Schema validation functions
- Schema loading utilities
- Validation error types

## When to Modify

- Adding new schema validation rules
- Supporting new spec file types

## When NOT to Modify

- Adding spec parsing (that goes in spec-ledger, spec-devex, etc.)

## Architectural Notes

- **JSON Schema**: Uses JSON Schema for validation
- **Cross-cutting**: Used by multiple spec parsers

## Consumers

`spec-runtime`, `spec-ledger`

## See Also

- `specs/config_schema.yaml` for config schema
- `crates/spec-runtime/` for schema usage
