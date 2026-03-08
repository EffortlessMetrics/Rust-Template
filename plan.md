1. **Analyze target files**: Look at `crates/app-http/src/middleware/security_headers.rs` and `crates/http-middleware/src/security_headers.rs`.
2. **Create CachedSecurityHeaders**: Implement `CachedSecurityHeaders` which stores `HeaderValue`s instead of strings. Add a `from(config)` method. Add `apply_headers` method to `CachedSecurityHeaders` that clones the cheap `HeaderValue`s instead of parsing strings on every request.
3. **Update AppState / Middleware Layer**: In `crates/app-http/src/lib.rs`, replace `security_headers_config` with `cached_security_headers`. In `crates/http-middleware/src/security_headers.rs`, update `security_headers_layer` to take `CachedSecurityHeaders` or convert the config to it.
4. **Update tests**: Fix any failing tests that expect the old behavior.
5. **Run linters and tests**: Ensure everything passes.
6. **Pre commit instructions**: Follow pre commit instructions to prepare for submission.
7. **Submit PR**: Submit the changes with appropriate Bolt title and description.
