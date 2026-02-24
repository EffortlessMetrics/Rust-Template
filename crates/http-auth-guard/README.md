# http-auth-guard

Single-responsibility request authorization helper for platform HTTP auth guard logic.

## Scope

- Decide whether a request should pass platform auth checks based on method, headers, and config.
- Keep route-level auth policy separate from Axum middleware construction.

## Why this crate exists

The app router needs a single pure authorization decision function that can be
unit-, property-, integration-, fuzz-, and BDD-exercised through the existing
auth platform scenarios.
