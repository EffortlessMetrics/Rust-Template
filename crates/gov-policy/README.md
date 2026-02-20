# gov-policy

Rego policy bundle and runner for the Rust-as-Spec platform.

## What It Is

`gov-policy` provides policy evaluation for governance validation. It loads
and evaluates Rego policy bundles against platform state to enforce governance
rules declaratively.

### Key Exports

- Policy evaluation functions
- Rego bundle loading
- Policy violation types

### What It Is Not

- **Not contract definitions**: Contract schemas live in `gov-contracts`
- **Not CLI commands**: Command-line interfaces live in `xtask`
- **Not HTTP endpoints**: Web serving logic lives in `gov-http`

## Design Principles

1. **Declarative Rules**: Policies are data, not code
2. **Rego-Based**: Uses Open Policy Agent policy language
3. **Bundled**: Policies ship with the crate

## Consumers

| Consumer | Usage |
|----------|-------|
| `xtask` | Evaluates policies during `selftest` |
| `gov-http` | Exposes policy results via HTTP |

## See Also

- [`gov-contracts/README.md`](../gov-contracts/README.md) - Contract definitions evaluated by policies
- [Open Policy Agent](https://www.openpolicyagent.org/) - Policy language documentation

## Stability

This crate is part of the **rust-as-spec** governance kernel.
Version numbers track the kernel release (currently 3.3.15).
Breaking changes require a major version bump and an ADR.
MSRV: 1.89.0.
