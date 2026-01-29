# gov-http-types – CLAUDE.md

**Tier:** Platform HTTP (Layer 5)
**Publish:** No (internal)
**Dependencies:** serde, platform-contract

## Purpose

Shared HTTP API types for governance endpoints. DTOs shared across multiple gov-http-* crates.

## Key Exports

- `FrictionEntry` – Friction tracking DTO
- `Question` – Question/clarification DTO
- Other shared governance DTOs

## When to Modify

- Adding new shared DTO types
- Extending existing DTOs

## When NOT to Modify

- Adding HTTP handlers (put in gov-http-* crates)
- Adding non-governance types (put in platform-contract)

## Architectural Notes

- **DTOs only**: Data transfer objects, no logic
- **Shared**: Prevents duplication across gov-http-* crates

## Consumers

`gov-http-friction`, `gov-http-questions`, `gov-http`

## See Also

- `crates/platform-contract/` for platform-wide types
- `crates/gov-http-*/` for endpoint implementations
