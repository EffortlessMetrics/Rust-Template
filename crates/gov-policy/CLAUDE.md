# gov-policy – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** serde, anyhow

## Purpose

Rego policy bundle and runner for platform policy validation. Evaluates governance policies against platform state.

## Key Exports

- Policy evaluation functions
- Rego bundle loading
- Policy violation types

## When to Modify

- Adding new policy rules
- Extending policy evaluation
- Adding new violation types

## When NOT to Modify

- Adding HTTP endpoints (those go in gov-http)
- Adding CLI commands (those go in xtask)

## Architectural Notes

- **Rego policies**: Uses Open Policy Agent policy language
- **Declarative rules**: Policies are data, not code
- **Bundled**: Policies ship with the crate

## Key Files

- `policy/` directory for Rego files
- Policy bundles in crate resources

## Consumers

`xtask` (selftest), `gov-http`, governance gates

## See Also

- Open Policy Agent documentation
- `cargo xtask selftest` for policy evaluation
