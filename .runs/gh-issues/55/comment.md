## Investigation Report: Issue #55 - Typed Errors

### Status
**Status:** fix-ready
**Local gates:** grep patterns, code review of error handling patterns, file-by-file analysis

### Evidence

**1. Business Core Layer - Result<T, String> trait methods**
- `crates/business-core/src/lib.rs:31-44` (ExampleTaskRepository trait)
- All four repository methods return `Result<_, String>` instead of typed errors:
  - `save()` → `Result<(), String>` (line 31)
  - `find_by_id()` → `Result<Option<ExampleTask>, String>` (line 34)
  - `find_all()` → `Result<Vec<ExampleTask>, String>` (line 37)
  - `update_status()` → `Result<Option<ExampleTask>, String>` (lines 40-44)
- **Impact:** Callers cannot distinguish between I/O errors, validation errors, or logical errors.

**2. Spec Loading - String error wrapper**
- `crates/adapters-spec-fs/src/tasks_def.rs:87-92` (`load_tasks_definitions`)
- Returns `Result<HashMap<String, TaskDefinition>, String>`
- Uses `map_err(|e| format!(...))` pattern for file I/O and YAML parsing
- **Impact:** File system errors and YAML parse errors conflated into generic strings.

**3. Security Configuration - String error return**
- `crates/app-http/src/security.rs:64, 235` (`try_from_sources`, `parse_strict`)
- Both methods return `Result<Self, String>` for auth configuration parsing
- **Impact:** Cannot distinguish invalid auth mode from config read failure.

**4. Doctor Checks - Mixed error patterns**
- `crates/xtask/src/commands/doctor.rs:268-425`
- Four functions use `Result<String, String>` pattern for health checks
- **Impact:** Cannot distinguish between different types of environment issues.

**5. Error Boundary Mapping - Lossy conversions**
- `crates/adapters-spec-fs/src/lib.rs:74` (FsGovernanceRepository)
- Converts string error to `GovernanceError::Io` using `std::io::Error::other(e)`
- **Impact:** Loses error context; file format errors masqueraded as I/O errors.

**6. Existing patterns show better approach**
- `crates/rust_iac_config/src/error.rs:1-84` - Well-structured `ConfigError` enum with thiserror
- `crates/gov-model/src/lib.rs:126-149` - Properly typed `GovernanceError` variants
- `crates/app-http/src/errors.rs:1-603` - Production-grade `AppError` with ErrorCode enum

### Impact

**Blast radius:** Cross-cutting
- Affects domain-to-HTTP adapter boundary (business-core → app-http)
- Affects spec loading in adapters-spec-fs (specs → domain)
- Affects security initialization in app-http (configuration → service)
- Affects development workflows (xtask health checks)

**Problems:**
- **Type Safety:** Callers cannot pattern match on specific error conditions
- **Error Context:** When string error reaches HTTP layer, no semantic information
- **Testing:** Cannot assert on specific error types in unit tests
- **Observability:** Errors lose information about root cause

### Plan

**Minimal fix:**

1. **Create typed error enums** in domain crates following `thiserror` pattern:
   - `business-core` crate: Add `TaskRepositoryError` enum
   - `adapters-spec-fs` crate: Add `SpecLoadError` enum
   - `app-http/security.rs`: Add `AuthConfigError` enum

2. **Update trait signatures** in business-core:
   - Change `ExampleTaskRepository` methods from `Result<_, String>` to `Result<_, TaskRepositoryError>`

3. **Update adapter implementations:**
   - `adapters-spec-fs/src/lib.rs`: Convert errors properly
   - `adapters-grpc/tests/smoke.rs`: Mock implementation

4. **Update error boundary mapping** in app-http:
   - Add `From<TaskRepositoryError>` on `AppError`
   - Add `From<AuthConfigError>` on `AppError`
   - Add `From<SpecLoadError>` on `AppError`

**Follow-ups:**
- Audit for remaining `Result<T, String>` patterns (14 instances found)
- Add custom error type for BDD/acceptance test errors if needed
- Document error handling pattern in ARCHITECTURE.md

**Test plan:**
```bash
cargo test -p business-core errors
cargo test -p adapters-spec-fs errors
cargo test -p app-http --test '*' error
cargo xtask selftest
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. The issue is well-scoped, has clear acceptance criteria, and the solution pattern is already proven in the codebase (ConfigError, GovernanceError, AppError examples). The fix requires ~4-6 crates to be updated but is straightforward with no breaking changes.

**Priority:** Medium - improves internal type safety and testability.
