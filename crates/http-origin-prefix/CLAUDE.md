# http-origin-prefix – CLAUDE.md

Focused matcher for CORS prefix wildcard rules.

## Responsibilities

- Evaluate one `http://host/*` or `https://host/*` rule against one origin string
- Enforce authority boundary semantics to prevent boundaryless host matches
- Keep behavior framework-agnostic and deterministic

## Non-responsibilities

- Exact match, `*`, or subdomain wildcard dispatch
- Managing lists of rules
- HTTP middleware orchestration
