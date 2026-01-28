# spec-ui-contract

UI-specific contract types for governed platform UI surfaces.

## Purpose

This crate provides:
- UI-specific DTOs
- UI schema types
- UI validation rules

## Dependencies

- **spec-types**: Foundation types
- **platform-contract**: Contract-grade stability

## Design Principles

- **Minimal deps**: Only spec-types, platform-contract, serde, serde_yaml, thiserror, anyhow
- **Contract-grade stability**: UI-specific DTOs with stable API
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types
- `UiContract`: Top-level UI contract specification
- `Screen`: A UI screen definition
- `Region`: A region within a UI screen

### Functions
- `load_ui_contract(path)`: Load UI contract from YAML file
- `validate_region_kinds(contract)`: Validate that all region kinds reference valid kind definitions
- `all_region_ids(contract)`: Get all region IDs across all screens
- `region_ids_for_screen(contract, screen_id)`: Get region IDs for a specific screen

## Example

```rust
use spec_ui_contract::{load_ui_contract, all_region_ids};

let contract = load_ui_contract(Path::new("specs/ui_contract.yaml"))?;
let regions = all_region_ids(&contract);

for region_id in regions {
    println!("Region: {}", region_id);
}
```
