# spec-types – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal)
**Dependencies:** Minimal (serde)

## Purpose

Stable spec ID newtypes. Provides type-safe identifiers for stories, requirements, and acceptance criteria.

## Key Exports

- `StoryId` – Newtype for story identifiers (e.g., "US-001")
- `RequirementId` – Newtype for requirement identifiers (e.g., "REQ-KERN-001")
- `AcId` – Newtype for acceptance criterion identifiers (e.g., "AC-KERN-001")

## When to Modify

- Adding new ID types for new spec entities
- Changing ID validation rules

## When NOT to Modify

- Adding parsing logic (that goes in spec-ledger)
- Adding HTTP serialization (that goes in gov-http-types)

## Architectural Notes

- **Newtype pattern**: Prevents mixing up different ID types at compile time
- **Minimal dependencies**: Only serde for serialization
- **Foundation contract**: Other spec-* crates depend on these types

## Consumers

`spec-ledger`, `spec-runtime`, `ac-kernel`, `gov-http-types`

## See Also

- `crates/spec-ledger/` for parsing specs that use these IDs
- `specs/spec_ledger.yaml` for the actual spec file format
