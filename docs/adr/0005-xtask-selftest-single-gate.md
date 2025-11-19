# ADR-0005: Selftest as the Single Quality Gate

**Status**: Accepted
**Date**: 2025-01-18
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-002

---

## Context

Quality checks are often scattered:

- Developers run `cargo test` but forget `cargo fmt`
- CI runs clippy but local builds don't
- BDD scenarios pass but unit tests fail
- Policy tests exist but no one runs them

This creates:

- **Inconsistency**: "worked on my machine" → fails in CI
- **Slow feedback**: Discover policy violation after 10min CI run
- **Cognitive load**: "What do I need to run before committing?"
- **Drift**: Local checks diverge from CI over time

We need:

- Single command that validates everything
- Fast enough to run before every commit (~1-2min max)
- Same behavior locally and in CI
- Clear failure messages that guide fixes

---

## Decision

We adopt **`xtask selftest`** as the single, mandatory quality gate:

```bash
cargo run -p xtask -- selftest
```

This runs **5 phases** in order, failing fast on first error:

### Phase 1: Code Quality

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

**Validates:**

- Code formatting (rustfmt)
- Linting rules (clippy, deny warnings)

**Rationale:**

- Catch style issues before review
- Enforce team conventions (no `unwrap()`, etc.)

### Phase 2: Unit and Integration Tests

```bash
cargo test --workspace
```

**Validates:**

- Business logic correctness
- Edge cases, error handling
- Module integration

**Rationale:**

- Fast feedback on core logic
- Isolated from I/O and infrastructure

### Phase 3: AC Mapping Integrity

```bash
cargo run -p xtask -- ac-status
```

**Validates:**

- Ledger ↔ feature file mapping
- AC IDs exist in both directions
- Feature file paths are correct
- Scenario names match

**Rationale:**

- Ensures specs stay in sync with tests
- Detects broken references early

### Phase 4: BDD Scenarios

```bash
cargo run -p acceptance
```

**Validates:**

- End-to-end behavior via Cucumber
- Scenarios tagged with `@AC-XXX-YYY`
- Service actually does what specs claim

**Rationale:**

- Tests behavior contracts, not implementation
- Validates full HTTP stack

### Phase 5: Policy Compliance

```bash
conftest test -p policy/ <targets>
```

**Validates:**

- Kubernetes manifests (security, resources)
- Privacy rules (no PII in logs)
- Feature flags (no prod experiments)
- LLM bundles (size, metadata)

**Behavior:**

- **Locally**: warns if `conftest` missing, continues
- **CI**: fails if `conftest` missing or tests fail

**Rationale:**

- Shift-left on governance
- Catch violations before deploy

---

## Enforcement

**Required workflow:**

1. Make changes (code, specs, tests)
2. Run `cargo run -p xtask -- selftest`
3. Fix failures
4. Commit only when selftest passes

**CI enforcement:**

```yaml
# .github/workflows/ci.yml
- name: Selftest
  run: nix develop -c cargo run -p xtask -- selftest
```

CI fails if selftest fails → blocks merge.

**Pre-commit hook (optional):**

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo run -p xtask -- selftest || exit 1
```

We don't commit this hook (avoids surprising developers), but document it as recommended.

---

## Consequences

### Positive

- **Single command**: No mental model of "what do I run?"
- **Fast feedback**: All checks in <2min (vs 10min full CI)
- **Consistency**: Same checks locally and CI
- **Confidence**: If selftest passes, PR likely merges
- **Documentation**: `xtask selftest --help` explains each phase

### Negative

- **Slow for quick iteration**: Developers may want to skip phases during TDD
  - Mitigation: `xtask check` for fast fmt+clippy+tests only
- **False confidence**: Passing selftest doesn't mean deploy will succeed (e.g., DB migrations)
- **Maintenance burden**: If we add new checks, selftest gets slower

### Neutral

- **Not a replacement for CI**: CI still runs (with caching, parallelism)
- **Not a replacement for integration tests**: Selftest validates the service in isolation, not with real dependencies

---

## Compliance

**Automated:**

- `xtask selftest` is the primary validation command
- CI runs same command (via Nix devshell)
- Failures block merge

**Manual:**

- Code review should ask: "Did you run selftest?"
- If PR fails CI but selftest passed locally, check:
  1. Is local using same Nix environment?
  2. Are there uncommitted changes?

**Detection:**

- CI logs show which selftest phase failed
- Each phase outputs clear error messages with file/line context

---

## Notes

**Why xtask instead of Makefile / Justfile?**

- Rust-native: no shell dependencies, cross-platform
- Strongly typed: args validated at compile time
- Composable: `xtask selftest` can call other xtask commands

We *do* provide a `Justfile` for convenience (`just selftest`), but xtask is the canonical implementation.

**Why 5 phases in order?**

- **Fail fast**: If fmt fails, no point running tests
- **Feedback loop**: Fix cheap things (fmt) before expensive things (BDD)
- **Clarity**: Numbered phases show progress ("✓ 3/5 phases passed")

**What if I want faster iteration?**

For TDD loops:

```bash
# Fast checks only (fmt, clippy, unit tests)
cargo run -p xtask -- check

# Or run a single test
cargo test --package business-core <test-name>
```

Run full selftest before committing.

**What about E2E tests with real DB / queue?**

- Selftest validates the service in isolation (with mocks/fakes)
- Full E2E tests belong in a separate CI job (slower, flakier)
- Selftest is the **commit gate**, E2E is the **deploy gate**

**Migration from scattered checks:**

If you have an existing project:

1. Audit current checks: list what CI runs
2. Create `xtask selftest` that runs each check in sequence
3. Document which phase maps to which original check
4. Update CI to call `xtask selftest` instead of individual commands
5. Train team: "run selftest before commit"

**Adding a new check:**

If you want to add a new validation (e.g., API schema validation):

1. Add check to relevant phase (or create new phase)
2. Update `xtask selftest` output to reflect new check
3. Document in `docs/testing-strategy.md`
4. Ensure check is fast (<30s) or consider moving to CI-only

**References:**

- [Cargo xtask pattern](https://github.com/matklad/cargo-xtask)
- [The Xtask Book](https://github.com/matklad/cargo-xtask#the-xtask-book)
