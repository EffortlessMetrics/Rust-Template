# model

Shared domain models and data structures for the Rust-as-Spec platform.

## What It Is

`model` contains pure domain entities for the application layer. It defines core
business data structures separate from governance concerns.

### Key Exports

- `ExampleTask` -- Example domain entity demonstrating CRUD lifecycle
- Domain types for business operations

### What It Is Not

- **Not governance types**: Governance-specific types live in `gov-model`
- **Not business logic**: Application logic lives in `business-core`
- **Not data access**: Repository implementations live in adapters

## Design Principles

1. **Pure Domain**: No infrastructure dependencies
2. **CRUD Lifecycle**: Entities follow create/read/update patterns
3. **Separate from Governance**: Business domain vs governance domain split

## Consumers

| Consumer | Usage |
|----------|-------|
| `business-core` | Uses domain entities in business logic |
| `adapters-db-sqlx` | Persists domain entities |
| `http-tasks` | Serializes entities for HTTP responses |

## See Also

- [`gov-model/README.md`](../gov-model/README.md) - Governance-specific domain types
- [`business-core/README.md`](../business-core/README.md) - Business logic using these models

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.92.0.
