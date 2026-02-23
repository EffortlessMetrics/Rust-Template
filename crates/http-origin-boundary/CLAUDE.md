# http-origin-boundary – CLAUDE.md

Focused boundary predicates for CORS origin matching.

## Responsibilities

- Evaluate whether a suffix match has a `.` label boundary
- Evaluate whether a prefix remainder is empty or path-boundary (`/`)
- Keep boundary behavior deterministic and framework-agnostic

## Non-responsibilities

- Parsing origins or schemes
- Rule dispatch across wildcard/exact forms
- HTTP middleware orchestration
