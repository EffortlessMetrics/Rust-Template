## Investigation Report: Issue #28 - Test Coverage for Adapters

### Status
**Status:** fix-ready - 774 LOC with 0 unit tests
**Local gates:** Coverage audit completed

### Evidence

| Crate | Source LOC | Unit Tests | Integration Tests |
|-------|------------|------------|-------------------|
| adapters-spec-fs | 359 | 0 | 2 tests (71 LOC) |
| adapters-grpc | 117 | 0 | 1 test (118 LOC) |
| adapters-db-sqlx | 298 | 0 | 1 test (102 LOC) |
| **Total** | **774** | **0** | **4 tests** |

**Critical gaps:**
- `tasks_state::update_task_status()` - File locking untested
- `PostgresTaskRepository::new()` - DATABASE_URL validation untested
- `parse_example_task_status()` - Error handling untested

### Plan

**Tier 1:** Add unit test modules to each adapter
**Tier 2:** Enhance integration tests with error paths

**Target:** 25-35 unit tests across 3 adapters

### Decision / Next Action

**Recommend:** Keep open as **HIGH priority**. Related to #69 (testing coverage epic).
