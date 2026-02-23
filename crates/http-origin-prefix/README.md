# http-origin-prefix

Single-responsibility matcher for CORS allowlist prefix wildcard rules.

## Scope

- Match one `https://host/*` or `http://host/*` allowlist rule against one origin string
- Enforce scheme/authority boundary semantics for prefix matches
- Delegate HTTP origin prefix parsing to `http-origin-parser`

## Why this crate exists

Prefix wildcard matching is distinct from exact/wildcard/subdomain rule dispatch.
This crate isolates prefix matching so boundary semantics can be reused, fuzzed, and
property-tested independently from higher-level rule orchestration.
