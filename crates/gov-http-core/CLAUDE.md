# gov-http-core – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** http-core, platform-contract, axum

## Purpose

Core HTTP types and helpers for governance endpoints. Provides shared functionality across all gov-http-* crates.

## Key Exports

- Shared extractors for governance endpoints
- Common response types
- Governance-specific HTTP utilities

## When to Modify

- Adding shared governance HTTP utilities
- Extending common types

## When NOT to Modify

- Adding specific endpoint logic (put in gov-http-* sub-crates)

## Architectural Notes

- **Shared foundation**: Used by all gov-http-* crates
- **No routes**: Provides utilities only

## Consumers

`gov-http-forks`, `gov-http-friction`, `gov-http-issues`, `gov-http-questions`, `gov-http`

## See Also

- `crates/gov-http-types/` for shared DTO types
- `crates/gov-http/` for router aggregation
