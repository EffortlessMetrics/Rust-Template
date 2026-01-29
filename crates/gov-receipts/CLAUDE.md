# gov-receipts – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** Yes
**Dependencies:** receipts-core, serde, chrono

## Purpose

Versioned receipt types for governance artifacts. Extends receipts-core with governance-specific schema and versioning.

## Key Exports

- Governance receipt schemas
- Version tracking for receipt format evolution
- Schema validation helpers

## When to Modify

- Adding new governance receipt types
- Evolving receipt schemas (with version bump)

## When NOT to Modify

- Changing receipt schemas without version bump (breaking)
- Adding non-governance receipt types (those go in receipts-core)

## Architectural Notes

- **Versioned schemas**: Each receipt type has explicit version
- **Backward compatibility**: Old receipts remain parseable
- **Governance-specific**: Only governance audit evidence

## Consumers

`xtask`, `gov-xtask-core`, quality gates

## See Also

- `crates/receipts-core/` for base receipt types
- `.claude/rules/45-semantic-only-merge.md` for field separation rules
- `target/receipts/` for generated receipt files
