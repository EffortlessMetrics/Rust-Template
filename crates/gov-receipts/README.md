# gov-receipts

Receipt types for audit evidence in the Rust-as-Spec platform.

## What It Is

`gov-receipts` defines versioned receipt types for governance artifacts. Receipts
provide structured, machine-parseable audit evidence that governance gates have
been evaluated and their outcomes recorded.

### Key Exports

- Governance receipt schemas
- Version tracking for receipt format evolution
- Schema validation helpers

### What It Is Not

- **Not base receipt types**: Generic receipt infrastructure lives in `receipts-core`
- **Not receipt generation**: Receipt creation logic lives in `xtask` and `gov-xtask-core`

## Design Principles

1. **Versioned Schemas**: Each receipt type has an explicit version
2. **Backward Compatibility**: Old receipts remain parseable
3. **Governance-Specific**: Only governance audit evidence

## Consumers

| Consumer | Usage |
|----------|-------|
| `xtask` | Generates receipts during governance gates |
| `gov-xtask-core` | Uses receipt types for quality evidence |

## See Also

- [Semantic-Only Merge Rule](../../.claude/rules/45-semantic-only-merge.md) - Field separation rules for receipts
- `target/receipts/` - Generated receipt files

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.89.0.
