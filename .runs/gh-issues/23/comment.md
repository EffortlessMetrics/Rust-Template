## Investigation Report: Issue #23 - Consolidate Duplicate Code

### Status
**Status:** CONFIRMED - ~350-450 LOC duplication identified
**Local gates:** Code audit completed

### Evidence

**Duplicated Entry Loading (~150-200 LOC):**
- `gov-http/src/friction.rs:104-148` - load_all_friction_entries()
- `gov-http/src/forks.rs:89-142` - Nearly identical pattern
- `gov-http/src/questions.rs:123-168` - Nearly identical
- Duplicated again in xtask commands

**Duplicated Data Structures (~200-250 LOC):**
- `FrictionEntry`: Identical in gov-http and xtask
- `ForkEntry`: Identical in gov-http and xtask
- `RelatedItems`: Defined 4+ times with slight variations

**Duplicated Test Helpers (~40 LOC):**
- `test_workspace_root()`: Identical in 5+ test files

### Plan

1. **Create generic entry loader** in spec-runtime:
   ```rust
   fn load_yaml_entries<T>(dir: &Path, filter: F) -> Result<Vec<T>, Error>
   ```

2. **Extract shared types** to gov-model or new commons crate

3. **Consolidate test helpers** into tests/common/mod.rs

**Effort:** 3-4 days (Medium complexity)

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Clear scope, high maintainability impact.
