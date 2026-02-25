## 2026-02-25 - Async Blocking in Dashboard Handler
**Learning:** `tokio::task::spawn_blocking` should be used to offload synchronous I/O operations (like file reading and parsing) in async handlers to prevent blocking the reactor.
**Action:** When implementing or modifying async handlers, identify blocking operations and wrap them in `spawn_blocking`, especially when dealing with file I/O or CPU-intensive parsing.
