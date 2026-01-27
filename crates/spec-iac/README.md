# spec-iac

IaC-related spec operations.

## Purpose

This crate provides:
- IaC spec types
- IaC generation helpers
- IaC validation

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger for validation

## Design Principles

- **Minimal deps**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
- **Workflow layer**: Provides IaC spec types and generation helpers
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types
- `IacSpec`: IaC specification
- `IacService`: IaC service specification
- `IacResource`: IaC resource specification
- `K8sManifest`: K8s manifest
- `K8sService`: K8s Service
- `K8sConfigMap`: K8s ConfigMap
- `IacValidationResult`: IaC validation result
- `IacValidationError`: IaC validation error

### Functions
- `load_iac_spec(path)`: Load IaC specification from YAML file
- `generate_k8s_manifest(iac)`: Generate K8s manifest from IaC spec
- `generate_k8s_service(service)`: Generate K8s Service manifest
- `validate_iac_spec(iac)`: Validate IaC specification

## Example

```rust
use spec_iac::{load_iac_spec, generate_k8s_manifest};

let iac = load_iac_spec(Path::new("specs/iac.yaml"))?;
let manifest = generate_k8s_manifest(&iac)?;

println!("Generated K8s manifest for {} services", iac.services.len());
```
