# http-auth-config-loader

Single-responsibility auth config loader for platform HTTP authentication.

## Scope

- Collect raw auth source values from environment and validated runtime config.
- Resolve the effective auth mode/credentials with env-first precedence.
- Return a concrete `PlatformAuthConfig` for policy enforcement.

## Why this crate exists

This crate owns the last-mile orchestration between input collection and policy object
assembly. It intentionally stays tiny so it is easy to fuzz, property-test, and
integration-test at the auth loading boundary.
