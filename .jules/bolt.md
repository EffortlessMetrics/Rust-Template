## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-02-05 - Axum Header Allocation Optimization
**Learning:** Security headers middleware was allocating Strings and calling `HeaderValue::from_str` per-request on the hot path. Axum's `HeaderValue::clone` is cheap (backed by `Bytes` ref-count).
**Action:** Always pre-parse configurations into `HeaderValue`s during `AppState` construction via a `.cache()` struct, and strictly use lowercase static strings (e.g. `"content-security-policy"`) for `.insert()` to avoid dynamic case conversion panics or allocations.
