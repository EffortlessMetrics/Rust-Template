# http-auth-policy

Single-responsibility auth-policy microcrate for platform HTTP authentication.

## Scope

- Own `PlatformAuthConfig`, the runtime policy state used by platform auth middleware.
- Evaluate token authorization behavior for `open`, `basic`, and `jwt` modes.
- Expose credential readiness and misconfiguration signals (`token_present`, `warn_if_misconfigured`).

## Why this exists

`http-auth-config` now focuses on source precedence and environment/config resolution,
while this crate owns deterministic policy behavior for downstream callers.
