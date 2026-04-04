1. **Understand Goal**: Refactor `SecurityHeadersConfig` in both `crates/app-http` and `crates/http-middleware` to introduce a `CachedSecurityHeaders` pre-parsed structure, as instructed by the memory "Middleware configurations stored in the Axum AppState (e.g., `security_headers_config` in `crates/app-http/src/lib.rs`) should store their pre-parsed cached representations (like `CachedSecurityHeaders`) rather than raw configuration structs to ensure zero-allocation access during request handling on the hot path."

2. **Actions in `crates/app-http`**:
   - Update `crates/app-http/src/middleware/security_headers.rs`:
     - Create `CachedSecurityHeaders` struct with pre-parsed `HeaderValue`s instead of `String`s.
     - Add `.cache()` method to `SecurityHeadersConfig` to produce `CachedSecurityHeaders`.
     - Move the `apply_headers` method implementation to `CachedSecurityHeaders`.
     - Note: memory states "ensure legacy dynamic methods retain their original inline parsing instead of calling `.cache()` internally". But for Axum middleware, `state.security_headers_config.apply_headers(&mut response)` should be called on `CachedSecurityHeaders`. Wait, memory says: "should store their pre-parsed cached representations... rather than raw configuration structs". So `AppState` should store `CachedSecurityHeaders`.
   - Update `crates/app-http/src/lib.rs`:
     - Change `pub security_headers_config: SecurityHeadersConfig` to `pub security_headers_config: CachedSecurityHeaders` (and import it).
     - Initialize it with `.cache()` in `AppState::new`.
   - Update `crates/app-http/src/middleware/platform_auth.rs`:
     - Update `security_headers_config: crate::middleware::SecurityHeadersConfig::default().cache()`.
   - Update `crates/app-http/tests/security_middleware.rs` and other test files if they reference `SecurityHeadersConfig` applying headers.

3. **Actions in `crates/http-middleware`**:
   - Similar to `app-http`, update `crates/http-middleware/src/security_headers.rs` with `CachedSecurityHeaders` and `.cache()`.
   - Update `security_headers_layer` to take `CachedSecurityHeaders` or `.cache()` it outside the closure. The memory says: "expensive configurations or structural parsing (e.g., caching settings into pre-parsed types) should be evaluated outside the middleware closure, and then `.clone()`d into the `async move` request handler to eliminate per-request overhead." So we can accept `SecurityHeadersConfig`, `.cache()` it, and move the cached config into the closure. Wait, the memory says "Middleware configurations stored in the Axum `AppState` ... should store their pre-parsed cached representations...". I will just modify `security_headers_layer` to cache it.

4. **Verify changes**:
   - `cargo check --workspace --tests`
   - `cargo test -p app-http -p http-middleware`

5. **Format and Lint**:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`

6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
7. **Submit the PR.**
