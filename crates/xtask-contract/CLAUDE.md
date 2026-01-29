# xtask-contract – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal)
**Dependencies:** Minimal (serde)

## Purpose

Contract types for xtask execution and CLI integration. Defines the interface between xtask commands and the rest of the system.

## Key Exports

- Task execution request/response types
- CLI command contracts
- Exit code semantics

## When to Modify

- Adding new xtask command contracts
- Changing CLI interface semantics

## When NOT to Modify

- Adding command implementations (those go in xtask)
- Adding infrastructure dependencies

## Architectural Notes

- **Contract-first**: Defines what xtask commands accept and return
- **Minimal dependencies**: Pure data structures

## Consumers

`xtask`, `xtask-lib`, `gov-xtask-core`

## See Also

- `crates/xtask/` for command implementations
- `crates/xtask-lib/` for shared xtask utilities
