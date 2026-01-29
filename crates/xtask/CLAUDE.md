# xtask – CLAUDE.md

**Tier:** Build Tooling (Layer 7)
**Publish:** Yes
**Dependencies:** Nearly all other crates (it's the CLI orchestrator)

## Purpose

Main CLI for build, test, governance, and maintenance. Entry point for all `cargo xtask` commands.

## Key Commands

```bash
cargo xtask dev-up          # Bootstrap environment
cargo xtask check           # Fast checks (fmt, clippy, tests)
cargo xtask selftest        # Full 11-step governance gate
cargo xtask ac-status       # AC coverage report
cargo xtask test-ac AC-XXX  # Test specific AC
cargo xtask help-flows      # List developer flows
cargo xtask friction-list   # Show friction log
```

## When to Modify

- Adding new xtask commands
- Changing CLI output format
- Adding terminal colors/spinners

## When NOT to Modify

- Adding shared logic (put in xtask-lib or gov-xtask-core)
- Adding non-CLI functionality

## Architectural Notes

- **CLI entry point**: All terminal I/O here
- **Thin wrapper**: Delegates to library crates
- **User-facing**: Colors, progress, formatting

## Key Files

- `src/main.rs` – Entry point
- `src/commands/` – Individual commands

## Consumers

Developers, CI, agents (via shell)

## See Also

- `crates/xtask-lib/` for shared utilities
- `crates/gov-xtask-core/` for governance utilities
- `specs/devex_flows.yaml` for flow definitions
