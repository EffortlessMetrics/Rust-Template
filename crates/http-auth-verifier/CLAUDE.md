# http-auth-verifier – CLAUDE.md

Focused token verification primitives for platform auth.

## Responsibilities

- Constant-time basic token equality checks
- JWT signature and claim validation
- Unified authorization decision from provided token + configured credentials
- Keep verification deterministic and framework-agnostic

## Non-responsibilities

- Header extraction
- Auth mode parsing
- Env/config sourcing
- Middleware/router orchestration
