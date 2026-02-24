# http-auth-mode

Single-responsibility platform auth mode parsing primitives:

- `PlatformAuthMode` enum
- strict parsing (`open`, `none`, `basic`, `jwt`)
- fallback parsing for tolerant callers
- stable lowercase mode labels

This crate is intentionally tiny and framework-agnostic.
