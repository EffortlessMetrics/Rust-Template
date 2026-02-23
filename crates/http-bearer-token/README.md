# http-bearer-token

Single-responsibility parser for `Authorization: Bearer <token>` header values.

## Scope

- Parse bearer tokens from authorization header strings
- Accept case-insensitive bearer schemes
- Return borrowed token slices without allocation

## Why this crate exists

Bearer parsing is a reusable primitive shared by higher-level HTTP auth crates.
This crate isolates scheme parsing so behavior can be tested, fuzzed, and
property-tested independently from header precedence and auth policy.
