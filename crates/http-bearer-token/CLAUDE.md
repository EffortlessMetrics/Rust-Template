# CLAUDE.md

This crate contains only `Authorization: Bearer <token>` parsing.

## Scope

- Case-insensitive `Bearer` scheme detection
- Token extraction from raw authorization header strings
- Deterministic `Option<&str>` output without allocation

## Out of scope

- Header map lookup
- Legacy token fallbacks
- Auth policy or JWT validation
