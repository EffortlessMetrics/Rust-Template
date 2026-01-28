# xtask-contract

Stable, dependency-light contract types for xtask `--json` output and exit codes.

## Purpose

This crate defines the externally-consumed shapes and semantics for xtask command JSON output. It contains:

- **JSON output DTOs**: Data transfer objects for commands like `ac-status --json`, `issues-search --json`
- **Exit code enums**: Stable exit codes for CLI behavior
- **Status enums**: Type-safe status values for ACs, questions, friction
- **Common JSON response envelopes**: Standardized wrapper for JSON output

## Design Philosophy

- **Minimal dependencies**: Only `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`
- **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
- **Internal-only**: `publish = false` - this is a contract crate for internal use only

## Public API

### Exit Codes

- [`ExitCode`] - CLI exit codes (Success, Error, InvalidArguments, FileNotFound, ValidationError, NetworkError)

### Status Enums

- [`AcStatus`] - AC status (Pass, Fail, Unknown)
- [`QuestionStatus`] - Question status (Open, Answered, Resolved, Obsolete)
- [`FrictionStatus`] - Friction status (Open, Investigating, InProgress, Resolved, WontFix)

### AC Coverage JSON DTOs

- [`AcStatusJson`] - Complete AC status report
- [`AcCategoryStats`] - AC category statistics
- [`KernelStats`] - Kernel AC statistics
- [`OptionalStats`] - Optional AC statistics
- [`AcJson`] - Individual AC representation
- [`TestMapping`] - Test mapping from spec ledger

### Issues Search JSON DTOs

- [`SearchResult`] - Unified search result (friction, question, or task)
- [`SearchOutput`] - Complete search output with query and results

### Friction JSON DTOs

- [`FrictionEntry`] - Complete friction entry
- [`FrictionContext`] - Friction discovery context
- [`Resolution`] - Resolution details
- [`RelatedItems`] - Related issues, ADRs, tasks
- [`FrictionListJson`] - Friction list JSON output
- [`FrictionStatsJson`] - Friction statistics
- [`FrictionSeverityStatsJson`] - Severity breakdown

### Question JSON DTOs

- [`Question`] - Complete question entry
- [`QuestionContext`] - Question flow/phase context
- [`QuestionOption`] - Question option with risk assessment
- [`Recommendation`] - Recommended option with rationale
- [`QuestionsListJson`] - Question list JSON output
- [`QuestionStatsJson`] - Question statistics

### Common Response Envelope

- [`JsonEnvelope<T>`] - Generic JSON envelope with schema version and timestamp

## Stability Guarantees

This crate follows semantic versioning. Breaking changes to public types will result in a major version bump.

### Stable Types

The following types are considered stable and will not have breaking changes in minor versions:

- `ExitCode` enum variants (new variants may be added via `#[non_exhaustive]`)
- `AcStatus` enum variants
- `QuestionStatus` enum variants
- `FrictionStatus` enum variants
- `AcStatusJson` structure
- `SearchResult` structure
- `JsonEnvelope<T>` generic structure

### Evolving Types

Types marked with `#[non_exhaustive]` may receive new fields in future versions:

- `FrictionEntry` - new optional fields may be added
- `Question` - new optional fields may be added
- `AcJson` - new optional fields may be added

## Usage Example

```rust
use xtask_contract::{AcStatusJson, JsonEnvelope, AcStatus};

// Create AC status JSON output
let ac_status = AcStatusJson {
    schema_version: "1.0".to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
    summary: xtask_contract::AcCategoryStats {
        total: 10,
        passing: 8,
        failing: 1,
        unknown: 1,
        kernel: xtask_contract::KernelStats {
            total: 5,
            passing: 4,
            failing: 1,
            unknown: 0,
        },
        optional: xtask_contract::OptionalStats {
            total: 5,
            passing: 4,
            failing: 0,
            unknown: 1,
        },
    },
    acs: vec![],
};

// Wrap in envelope
let envelope = JsonEnvelope::new(ac_status, "1.0");
let json = serde_json::to_string(&envelope)?;
```

## Migration Notes

When upgrading this crate:

1. Review changelog for breaking changes
2. Update any pattern matching on status enums
3. Handle new optional fields in DTO structures
4. Re-run `cargo check` to catch any API mismatches
