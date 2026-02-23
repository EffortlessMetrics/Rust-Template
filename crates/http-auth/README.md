# http-auth

Single-responsibility HTTP platform authentication primitives:

- auth mode usage (re-exported from `http-auth-mode`)
- token/secret sourcing from runtime config and env vars
- auth policy decisions across open/basic/jwt modes
- re-exported `Claims` for compatibility with existing callers

This crate is intentionally framework-agnostic and owns auth policy + credential sourcing.
Low-level token verification lives in `http-auth-verifier`.
