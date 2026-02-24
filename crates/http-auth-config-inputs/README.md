# http-auth-config-inputs

Single-responsibility platform auth input collection.

## Scope

- Read auth-related environment variables and runtime configuration values.
- Preserve env-vs-config precedence rules (env first, then config).
- Provide raw effective values for downstream source-resolution.

## Why this crate exists

Authentication config loading should be separate from source interpretation and policy
decision logic. This crate owns only input collection and leaves validation to
`http-auth-sources` and policy evaluation to `http-auth-policy`.
