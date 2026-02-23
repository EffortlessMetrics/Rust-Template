# http-auth-token

Single-responsibility token extraction for platform HTTP authentication headers.

## Scope

- Parse bearer tokens from `Authorization` header values
- Select the effective auth token from `Authorization` and legacy `X-Platform-Token`
- Extract auth token from `http::HeaderMap`

## Why this crate exists

`app-http` middleware previously embedded header parsing logic directly.
This crate isolates token extraction so precedence and parsing rules can be
reused, fuzzed, and property-tested independently from auth policy and routing.
