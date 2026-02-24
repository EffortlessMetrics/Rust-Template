# http-origin-policy

Single-responsibility origin allowlist matching for CORS policy checks.

## Scope

- Match request origins against allowed-origin patterns
- Support exact, wildcard (`*`), path wildcard (`/*`), and subdomain wildcard (`*.domain`) rules
- Provide framework-agnostic matching helpers
- Enforce safe prefix wildcard boundaries (`example.com/*` does not match `example.com.evil`)
- Enforce safe subdomain wildcard boundaries (`api.example.com` matches, `notexample.com` does not)

## Why this crate exists

CORS origin matching logic was duplicated across HTTP crates.
This crate centralizes that behavior so matching rules are tested once and reused consistently.
