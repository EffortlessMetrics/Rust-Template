# spec-types

Stable, dependency-light types for spec-related operations (ledger, graph, hints, tasks).

## Purpose

This crate defines the externally-consumed shapes and semantics for spec-related operations. It contains:

- **ID newtypes**: Type-safe identifiers for Story, Requirement, AC, Task
- **Shared structs**: Common data structures from spec_ledger.yaml
- **Path types**: Type-safe file path representations
- **Common error types**: Errors for spec operations

## Design Philosophy

- **Minimal dependencies**: Only `serde`, `serde_yaml`, `thiserror`
- **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
- **Internal-only**: `publish = false` - this is a contract crate for internal use

## Public API

### ID Newtypes

- [`StoryId`] - Type-safe story identifier
- [`RequirementId`] - Type-safe requirement identifier
- [`AcId`] - Type-safe acceptance criterion identifier
- [`TaskId`] - Type-safe task identifier

All ID types implement:
- `new(id: impl Into<String>)` - Constructor
- `as_str() -> &str` - Get inner string value
- `Display` - String representation
- `AsRef<str>` - Borrow as string

### Shared Structs

- [`Story`] - Story with title and requirements
- [`Requirement`] - Requirement with title, ACs, and optional `must_have_ac`
- [`AcceptanceCriterion`] - AC with text, optional `must_have_ac`, test mappings, and tags
- [`TestMapping`] - Test type and optional tag from spec ledger
- [`Task`] - Task with ID, title, summary, status, requirement, ACs, owner, and labels

### Path Types

- [`SpecPath`] - Type-safe file path within specs directory

### Ledger Metadata

- [`LedgerMetadata`] - Template version from spec_ledger.yaml
- [`SpecLedger`] - Complete ledger with metadata and stories

### Error Types

- [`SpecError`] - Error enum for spec operations (Io, YamlParse, Validation, NotFound, InvalidFormat)
- [`SpecResult<T>`` - Result type alias for spec operations

## Stability Guarantees

This crate follows semantic versioning. Breaking changes to public types will result in a major version bump.

### Stable Types

The following types are considered stable and will not have breaking changes in minor versions:

- ID newtypes (`StoryId`, `RequirementId`, `AcId`, `TaskId`) - inner string accessors are stable
- `SpecPath` - string wrapper is stable
- `SpecError` enum variants (new variants may be added via `#[non_exhaustive]`)

### Evolving Types

Types marked with `#[non_exhaustive]` may receive new fields in future versions:

- `Story` - new optional fields may be added
- `Requirement` - new optional fields may be added
- `AcceptanceCriterion` - new optional fields may be added
- `Task` - new optional fields may be added
- `LedgerMetadata` - new optional fields may be added

## Usage Example

```rust
use spec_types::{StoryId, RequirementId, AcId, SpecPath};

// Create type-safe IDs
let story_id = StoryId::new("US-PLT-001");
let req_id = RequirementId::new("REQ-PLT-001");
let ac_id = AcId::new("AC-PLT-001");

// Create a spec path
let ledger_path = SpecPath::new("specs/spec_ledger.yaml");

// Use in error handling
use spec_types::{SpecError, SpecResult};

fn load_spec() -> SpecResult<spec_types::SpecLedger> {
    let content = std::fs::read_to_string(ledger_path.as_str())?;
    serde_yaml::from_str(&content).map_err(|e| SpecError::YamlParse(e.to_string()))
}
```

## Migration Notes

When upgrading this crate:

1. Review changelog for breaking changes
2. Update any pattern matching on `SpecError` enum
3. Handle new optional fields in DTO structures
4. Re-run `cargo check` to catch any API mismatches
