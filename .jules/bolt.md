## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-01-27 - Security Headers Optimization
**Learning:** Found that `SecurityHeadersConfig` in both `crates/app-http` and `crates/http-middleware` was re-parsing `String` headers into `HeaderValue` inside the `security_headers_middleware` on every single request. In `axum` / `http`, `HeaderValue` instances are cheap to clone (utilizing atomic ref-counting or inline copying).
**Action:** Created a `CachedSecurityHeaders` struct holding `Option<HeaderValue>`, instantiated it at app startup (via a new `.cache()` method), and updated the Axum `AppState` (and `security_headers_layer`) to store the cached struct. This provides zero-allocation header application during request handling on the hot path. Remember that middleware logic like this is often duplicated across `app-http` and `http-middleware` and must be updated in both places.
