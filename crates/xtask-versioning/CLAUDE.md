# xtask-versioning – CLAUDE.md

**Tier:** Build Tooling (Layer 7)
**Publish:** Yes
**Dependencies:** semver, toml_edit

## Purpose

Version management and bumping. Handles semantic versioning for the workspace.

## Key Exports

- Version bumping functions
- Cargo.toml manipulation
- Version validation

## When to Modify

- Changing versioning strategy
- Adding version synchronization logic

## Consumers

`xtask` (release commands)

## See Also

- `crates/xtask/` for release commands
- Root `Cargo.toml` for workspace version
