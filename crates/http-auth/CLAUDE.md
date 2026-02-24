# CLAUDE.md

This crate contains only platform authentication policy and credential sourcing.

## Scope

- `PlatformAuthConfig`
- Re-export of `PlatformAuthMode` from `http-auth-mode`
- Re-export of `Claims` for compatibility
- Policy-level authorization decisions by mode

## Out of scope

- auth mode parser implementation (in `http-auth-mode`)
- token verification primitives (in `http-auth-verifier`)
- HTTP framework middleware
- header extraction
- router wiring
