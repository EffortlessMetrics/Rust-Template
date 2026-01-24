## 2024-05-22 - Async Runtime Blocking
**Learning:** The `generate_snapshot` function in `crates/app-http/src/platform/idp.rs` was performing synchronous file I/O on the main async thread. This blocks the Tokio runtime worker, degrading performance for all concurrent requests.
**Action:** Always wrap synchronous, heavy operations (like file I/O or CPU-intensive tasks) in `tokio::task::spawn_blocking` when working within async handlers (Axum/Tokio).
