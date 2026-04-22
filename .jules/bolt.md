## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Scoping performance optimizations
**Learning:** Attempting to fix all instances of a performance anti-pattern (like blocking I/O) across the entire codebase can violate strict persona boundaries (e.g., `< 50 lines`).
**Action:** Always scope performance improvements to a single specific handler or module to ensure the change is small, safe, and verifiable.
