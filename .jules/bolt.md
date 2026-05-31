## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-04-21 - Cache Axum Middleware Header Parsing
**Learning:** In Axum middleware (like SecurityHeaders), parsing header values from strings on every request (using `HeaderValue::from_str`) causes unnecessary allocations and CPU overhead on the hot path.
**Action:** Pre-parse these header values during initialization into a cached configuration struct (e.g., `CachedSecurityHeaders`), and then `.clone()` these pre-allocated values into the `async move` request handler. Because the underlying data for `HeaderValue` is backed by `Bytes`, cloning is extremely cheap (just a reference count increment).
