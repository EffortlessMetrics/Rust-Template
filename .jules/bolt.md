## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-02-14 - Graph Adjacency Indexing
**Learning:** The core `Graph` struct uses a flat `Vec<Edge>` storage, making connectivity checks O(N*M) by default. Invariant checking was iterating all edges for every node.
**Action:** Always build a temporary `HashSet` or adjacency index when performing graph traversals or connectivity checks to achieve O(N+M) complexity.
