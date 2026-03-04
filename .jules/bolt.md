## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-03-04 - Type Inference with tokio::task::spawn_blocking
**Learning:** When returning complex tuples containing multiple `Result`s and `Option`s from `tokio::task::spawn_blocking` closures, Rust's type inference often fails.
**Action:** Always explicitly define a type alias for the expected return tuple and provide it as a generic type argument when calling `tokio::task::spawn_blocking::<_, TypeAlias>(...)`. Ensure `Result` success values have matching types, like extracting inner values or explicitly coercing types (e.g., using `?` for `Result` flattening inside `Ok`).
