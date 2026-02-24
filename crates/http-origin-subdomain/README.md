# http-origin-subdomain

Single-responsibility matcher for CORS subdomain wildcard rules.

## Scope

- Match one `https://*.domain` or `http://*.domain` allowlist rule against one origin
- Enforce same-scheme matching
- Enforce safe label boundaries (`api.example.com` matches, `notexample.com` does not)
- Delegate HTTP origin parsing to `http-origin-parser`

## Why this crate exists

This crate isolates subdomain wildcard semantics from generic origin rule dispatch.
Higher-level crates can compose wildcard, exact, and prefix rules while this crate owns only boundary-safe wildcard matching.
