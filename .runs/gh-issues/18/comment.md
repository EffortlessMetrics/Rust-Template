## Investigation Report: Issue #18 - Standardize Error Handling Strategy

### Status
**Status:** fix-ready - Strategy clear, proven patterns exist in codebase
**Local gates:** Code audit, pattern analysis

### Evidence

**THREE DISTINCT PATTERNS found:**

#### Pattern 1: `Result<T, String>` (ANTI-PATTERN) - ~20+ instances
- `crates/business-core/src/lib.rs:31-44` - ExampleTaskRepository trait (5 methods)
- `crates/adapters-db-sqlx/src/lib.rs:177-272` - PostgresTaskRepository (4 methods)
- `crates/app-http/src/lib.rs:188,197,227,243` - AppState initialization (4 methods)
- `crates/app-http/src/security.rs:64,235` - Auth parsing (2 methods)
- `crates/adapters-spec-fs/src/tasks_def.rs:87` - Task loading

#### Pattern 2: `anyhow::Result` (ACCEPTABLE for CLI) - ~36 files
- `crates/spec-runtime/` - SpecLedger loading
- `crates/xtask/` - CLI commands (appropriate)

#### Pattern 3: Custom `thiserror` enums (BEST PRACTICE) ✅
- `crates/app-http/src/errors.rs:1-603` - **AppError** with ErrorCode enum
- `crates/gov-model/src/lib.rs:126-149` - **GovernanceError** (5 variants)
- `crates/gov-http/src/error.rs:1-88` - **PlatformError** (3 variants)
- `crates/rust_iac_config/src/error.rs:1-110` - **ConfigError** (9 variants)

**Good patterns already implemented:**
- AC ID tracking in errors
- Request ID correlation
- Machine-readable error codes
- Proper `IntoResponse` for HTTP

**Missing From implementations:**
- ✗ `From<TaskRepositoryError>` (doesn't exist)
- ✗ `From<SpecLoadError>` (doesn't exist)
- ✗ `From<AuthConfigError>` (doesn't exist)

### Impact

- **Type Safety:** Cannot pattern match on `Result<T, String>` errors
- **Debugging:** String errors lose semantic information
- **Testing:** Cannot assert on specific error types
- **Observability:** Incorrect HTTP status codes possible

### Plan

**Strategy: Extend Pattern 3 (thiserror enums)**

1. **Create domain error types:**
   - `business-core::RepositoryError` enum
   - `spec-runtime::SpecError` enum
   - `app-http::AuthConfigError` enum

2. **Update trait signatures:**
   - `ExampleTaskRepository` methods: `Result<T, RepositoryError>`

3. **Add From implementations:**
   - `impl From<RepositoryError> for AppError`
   - `impl From<SpecError> for AppError`

4. **Keep xtask on anyhow** (acceptable for CLI)

**Test plan:**
```bash
cargo test --workspace
cargo xtask selftest
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. This is upstream of #55 (typed errors). Strategy is clear, patterns proven in codebase. Implement in phases without blocking other work.

**Related:** #55, #70 (Code Quality Epic)
