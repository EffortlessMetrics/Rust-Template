# http-origin-subdomain - CLAUDE.md

Focused matcher for CORS subdomain wildcard rules.

## Responsibilities

- Evaluate one subdomain wildcard rule (for example `https://*.example.com`) against one origin
- Enforce scheme matching and label-boundary safety
- Keep matching deterministic and framework-agnostic

## Non-responsibilities

- Handling `*`, exact, or `/*` rule forms
- Rule list orchestration
- HTTP middleware orchestration
- Config/env loading
- HTTP origin string parsing (handled by `http-origin-parser`)
