# http-auth-verifier

Single-responsibility token verification primitives for platform HTTP auth.

## Scope

- Constant-time string equality for basic token checks
- HS256 JWT verification with claim sanity checks (`exp`, `iat`, issuer, subject)
- Authorization decision helper combining basic token and JWT secret inputs

## Why this crate exists

`http-auth` originally bundled credential sourcing and token verification.
This crate isolates verification semantics so they can be fuzzed, property-tested,
and reused without coupling to config loading.
