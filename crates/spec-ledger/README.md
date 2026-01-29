# spec-ledger

Parse, load, and index the spec ledger (`spec_ledger.yaml`).

## Purpose

This crate provides the foundation layer for spec-related operations. It handles:
- Parsing YAML ledger files
- Ledger validation (ledger-specific invariants)
- Index structures for fast lookup
- Path resolution helpers

## Dependencies

- **spec-types**: Foundation types (ID newtypes, error types)
- **serde**: Serialization/deserialization
- **serde_yaml**: YAML parsing
- **thiserror**: Error handling
- **anyhow**: Error context

## Design Principles

- **Foundation**: All other spec-* crates depend on this, not vice versa
- **Minimal deps**: No jsonschema, no axum, no heavy dependencies
- **Fast lookup**: Provides index structures for O(1) ID lookups

## Public API

### Types

- `SpecLedger`: Root ledger structure
- `Metadata`: Ledger metadata
- `Story`: User story with requirements
- `Requirement`: Requirement with acceptance criteria
- `AcceptanceCriterion`: AC with test mappings
- `TestMapping`: Test reference

### Index Types

- `AcIdIndex`: HashSet of AC IDs for fast lookup
- `ReqIdIndex`: HashSet of REQ IDs for fast lookup

### Functions

- `load_spec_ledger(path)`: Load ledger from YAML file
- `build_ac_id_index(ledger)`: Build AC ID index
- `build_req_id_index(ledger)`: Build REQ ID index
- `validate_ledger(ledger)`: Validate ledger invariants

## Example

```rust
use spec_ledger::{load_spec_ledger, build_ac_id_index};

let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
let ac_index = build_ac_id_index(&ledger);

assert!(ac_index.contains("AC-TPL-001"));
```
