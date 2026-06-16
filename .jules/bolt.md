## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Properly handling nested Results in spawn_blocking
**Learning:** When moving complex operations that return `Result<T, E>` into `spawn_blocking`, the returned type becomes `Result<Result<T, E>, JoinError>`. Using `.ok().and_then(|r| r.ok())` or mapping errors gracefully and using the double-question-mark operator `??` is much safer and more idiomatic than blindly trying to construct unverified custom error enum variants.
**Action:** When mapping nested results from `spawn_blocking`, prefer using `??` combined with `.map_err` or flattening methods over manually guessing custom `Error` variants which often leads to compilation failures.
