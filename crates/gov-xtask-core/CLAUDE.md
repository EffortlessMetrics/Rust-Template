# gov-xtask-core – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** gov-model, spec-runtime, anyhow

## Purpose

Shared governance utilities for xtask commands. Provides common functionality used by multiple xtask commands without the CLI concerns.

## Key Exports

- Governance workflow utilities
- RepoContext helpers
- Common governance operations

## When to Modify

- Adding utilities needed by multiple xtask commands
- Refactoring common governance patterns

## When NOT to Modify

- Adding CLI output formatting (that goes in xtask)
- Adding command implementations (those go in xtask)

## Architectural Notes

- **Library, not CLI**: Pure utilities without terminal I/O
- **Shared logic**: Prevents duplication across xtask commands
- **RepoContext-aware**: Uses consistent path resolution

## Consumers

`xtask`, `xtask-lib`

## See Also

- `crates/xtask/` for command implementations
- `crates/xtask-lib/` for additional utilities
