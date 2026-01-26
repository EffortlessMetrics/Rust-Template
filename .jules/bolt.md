## 2024-05-24 - Blocking I/O in Async Handlers
**Learning:** The `app-http` crate uses `axum` (async) but relies on `spec_runtime` which performs synchronous file I/O. Using these directly in handlers blocks the async runtime.
**Action:** Always wrap `spec_runtime` calls (like `load_all_specs`, `load_service_metadata`) in `tokio::task::spawn_blocking` within async handlers to maintain responsiveness.
