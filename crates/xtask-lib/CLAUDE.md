# xtask-lib – CLAUDE.md

**Tier:** Build Tooling (Layer 7)
**Publish:** Yes
**Dependencies:** gov-xtask-core, anyhow

## Purpose

Shared xtask utilities without CLI concerns. Pure library code used by xtask commands.

## Key Exports

- Task automation utilities
- Build helpers
- Common operations

## When to Modify

- Adding utilities needed by xtask commands
- Refactoring common patterns

## When NOT to Modify

- Adding CLI output (that goes in xtask)
- Adding governance-specific logic (that goes in gov-xtask-core)

## Architectural Notes

- **Library, not CLI**: No terminal I/O
- **Reusable**: Can be used by other tools

## Consumers

`xtask`

## See Also

- `crates/xtask/` for CLI commands
- `crates/gov-xtask-core/` for governance utilities
