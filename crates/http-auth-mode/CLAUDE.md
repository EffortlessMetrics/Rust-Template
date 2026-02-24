# CLAUDE.md

This crate contains only platform auth mode parsing and labeling.

## Scope

- `PlatformAuthMode` enum
- Strict parser with fail-closed errors
- Fallback parser with deterministic `open` fallback
- Mode label rendering (`open`, `basic`, `jwt`)

## Out of scope

- credential sourcing from env/config
- token or JWT validation
- HTTP middleware/router wiring
