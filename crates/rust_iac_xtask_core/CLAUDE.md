# rust_iac_xtask_core – CLAUDE.md

**Tier:** IaC Support (Layer 8)
**Publish:** Yes
**Dependencies:** rust_iac_config, anyhow

## Purpose

Governance and xtask orchestration for IaC projects. Provides utilities for infrastructure-as-code workflows.

## Key Exports

- IaC workflow utilities
- Governance integration for IaC

## When to Modify

- Adding IaC-specific xtask utilities
- Extending IaC governance

## Consumers

`xtask` (IaC commands), deployment workflows

## See Also

- `crates/rust_iac_config/` for configuration
- `crates/xtask/` for CLI integration
