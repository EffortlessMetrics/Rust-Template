1. Add `CachedSecurityHeaders` to `crates/app-http/src/middleware/security_headers.rs` and `crates/http-middleware/src/security_headers.rs` to cache pre-parsed `HeaderValue`s.
2. Update `crates/app-http/src/lib.rs` to initialize `cached_security_headers` as `Arc<CachedSecurityHeaders>` in `AppState`.
3. Update `security_headers_middleware` in `app-http` to use `cached_security_headers.apply_headers()`.
4. Update `security_headers_layer` in `http-middleware` to use `CachedSecurityHeaders`.
5. Run format, clippy, and tests.
6. Submit a PR.
