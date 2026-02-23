# http-auth

Single-responsibility HTTP platform authentication primitives:

- auth mode parsing (`open`, `basic`, `jwt`)
- token/secret sourcing from runtime config and env vars
- basic token validation (constant-time compare)
- JWT validation (HS256 + leeway and claim checks)

This crate is intentionally framework-agnostic and only owns auth policy and token checks.
