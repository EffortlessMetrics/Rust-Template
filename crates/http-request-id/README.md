# http-request-id

Single-responsibility request identifier middleware for Axum services.

## Scope

- Extract `X-Request-ID` from incoming requests when present.
- Generate a UUID request ID when one is not supplied.
- Store the request ID in request extensions (`RequestId`).
- Mirror the request ID onto response headers.
- Provide a middleware layer helper (`request_id_layer`).

## Why this crate exists

This crate isolates request-correlation behavior into a tiny primitive so it can be
reused independently from the broader middleware stack.

## Testing surface

- Unit tests in `src/lib.rs` for value generation/parsing and middleware behavior.
- Property tests in `src/lib.rs` (under `proptests` module).
- Integration tests in `tests/request_id_integration.rs`.
- Fuzz target in `fuzz/fuzz_targets/request_id.rs`.

## Integration

- Re-exported by `http-middleware` for compatibility with existing callers.
