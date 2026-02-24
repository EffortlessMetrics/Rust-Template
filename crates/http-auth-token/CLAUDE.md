# CLAUDE.md

This crate contains only HTTP auth header token extraction.

## Scope

- `Authorization: Bearer <token>` parsing
- Legacy `x-platform-token` fallback
- Header precedence rules

## Out of scope

- auth policy decisions
- JWT verification
- HTTP middleware/router wiring
