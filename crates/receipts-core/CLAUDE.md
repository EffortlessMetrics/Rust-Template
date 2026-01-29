# receipts-core – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal)
**Dependencies:** Minimal (serde, chrono)

## Purpose

Core receipt types for audit evidence and quality tracking. Receipts are immutable records of actions taken and their outcomes.

## Key Exports

- `Receipt` – Base receipt type
- Audit trail types
- Evidence linking types

## When to Modify

- Adding new receipt categories
- Extending evidence metadata

## When NOT to Modify

- Adding governance-specific receipt logic (that goes in gov-receipts)
- Adding file I/O (receipts are pure data)

## Architectural Notes

- **Immutable records**: Receipts capture point-in-time evidence
- **Audit trail**: Used by selftest, PR validation, and quality gates
- **Foundation**: gov-receipts and xtask-receipts build on this

## Consumers

`gov-receipts`, `xtask-receipts`, `xtask`

## See Also

- `crates/gov-receipts/` for governance-specific receipts
- `crates/xtask-receipts/` for xtask gate receipts
- `.claude/rules/45-semantic-only-merge.md` for LLM receipt field rules
