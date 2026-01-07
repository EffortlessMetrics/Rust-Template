## Investigation Report: Issue #35 - Non-Deterministic Generation

### Status
**Status:** CONFIRMED - Multiple non-determinism sources found
**Local gates:** Code audit completed

### Evidence

**Source 1: HashMap iteration (line 1043-1044)**
- Final AC table IS sorted, but intermediate operations use unsorted HashMap

**Source 2: HashSet iteration (line 519)**
```rust
let mut candidates: HashSet<String> = HashSet::new();
for candidate in candidates {  // Non-deterministic order!
```

**Source 3: Test execution count**
- `tests_executed` varies based on parallelism, timing, regex captures

**Source 4: Regex-based parsing (line 452)**
- `cargo test` stdout parsing is timing-dependent

### Plan

**Quick fix:**
```rust
// Replace HashSet with sorted Vec
let mut candidates: Vec<_> = candidates.into_iter().collect();
candidates.sort();
```

**Medium-term:**
- Add `--test-threads=1` for deterministic test ordering
- Remove volatile `tests_executed` from markdown generation

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. HashSet sort is a quick win; consider removing volatile counts from output.
