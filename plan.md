1. **Optimize SecurityHeadersConfig caching**
   - The current `security_headers_middleware` parses `HeaderValue::from_str` for every single request in `crates/app-http/src/middleware/security_headers.rs` and `crates/http-middleware/src/security_headers.rs`.
   - We will create a `CachedSecurityHeaders` struct that stores the parsed `HeaderValue`s to avoid string parsing and allocation on every request.
   - We will parse the headers once when the `SecurityHeadersConfig` is loaded, and then store `CachedSecurityHeaders` in `AppState`.
   - The middleware will simply clone/insert the already parsed `HeaderValue`s into the response headers.

2. **Update `crates/app-http/src/middleware/security_headers.rs`**
   - Implement `CachedSecurityHeaders` struct containing `Option<HeaderValue>` and `HeaderValue` fields.
   - Add a method to `SecurityHeadersConfig` to generate `CachedSecurityHeaders`.
   - Change `apply_headers` to accept `&CachedSecurityHeaders` or move logic to `CachedSecurityHeaders::apply_headers`.
   - Actually, since `CachedSecurityHeaders` requires parsed headers, we can just replace `security_headers_config` with `cached_security_headers` in `AppState` or store both.

3. **Update `crates/http-middleware/src/security_headers.rs`**
   - Perform the same optimization as above to keep both crates consistent.

4. **Verify and Pre-commit**
   - Run tests `cargo test -p app-http -p http-middleware`.
   - Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
   - Run `cargo fmt --all`.
   - Call `pre_commit_instructions` tool to perform required checks before submit.
