# spec-docs

Documentation index and staleness tracking.

## Purpose

This crate provides:
- Doc index structures
- Staleness detection logic
- Frontmatter validation
- Doc policy checking

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger for validation

## Design Principles

- **Minimal deps**: Only spec-types, serde, serde_yaml, thiserror, anyhow
- **View layer**: Depends on spec-ledger for validation
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types
- `DocIndex`: Documentation index structure
- `DocEntry`: Single documentation entry
- `DocPolicies`: Documentation policy specification
- `PolicyRule`: Policy rule for documentation requirements
- `AppliesTo`: What a policy rule applies to
- `StalenessReport`: Staleness report for documentation
- `StaleDoc`: A stale document that needs updating
- `MissingDoc`: A missing document that should exist

### Functions
- `load_doc_index(path)`: Load doc index from YAML file
- `load_policies(path)`: Load doc policies from YAML file
- `check_staleness(docs, ledger)`: Check documentation staleness
- `check_policies(docs, policies)`: Check documentation against policies

## Example

```rust
use spec_docs::{load_doc_index, check_staleness};

let docs = load_doc_index(Path::new("specs/doc_index.yaml"))?;
let report = check_staleness(&docs, &ledger)?;

for stale in report.stale_docs {
    eprintln!("Stale doc: {} - {}", stale.doc_id, stale.reason);
}
```
