# http-auth-sources

Single-responsibility auth source resolution for platform HTTP auth.

## Scope

- Select auth mode, token, and JWT secret from environment and validated runtime config.
- Enforce precedence (`env` over config, then defaults).
- Keep mode parsing delegated to `http-auth-mode`.

## Why this exists

`http-auth-config` owns auth policy decisions and can consume a resolved
`PlatformAuthSourceConfig`, while source precedence now lives behind this
microcrate for focused tests and fuzzing at the input boundary.
