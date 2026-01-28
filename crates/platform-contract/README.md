# platform-contract

Stable, dependency-light contract types for the `/platform/*` API endpoints.

## Purpose

This crate defines the externally-consumed shapes and semantics that must remain stable for the Rust-as-Spec platform. It contains:

- **Error envelope types**: Consistent error response format across all platform endpoints
- **Response/request DTOs**: Data transfer objects for platform API
- **Version fields**: Template version and kernel version information
- **Schema IDs**: Type-safe identifiers for platform schemas

## Design Philosophy

- **Minimal dependencies**: Only `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`
- **Stable public API**: Types marked with `#[non_exhaustive]` where growth is expected
- **Internal-only**: `publish = false` - this is a contract crate for internal use only

## Public API

### Error Types

- [`ErrorCode`] - Machine-readable error codes (e.g., `INVALID_REQUEST`, `RESOURCE_NOT_FOUND`)
- [`ErrorResponse`] - JSON error response body with error code, message, request ID
- [`LastErrorSummary`] - Summary of last error for `/platform/status`
- [`ErrorStats`] - Aggregated error statistics
- [`ErrorSummary`] - Complete error summary surfaced via status endpoint

### Platform Status DTOs

- [`PlatformStatus`] - Top-level platform status response
- [`ServiceInfo`] - Service metadata (ID, version, links, tags)
- [`GovernanceStatus`] - Governance health and metrics
- [`LedgerCounts`] - Story/requirement/AC counts
- [`DevExCounts`] - DevEx commands and flows counts
- [`DocCounts`] - Documentation counts and issues
- [`TaskCounts`] - Task counts with optional status breakdown
- [`TaskStatusBreakdown`] - Task status distribution
- [`AcCoverageInfo`] - AC coverage summary
- [`QuestionCounts`] - Question counts with top open questions
- [`FrictionCounts`] - Friction entry counts by severity
- [`SeverityCounts`] - Severity breakdown for friction
- [`FrictionSummary`] - Brief friction entry summary
- [`ForkCounts`] - Fork registry counts
- [`PolicyStatus`] - Policy test status
- [`ConfigSummary`] - Runtime configuration summary (redacted)
- [`AuthSummary`] - Authentication mode and token presence

### IDP Snapshot DTOs

- [`IdpSnapshot`] - Complete IDP snapshot for external IDPs
- [`GovernanceHealth`] - Governance health status
- [`AcCoverage`] - AC coverage metrics
- [`SpecCounts`] - Ledger specification counts
- [`DocumentationMetrics`] - Documentation validation metrics
- [`TaskHints`] - Task hints for agents
- [`TaskHint`] - Individual task hint with metadata

### Coverage DTOs

- [`CoverageSummary`] - AC coverage summary (passing/failing/unknown/total)
- [`CoverageDetail`] - Detailed AC coverage with scenarios
- [`CoverageResponse`] - Complete coverage response

### Utility Types

- [`SchemaId`] - Type-safe schema identifier
- [`QuestionBrief`] - Brief question info for status endpoint

## Stability Guarantees

This crate follows semantic versioning. Breaking changes to public types will result in a major version bump.

### Stable Types

The following types are considered stable and will not have breaking changes in minor versions:

- `ErrorCode` enum variants (new variants may be added via `#[non_exhaustive]`)
- `ErrorResponse` structure
- `PlatformStatus` structure
- `IdpSnapshot` structure

### Evolving Types

Types marked with `#[non_exhaustive]` may receive new fields in future versions:

- `ServiceInfo` - new optional fields may be added
- `GovernanceStatus` - new metrics may be added
- `ConfigSummary` - new settings categories may be added

## Usage Example

```rust
use platform_contract::{ErrorResponse, ErrorCode};

// Create an error response
let error = ErrorResponse {
    error: ErrorCode::ResourceNotFound.to_string(),
    message: "Task not found".to_string(),
    request_id: uuid::Uuid::new_v4().to_string(),
    ac_id: Some("AC-PLT-001".to_string()),
    feature_id: None,
};

let json = serde_json::to_string(&error)?;
```

## Migration Notes

When upgrading this crate:

1. Review changelog for breaking changes
2. Update any pattern matching on `ErrorCode` enum
3. Handle new optional fields in DTO structures
4. Re-run `cargo check` to catch any API mismatches
