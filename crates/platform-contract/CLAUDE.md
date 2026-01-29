# platform-contract – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal contract)
**Dependencies:** Minimal (serde, chrono, uuid, thiserror)

## Purpose

Dependency-light contract types for `/platform/*` API endpoints. Defines the stable external API surface that consumers depend on.

## Key Exports

### Error Types
- `ErrorCode` – Machine-readable error codes
- `ErrorResponse` – JSON error envelope
- `ErrorStats`, `ErrorSummary` – Error aggregation

### Platform DTOs
- `PlatformStatus` – Top-level `/platform/status` response
- `ServiceInfo`, `GovernanceStatus` – Service metadata
- `LedgerCounts`, `DevExCounts`, `DocCounts`, `TaskCounts` – Spec metrics
- `AcCoverageInfo`, `CoverageSummary`, `CoverageDetail` – Coverage data
- `IdpSnapshot` – Complete snapshot for external IDPs

## When to Modify

- Adding new platform endpoint response types
- Extending existing DTOs with new optional fields
- Adding new `ErrorCode` variants

## When NOT to Modify

- Changing existing field types (breaking change)
- Removing fields (breaking change)
- Adding infrastructure dependencies (violates contract)

## Stability Contract

- Types marked `#[non_exhaustive]` may gain fields in minor versions
- Removing or renaming fields requires major version bump
- New `ErrorCode` variants may be added

## Consumers

`gov-http`, `app-http`, `http-platform`, external IDP integrations

## See Also

- `README.md` in this crate for full type documentation
- `crates/gov-http/` for HTTP handlers using these types
