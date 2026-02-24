# http-origin-policy – CLAUDE.md

Focused matcher for CORS origin allowlist rules.

## Responsibilities

- Determine whether an origin matches an allowlist entry
- Evaluate an origin against multiple allowlist entries
- Keep behavior framework-agnostic and deterministic
- Preserve secure wildcard semantics (subdomain boundaries)

## Non-responsibilities

- HTTP middleware orchestration
- Response header formatting
- Environment/config loading
