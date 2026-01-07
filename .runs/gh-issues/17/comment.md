## Investigation Report: Issue #17 - Spec Ledger Caching

### Status
**Status:** fix-ready - Clear performance bottlenecks identified
**Local gates:** Code audit, call chain analysis

### Evidence

**Repeated spec loads on every request:**

1. **`gov-http/src/handlers.rs:279-291`** - `get_graph()`
   - Calls `load_all_specs_with_context()` on every request
   - Then calls `build_graph()` with nested iterations

2. **`gov-http/src/handlers.rs:385-412`** - `get_coverage()`
   - Calls `load_all_specs_with_context()` on every request
   - Builds complete AC map via triple-nested loop

3. **`app-http/src/platform.rs:384-399`** - `get_status()`
   - Calls `load_all_specs(root)` + `load_tasks()` on every request

4. **`app-http/src/platform/ui.rs:186-188`** - `dashboard()`
   - Calls `load_all_specs()` + `load_tasks()` on every page render

5. **`app-http/src/platform/ui.rs:400-405`** - `graph_view()`
   - Calls `load_all_specs()` + `build_graph()` for Mermaid diagram

**Underlying I/O (`spec-runtime/src/lib.rs:109-115`):**
- 3 separate `fs::read_to_string()` calls per load
- 3 YAML parses with `serde_yaml::from_str()` per load
- No caching - every load hits filesystem

### Impact

**Current Performance:**
- Minimum 3 file reads + 3 YAML parses per spec-using endpoint
- Graph building: O(stories × requirements × ACs × tests)
- Under load: disk I/O contention, CPU overhead

**Expected Improvement:**
- First request: baseline (initial load)
- Subsequent requests: 99%+ latency reduction
- 10+ concurrent requests: dramatic reduction in I/O contention

### Plan

**Recommended: Lazy Static Cache with Arc**

```rust
// In spec-runtime or app-http
use once_cell::sync::Lazy;

static SPECS_CACHE: Lazy<Arc<AllSpecs>> = Lazy::new(|| {
    Arc::new(load_all_specs(&workspace_root()).expect("specs must load"))
});

static GRAPH_CACHE: Lazy<Arc<Graph>> = Lazy::new(|| {
    Arc::new(build_graph(&SPECS_CACHE))
});
```

**Implementation steps:**
1. Define cache statics in `spec-runtime/src/lib.rs`
2. Update all handlers to use cached versions
3. Document invalidation strategy (server restart)
4. Add benchmark test

**Invalidation strategy:**
- Specs are static per-runtime (acceptable)
- Server restart required for spec changes
- Document in `/platform/status` if needed

**Test plan:**
```bash
# Benchmark before/after
wrk -t4 -c50 -d10s http://localhost:8080/platform/graph
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Simple implementation (5-10 lines per handler), aligns with existing patterns (`once_cell`, `OnceLock`), target 50%+ latency reduction.
