# http-origin-parser

Single-responsibility parser for HTTP/HTTPS origin strings used by origin matching crates.

## Scope

- Parse `http://` and `https://` origins into scheme + authority
- Reject malformed values (missing authority, path/query/fragment/userinfo, whitespace)
- Provide deterministic parsing primitives for higher-level rule matchers

## Why this crate exists

Origin matching crates should own matching semantics, not low-level origin string parsing.
This crate isolates strict parsing so wildcard and policy crates can share one audited parser.
