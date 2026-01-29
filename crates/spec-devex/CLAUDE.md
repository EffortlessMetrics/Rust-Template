# spec-devex – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde, serde_yaml

## Purpose

Developer experience flows and workflows parser. Handles `devex_flows.yaml` which defines developer workflows and xtask command specs.

## Key Exports

- DevEx flow parsing
- Workflow definitions
- Command specifications

## When to Modify

- Adding new flow parsing logic
- Extending workflow definitions

## When NOT to Modify

- Adding CLI implementations (those go in xtask)

## Key Files

- `specs/devex_flows.yaml` – Developer flows definition

## Consumers

`spec-runtime`

## See Also

- `specs/devex_flows.yaml` for file format
- `crates/xtask/` for flow implementations
