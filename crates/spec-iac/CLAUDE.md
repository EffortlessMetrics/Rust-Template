# spec-iac – CLAUDE.md

**Tier:** Spec Processor (Layer 3)
**Publish:** No (internal)
**Dependencies:** serde, serde_yaml

## Purpose

Kubernetes IaC configuration support. Handles infrastructure-as-code specs for Kubernetes deployments.

## Key Exports

- K8s IaC parsing
- Infrastructure spec types

## When to Modify

- Adding new IaC spec types
- Extending K8s configuration support

## Consumers

`spec-runtime`, `rust_iac_config`

## See Also

- `crates/rust_iac_config/` for IaC configuration
- `specs/` for IaC spec files
