## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-04-13 - Middleware Cache Config Parsing
**Learning:** Security header configs and similar middleware configurations that construct header values from strings do so per request. In a hot path, this is a significant performance hit. The `HeaderValue::clone()` operation is much cheaper as it works with bytes ref counts.
**Action:** Extract caching of middleware configs into an explicit init step (e.g., `CachedSecurityHeaders`) rather than building headers per request.
