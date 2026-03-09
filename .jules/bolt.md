## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-02-17 - Maud Template `clone()` Optimization
**Learning:** Maud HTML templates evaluate dynamically but variable consumption rules still apply via Rust's borrow checker. When variables (like configuration struct fields) are only needed once at the end of their lifecycle within a template, `.clone()` can be safely omitted. Moving the variables instead of cloning saves unnecessary allocations, CPU cycles, and cleans up the code without sacrificing readability.
**Action:** When working with Maud templates, review the scope and lifetime of variables passed into them. Prefer moving variables (or passing references if they need to outlive the template) over indiscriminately using `.clone()`.
