# xtask-receipts – CLAUDE.md

**Tier:** Build Tooling (Layer 7)
**Publish:** Yes
**Dependencies:** receipts-core, gov-receipts, serde

## Purpose

Receipt generation and validation for xtask gates. Generates audit evidence for selftest and other gates.

## Key Exports

- Gate receipt generation
- Receipt validation
- Receipt persistence

## Key Files

- `target/receipts/` – Generated receipts

## When to Modify

- Adding new receipt types for gates
- Extending receipt validation

## Consumers

`xtask` (selftest, gates)

## See Also

- `crates/receipts-core/` for base receipt types
- `crates/gov-receipts/` for governance receipts
- `.claude/rules/45-semantic-only-merge.md` for field rules
