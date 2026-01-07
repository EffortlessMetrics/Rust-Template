## Investigation Report: Issue #70 - Code Quality & Type Safety Epic

### Status
**Status:** triaged / fix-ready
**Local gates:** clippy ✓ | selftest ✓ | cargo check ✓ | tests pass ✓

### Evidence

**Category 1: Dead Code Annotations Hiding Actual Implementation**
- `crates/xtask/src/commands/ac_status.rs:728` - `#[allow(dead_code)]` on `normalize_testcase_name()` (test-only, cfg-gated)
- `crates/xtask/src/commands/ac_status.rs:788` - `#[allow(dead_code)]` on `parse_junit()` (test-only, cfg-gated)
- `crates/xtask/src/commands/github.rs:85-86` - `#[allow(dead_code)]` on `get_repo()` (for `--repo` override)
- `crates/xtask/src/commands/github.rs:164-165` - `#[allow(dead_code)]` on `close_issue()` (friction resolution)
- `crates/xtask/src/commands/github.rs:182-183` - `#[allow(dead_code)]` on `get_issue_state()`
- `crates/xtask/src/commands/versioning.rs:113` - `#[allow(dead_code)]` on `with_date()` (retroactive updates)

**Category 2: Result<T, String> Pattern (Untyped Errors)**
- `crates/business-core/src/lib.rs:31-44` - `ExampleTaskRepository` trait uses `Result<_, String>` for all methods
- `crates/business-core/src/lib.rs:63-109` - All use_cases functions return `Result<T, String>`
- `crates/adapters-spec-fs/src/tasks_def.rs:87` - `load_tasks_definitions()` returns `Result<HashMap, String>`
- `crates/app-http/src/lib.rs:188, 197, 227, 243` - Multiple initialization functions use `Result<_, String>`
- `crates/app-http/src/security.rs:64, 235` - Security parsing functions use `Result<_, String>`

**Note:** Codebase HAS typed error enums in some places:
- `crates/gov-policy/src/runner.rs` - `PolicyTestError` enum
- `crates/gov-model/src/lib.rs` - `GovernanceError` enum
- `crates/app-http/src/errors.rs` - `ErrorCode` enum with comprehensive error tracking

**Category 3: Overly Complex Functions**
- `crates/xtask/src/commands/ac_status.rs` - **1869 lines total**
  - `run()` function (lines 124-369): 245 lines with nested conditionals
  - `update_ac_statuses()` (lines 547-630): Complex state machine logic
  - `collect_unit_test_results()` (lines 438-512): Runs cargo test twice with different target dirs

**Category 4: Missing Documentation**
- `crates/adapters-db-sqlx/src/lib.rs:111` - `PostgresTaskRepository` struct lacks doc comment
- `crates/xtask/src/commands/ac_status.rs` - File-level docs minimal; no overview of three-path fallback strategy
- Multiple utility functions lack doc comments

**Category 5: Inconsistent Naming Conventions**
- `collect_unit_test_results()` vs `parse_junit()` vs `parse_ac_coverage()` - mixing verbs
- Repository naming: `PostgresTaskRepository` vs `FsGovernanceRepository` vs `InMemoryExampleTaskRepository`

### Impact
- **Maintainability**: Dead code annotations mask design intentions
- **Type Safety**: String errors prevent compile-time checking (#55 explicitly tracking this)
- **Correctness**: Complex state machine logic hard to verify without detailed tests
- **Onboarding**: 1869-line ac_status.rs with three fallback paths unclear to new developers
- **Consistency**: Inconsistent error types across crates make refactoring harder

**Blast Radius:**
- **High impact**: business-core (foundational), app-http (public API), xtask commands
- **Medium impact**: adapters-spec-fs, adapters-db-sqlx

### Plan

**Minimal fix (Phase 1):**

1. **Untyped Errors** - Related to #55
   - Create `TaskRepositoryError` enum in business-core
   - Update all implementations to use typed error
   - Migrate app-http initialization errors to ErrorCode enum

2. **Remove Unnecessary Dead Code Markers** - Related to #51
   - Delete test-only `normalize_testcase_name()` and `parse_junit()` from ac_status.rs
   - Document github.rs methods with issues/ADR rather than hiding

3. **Refactor ac_status.rs Complexity (Phase 2)** - Related to #26
   - Extract fallback_to_junit() into separate module
   - Extract outcome_for_unit_test() into testcase_matcher.rs
   - Add module-level doc comments

4. **Naming Consistency (Phase 3)**
   - Standardize Repository struct naming
   - Function naming: consistent verb prefixes

**Follow-ups:**
- Issue #26: Audit duplicated entry loading code
- Issue #23: Create shared data model structures
- Issue #29: Complete documentation for all public APIs

**Test plan:**
```bash
cargo clippy --all-targets
cargo test --workspace
cargo xtask selftest
```

### Decision / Next Action

**Recommend:** Keep open with labels [epic, fix-ready, type-safety]

This epic has clear sub-issues (#55, #51, #18, #26, #27, #29). The codebase is **functionally correct** (selftest passes) but has **maintainability and type safety debt**.

**Suggested sequencing:**
1. Start with #55 (Result<T, String> → typed errors) - highest impact, foundational
2. Follow with #51 (dead_code cleanup) - quick wins
3. Then #26 (refactor ac_status) - large but well-scoped
4. Parallel #18 (standardize error strategy) - cross-cutting
