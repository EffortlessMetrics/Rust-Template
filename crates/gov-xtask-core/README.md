# gov-xtask-core

Governance xtask core utilities for the Rust-as-Spec platform.

## What It Is

`gov-xtask-core` provides shared governance utilities used by multiple xtask
commands. It contains common functionality extracted from xtask to avoid
duplication, without any CLI or terminal I/O concerns.

### Key Exports

- Governance workflow utilities
- RepoContext helpers
- Common governance operations

### What It Is Not

- **Not a CLI**: Command-line interfaces and formatting live in `xtask`
- **Not command implementations**: Individual commands live in `xtask`

## Design Principles

1. **Library, Not CLI**: Pure utilities without terminal I/O
2. **Shared Logic**: Prevents duplication across xtask commands
3. **RepoContext-Aware**: Uses consistent path resolution via `gov-model::RepoContext`

## Consumers

| Consumer | Usage |
|----------|-------|
| `xtask` | Uses shared utilities for command implementations |
| `xtask-lib` | Additional utility layer |

## See Also

- [`gov-model/README.md`](../gov-model/README.md) - RepoContext and governance types
- [`spec-runtime/README.md`](../spec-runtime/README.md) - Spec loading used by these utilities

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.92.0.
