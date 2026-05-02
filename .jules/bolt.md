## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-05-02 - Middleware Performance Bottleneck
**Learning:** Security headers middleware was performing repetitive string parsing (`HeaderValue::from_str`) on every single incoming HTTP request. This constant allocation and parsing causes measurable CPU overhead under load.
**Action:** Always pre-parse static configuration into `HeaderValue` during initialization, cache them in a dedicated struct (e.g., `CachedSecurityHeaders`), and `clone()` them per request. In `axum` (which uses the `http` crate), `HeaderValue::clone()` is extremely cheap (O(1) reference count increment) and eliminates per-request parsing.
