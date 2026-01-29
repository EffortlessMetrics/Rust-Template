# spec-docs – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde, serde_yaml

## Purpose

Documentation inventory and indexing. Parses `doc_index.yaml` to track documentation files and their relationships to specs.

## Key Exports

- Doc index parsing
- Documentation inventory types
- Doc-to-spec linking

## When to Modify

- Adding new doc index fields
- Extending doc inventory logic

## Key Files

- `specs/doc_index.yaml` – Documentation index

## Consumers

`spec-runtime`

## See Also

- `specs/doc_index.yaml` for file format
- `docs/` directory for actual documentation
