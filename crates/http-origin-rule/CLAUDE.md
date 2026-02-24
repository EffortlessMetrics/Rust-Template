# http-origin-rule – CLAUDE.md

Focused matcher for a single CORS allowlist rule.

## Responsibilities

- Evaluate one allowlist rule against one origin string
- Keep wildcard behavior deterministic and framework-agnostic
- Delegate `/*` prefix wildcard boundary semantics to `http-origin-prefix`
- Delegate subdomain wildcard boundary semantics to `http-origin-subdomain`

## Non-responsibilities

- Managing lists of rules
- HTTP middleware orchestration
- Config/env loading
