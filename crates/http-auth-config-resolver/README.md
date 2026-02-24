# http-auth-config-resolver

Single-responsibility auth config assembly for platform HTTP authentication.

## Scope

- Convert collected auth input material to `PlatformAuthConfig`.
- Preserve the environment-vs-config precedence semantics from `http-auth-sources`.
- Keep assembly separate from env/config extraction and raw policy behavior.

## Why this crate exists

`http-auth-config` now delegates config assembly to this crate, making each crate in
the auth stack own one narrow responsibility.
