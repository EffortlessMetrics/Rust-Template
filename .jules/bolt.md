## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-02-12 - Middleware Hot Path Allocations
**Learning:** Found instances of `HeaderValue::from_str` string parsing being performed dynamically within Axum middleware on the hot path (per-request) for security headers configuration. This introduces unnecessary per-request parsing overhead and memory allocations.
**Action:** Use `.cache()` mechanisms to pre-parse configuration values into static or reusable structs (like `HeaderValue`s or `CachedSecurityHeaders`) when generating the middleware layer, storing the pre-parsed values in `AppState` or closure state so that the hot path performs zero-allocation header insertions using `.clone()` on cheap types.
