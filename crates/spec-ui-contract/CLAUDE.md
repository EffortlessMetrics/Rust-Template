# spec-ui-contract – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde

## Purpose

UI surface contracts with stable identifiers. Defines contracts for UI elements that external systems may depend on.

## Key Exports

- UI contract types
- Stable UI identifiers
- Contract validation

## When to Modify

- Adding new UI surface contracts
- Evolving existing contracts (with versioning)

## When NOT to Modify

- Changing identifiers without versioning (breaking change)

## Architectural Notes

- **Stable identifiers**: UI elements have stable IDs for external reference
- **Contract-first**: Defines what UI surfaces look like

## Consumers

`spec-runtime`, external UI integrations

## See Also

- `specs/ui_contract.yaml` for UI contract definitions
