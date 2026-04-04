1. Modify `crates/http-middleware/src/security_headers.rs` using `replace_with_git_merge_diff`
  - Add `CachedSecurityHeaders`
  - Implement `apply_headers` for `CachedSecurityHeaders`
  - Change `SecurityHeadersConfig::apply_headers` to use lowercase header names
  - Add `cache` method to `SecurityHeadersConfig`
  - Modify `security_headers_layer` to parse into `CachedSecurityHeaders` outside of the middleware loop

2. Modify `crates/app-http/src/middleware/security_headers.rs` using `replace_with_git_merge_diff`
  - Add `CachedSecurityHeaders`
  - Implement `apply_headers` for `CachedSecurityHeaders`
  - Change `SecurityHeadersConfig::apply_headers` to use lowercase header names
  - Add `cache` method to `SecurityHeadersConfig`
  - Modify `security_headers_middleware` to use `CachedSecurityHeaders`

3. Modify `crates/app-http/src/lib.rs` using `replace_with_git_merge_diff`
  - Change `pub security_headers_config: SecurityHeadersConfig` to `CachedSecurityHeaders`
  - Set it as `security_headers_config: SecurityHeadersConfig::from_sources(config.as_ref()).cache()` in `AppState::new()`
  - Update imports

4. Modify `crates/app-http/src/middleware/platform_auth.rs` using `replace_with_git_merge_diff`
  - Update `security_headers_config` to `crate::middleware::SecurityHeadersConfig::default().cache()`

5. Verify: `cargo check --workspace --tests`
6. Test: `cargo test -p app-http -p http-middleware`
7. Format/Lint: `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`
8. Complete pre-commit steps.
9. Submit PR.
