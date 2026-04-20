1. Refactor `CORS` in `crates/http-middleware/src/cors.rs` and `crates/app-http/src/middleware/cors.rs` to cache pre-parsed `HeaderValue`s for:
    - `exposed_headers`
    - `allowed_methods`
    - `max_age`
   These are dynamic strings derived from config on *every request* right now, despite never changing at runtime.
2. `allowed_origins` and `allowed_headers` need request context dynamically but `exposed_headers`, `allowed_methods` and `max_age` can absolutely be pre-computed.
