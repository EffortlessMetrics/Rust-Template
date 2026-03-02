# gov-http-pagination – CLAUDE.md

Focused pagination primitives for governance HTTP responses.

## Responsibilities

- Define pagination request parameters and response metadata
- Provide deterministic page math (clamping and offsets)
- Stay framework-agnostic and serde-friendly

## Non-responsibilities

- Repository I/O or sorting/filtering logic
- Axum extractors, handlers, or router composition
- Domain-specific governance DTOs
