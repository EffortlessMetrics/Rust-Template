## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-02-05 - Tokio spawn_blocking in unlinked crates
**Learning:** When using `tokio::task::spawn_blocking` in a crate, you must ensure `tokio` is added to its `Cargo.toml` dependencies (e.g., `tokio.workspace = true`). Without it, the compiler cannot infer the types and fails with 'unresolved module or unlinked crate'.
**Action:** Always verify `tokio` is a dependency before using `spawn_blocking`.
