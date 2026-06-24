## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Security Headers Middleware Hot Path Optimization
**Learning:** Parsing string configs into `HeaderValue` on every Axum response is an unnecessary allocation and CPU hit on the hot path. Axum's `HeaderValue` is backed by `Bytes`, meaning `.clone()` is extremely cheap (just a reference count increment).
**Action:** Use a `CachedSecurityHeaders` struct to parse strings into `HeaderValue`s once during startup, store it in the `AppState`, and inject using cheap clones during request handling.
