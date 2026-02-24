# http-auth-config

Compatibility adapter for platform HTTP authentication configuration.

## Scope

- Re-export `PlatformAuthConfig`/`PlatformAuthMode` and token claims used by callers.
- Keep public `try_from_sources(...)` behavior stable.
- Delegate actual config loading work to `http-auth-config-loader`.
- Keep config loading and source precedence concerns isolated in the SRP microcrate.

## Why this crate exists

`http-auth-config` is a focused adapter for converting runtime/configuration inputs
into a fully formed `PlatformAuthConfig` for policy evaluation.
