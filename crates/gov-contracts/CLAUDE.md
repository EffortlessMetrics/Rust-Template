# gov-contracts – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** serde

## Purpose

Governance contracts and schemas. Defines the contracts that governance artifacts must conform to.

## Key Exports

- Contract type definitions
- Schema validation types
- Contract versioning

## When to Modify

- Adding new contract types
- Evolving existing contracts (with versioning)

## When NOT to Modify

- Adding contract validation logic (that goes in gov-policy)
- Changing contracts without version bump (breaking)

## Architectural Notes

- **Contract-first**: Defines what governance artifacts look like
- **Versioned**: Contracts have explicit versions
- **Stable API**: Breaking changes require major version

## Consumers

`gov-policy`, `xtask`, governance validation

## See Also

- `crates/gov-policy/` for policy evaluation
- `specs/` for actual governance artifacts
