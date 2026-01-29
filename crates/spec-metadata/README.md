# spec-metadata

Spec metadata extraction and management.

## Purpose

This crate provides:
- Metadata types
- Metadata extraction
- Metadata validation

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger for validation

## Design Principles

- **Minimal deps**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
- **Utility layer**: Provides metadata types and extraction helpers
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types

- `SpecMetadata`: Spec metadata specification
- `LedgerMetadata`: Ledger metadata
- `TagsMetadata`: Tags metadata
- `DocumentationMetadata`: Documentation metadata
- `TagExtraction`: Tag extraction result
- `MetadataValidationResult`: Metadata validation result
- `MetadataValidationError`: Metadata validation error

### Functions

- `load_metadata(path)`: Load spec metadata from YAML file
- `extract_tags(ledger)`: Extract tags from spec ledger
- `build_ledger_metadata(ledger)`: Build ledger metadata
- `validate_metadata(metadata, ledger)`: Validate spec metadata

## Example

```rust
use spec_metadata::{load_metadata, extract_tags};

let metadata = load_metadata(Path::new("specs/metadata.yaml"))?;
let tags = extract_tags(&ledger);

for tag in tags.tags {
    println!("Tag: {}", tag);
}
```
