# rust_iac_config – CLAUDE.md

**Tier:** IaC Support (Layer 8)
**Publish:** Yes
**Dependencies:** serde, serde_yaml

## Purpose

YAML-based infrastructure configuration with validation. Handles IaC configuration files for deployment.

## Key Exports

- IaC configuration types
- YAML loading and validation
- Environment-specific config

## When to Modify

- Adding new IaC configuration options
- Extending validation rules

## Architectural Notes

- **Configuration as code**: YAML-based
- **Validated**: Schema validation on load
- **Environment-aware**: Supports multiple environments

## Consumers

Deployment tooling, `spec-iac`

## See Also

- `crates/spec-iac/` for spec-level IaC
- `crates/rust_iac_xtask_core/` for IaC xtask support
