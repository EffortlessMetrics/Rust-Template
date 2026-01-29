# spec-ledger – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** spec-types, serde, serde_yaml

## Purpose

Spec ledger YAML parsing and validation. Handles the stories → requirements → ACs hierarchy from `spec_ledger.yaml`.

## Key Exports

- Ledger parsing functions
- Story, Requirement, AC structs
- Validation helpers

## When to Modify

- Changing ledger parsing logic
- Adding new ledger fields
- Extending validation rules

## When NOT to Modify

- Adding spec-runtime orchestration (that's in spec-runtime)
- Adding graph building (that's in spec-graph)

## Architectural Notes

- **Single responsibility**: Only parses spec_ledger.yaml
- **Used by spec-runtime**: Not typically called directly

## Key Files

- `specs/spec_ledger.yaml` – The spec ledger being parsed

## Consumers

`spec-runtime`

## See Also

- `crates/spec-runtime/` for orchestration
- `specs/spec_ledger.yaml` for file format
