# spec-metadata – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde, serde_yaml

## Purpose

Spec metadata extraction and indexing. Extracts and indexes metadata from spec files for querying.

## Key Exports

- Metadata extraction functions
- Index building utilities

## When to Modify

- Adding new metadata fields
- Extending indexing logic

## Consumers

`spec-runtime`

## See Also

- `crates/spec-runtime/` for metadata usage
