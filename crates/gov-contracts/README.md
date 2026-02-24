# gov-contracts

Governance contracts and schemas for the Rust-as-Spec platform.

## What It Is

`gov-contracts` defines the contracts that governance artifacts must conform to.
It provides the structural definitions and validation schemas that ensure
governance data is well-formed and versioned.

### Key Exports

- Contract type definitions
- Schema validation types
- Contract versioning

### What It Is Not

- **Not policy evaluation**: Policy enforcement lives in `gov-policy`
- **Not governance types**: Core types live in `gov-model`

## Design Principles

1. **Contract-First**: Defines what governance artifacts look like
2. **Versioned**: Contracts have explicit versions
3. **Stable API**: Breaking changes require a major version bump

## Consumers

| Consumer | Usage |
|----------|-------|
| `gov-policy` | Evaluates policies against contracts |
| `xtask` | Validates governance artifacts |

## See Also

- [`gov-policy/README.md`](../gov-policy/README.md) - Policy evaluation using these contracts
- [`gov-model/README.md`](../gov-model/README.md) - Core governance domain types

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.92.0.
