## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-06-25 - Header Parsing in Middleware Hot Path
**Learning:** Security headers were being parsed from strings into `HeaderValue`s using `HeaderValue::from_str` on every single request in the Axum middleware closure, allocating strings repeatedly for static values.
**Action:** When implementing Axum middleware, perform expensive configuration parsing (like converting config strings to `HeaderValue`s) once during app initialization, store them in a cached struct, and `.clone()` them into the `async move` request handler since `HeaderValue::clone()` is cheap (backed by `Bytes` reference counting).
