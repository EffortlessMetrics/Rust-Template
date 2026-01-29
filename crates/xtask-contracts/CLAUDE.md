# xtask-contracts – CLAUDE.md

**Tier:** Build Tooling (Layer 7)
**Publish:** Yes
**Dependencies:** serde

## Purpose

Contract validation for xtask commands. Validates that command inputs/outputs conform to expected contracts.

## Key Exports

- Contract validation functions
- Contract types
- Validation error types

## When to Modify

- Adding new contract validations
- Extending contract types

## Consumers

`xtask`

## See Also

- `crates/xtask-contract/` for contract type definitions
- `crates/xtask/` for command implementations
