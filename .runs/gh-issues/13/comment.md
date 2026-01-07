## Investigation Report: Issue #13 - Unsafe Env Var Manipulation

### Status
**Status:** triaged - P0 CRITICAL
**Local gates:** grep and code audit for comprehensive analysis

### Evidence

Found **31 instances** of unsafe environment variable manipulation:

**1. crates/xtask/src/commands/tasks.rs:173-186** - Test isolation wrapper
- Three `unsafe` blocks manipulating SPEC_ROOT
- No synchronization mechanism despite comment claiming isolation

**2. crates/app-http/tests/security_middleware.rs:75,84-86,119** - EnvVarGuard
- Uses `tokio::sync::Mutex` + `OnceLock` (GOOD pattern)
- BUT: All 13 `#[tokio::test]` functions lack `#[serial]` attribute
- Race condition: tests run with default parallelism despite guard

**3. crates/app-http/src/middleware/security_headers.rs:393-443** - Security tests
- Only 3 of ~7 tests with env mutations marked `#[serial]`
- Incomplete coverage leaves 4 tests vulnerable

**4. crates/xtask/src/kernel.rs:230-252** - with_spec_root function
- Uses ENV_LOCK mutex (GOOD) but assumes single-threaded usage

**5. crates/acceptance/src/world.rs:267-278** - Cucumber World
- Depends on `.max_concurrent_scenarios(1)` runtime config
- No compile-time guarantee; fragile assumption

**6. crates/telemetry/src/lib.rs:310-351** - RUST_LOG tests
- 5 unsafe operations, ZERO `#[serial]` markers
- Comments claim isolation but tests use default parallelism

**7. crates/xtask/src/commands/doctor.rs:512-534** - sccache tests
- 3 unsafe operations, ZERO `#[serial]` markers
- No restoration guarantee if test panics

### Impact

**Test Flakiness Risk:** HIGH
- Parallel test execution = data races on process-global state
- Heisenbugs that disappear with `--test-threads=1`

**Security Impact:** MEDIUM
- Tokens/credentials in env vars could leak between tests
- PLATFORM_AUTH_TOKEN in acceptance tests

**Blast Radius:**
- 597 total tests use default parallelism
- Only 3 uses of `#[serial]` in entire codebase

### Plan

**Minimal fix:**

1. **Add `#[serial]` to all env-mutating tests:**
   - telemetry/src/lib.rs: 2 tests
   - doctor.rs: 2 tests
   - security_headers.rs: remaining ~4 tests
   - security_middleware.rs: all 13 tokio tests

2. **Add governance check:**
   - New xtask command: `cargo xtask env-var-safety-check`
   - Fail if env mutation without `#[serial]` marker
   - Add to selftest gates

**Follow-ups:**
- Extract TestConfig struct instead of env vars (ADR needed)
- Add docs/TESTING_GUIDE.md for env var safety patterns

**Test plan:**
```bash
# Verify parallel works after fix
cargo test --test security_middleware -- --test-threads=4
cargo xtask selftest
```

### Decision / Next Action

**RECOMMENDATION: Fix Immediately (P0)**

This is undefined behavior (race conditions on process-global state). The fix is straightforward:
1. Add `#[serial]` to ~20 tests (5 min)
2. Add governance check (15 min)

Should be included in next release cycle.
