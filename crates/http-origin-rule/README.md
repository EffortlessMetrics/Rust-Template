# http-origin-rule

Single-responsibility matcher for one CORS origin allowlist rule.

## Scope

- Match a single rule against one origin
- Support `*`, exact, `/*` prefix, and `https://*.domain`/`http://*.domain`
- Delegate `/*` prefix wildcard boundary checks to `http-origin-prefix`
- Delegate subdomain wildcard boundary checks to `http-origin-subdomain`

## Why this crate exists

This crate isolates one-rule matching so higher-level crates can focus on list orchestration and policy composition.
Prefix wildcard behavior is isolated in `http-origin-prefix`, and subdomain wildcard behavior is isolated
in `http-origin-subdomain`, for stricter SRP boundaries.
