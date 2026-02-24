# http-origin-boundary

Single-responsibility boundary predicates for CORS origin rule matching.

## Scope

- Check label boundaries before domain suffixes
- Check path boundaries after origin prefix matches
- Keep boundary semantics deterministic and framework-agnostic

## Why this crate exists

Origin matcher crates (`http-origin-prefix`, `http-origin-subdomain`) both rely on strict
boundary checks. This crate isolates those predicates so they can be reused, fuzzed,
and property-tested independently from rule parsing/orchestration.
