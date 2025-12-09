# Fork Dry-Run Receipt – v3.3.7-kernel

> **Template:** Copy this file and rename to `FORK_DRY_RUN_YYYY-MM-DD.md` to record
> your fork adoption dry-run results. The goal is to prove that a competent user
> can adopt the kernel following only the docs.

**Service:** [YOUR-SERVICE-NAME]
**Source kernel:** EffortlessMetrics/Rust-Template@v3.3.7-kernel
**Date:** YYYY-MM-DD
**Performed by:** [human | agent]

---

## Summary

- [ ] `cargo xtask selftest` green immediately after checkout
- [ ] `cargo xtask selftest` green after `service-init`
- [ ] `/platform/status` and `/ui` reflect new identity
- [ ] MyService todos ACs still [PASS]
- [ ] No issues requiring unpublished knowledge

**Overall result:** [PASS | PASS WITH ISSUES | FAIL]

---

## Steps and Observations

### 1. Clone + checkout

```bash
git clone https://github.com/EffortlessMetrics/Rust-Template.git [service-name]
cd [service-name]
git checkout v3.3.7-kernel
```

**Outcome:** [OK | ISSUES]

**Notes:**

- [Any observations about the clone step]

---

### 2. Environment setup

```bash
nix develop
cargo xtask dev-up
cargo xtask selftest
```

**Outcome:** [OK | ISSUES]

**Notes:**

- [Were all 11 selftest steps green?]
- [Any environment warnings or issues?]
- [Time to complete?]

---

### 3. Identity change

```bash
cargo xtask service-init \
  --id [your-service-id] \
  --name "[Your Service Name]" \
  --description "[What this service does]" \
  --tags [tag1] [tag2]

cargo xtask selftest
```

**Outcome:** [OK | ISSUES]

**Notes:**

- [Did service-init update all expected files?]
- [Was selftest still green after identity change?]
- [Any files that needed manual editing?]

---

### 4. Introspection validation

**Started app:**

```bash
cargo run -p app-http
```

**Checked endpoints:**

```bash
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/docs/index
curl http://localhost:8080/platform/tasks
```

**Visited UI:** http://localhost:8080/ui

**Outcome:** [OK | ISSUES]

**Notes:**

- [Did /platform/status show the new service_id?]
- [Did /ui reflect the new identity?]
- [Any discrepancies?]

---

### 5. Domain smoke test

**Tested MyService endpoints:**

```bash
curl http://localhost:8080/todos
cargo xtask ac-status
```

**Outcome:** [OK | ISSUES]

**Notes:**

- [Were AC-MYSERV-* still [PASS]?]
- [Any unexpected failures?]

---

## Gaps / Recommended Kernel Improvements

List any issues discovered that should be fed back to the kernel:

| # | Gap Description | Severity | Recommended Fix | Kernel vs Fork |
|---|-----------------|----------|-----------------|----------------|
| 1 | [e.g., docs didn't mention X] | [low\|medium\|high] | [Specific fix] | [kernel\|fork] |
| 2 | [e.g., service-init didn't update Y] | [low\|medium\|high] | [Specific fix] | [kernel\|fork] |
| 3 | [e.g., SPEC_ROOT expectations unclear] | [low\|medium\|high] | [Specific fix] | [kernel\|fork] |

---

## Friction Entry

If gaps were found, create a corresponding friction entry:

- **Friction entry ID:** FRICTION-FORK-DRY-RUN-XXX
- **Location:** `friction/FRICTION-FORK-DRY-RUN-XXX.yaml`
- **Status:** [created | pending]

---

## Success Criteria

The kernel is **fork-ready** when:

- [ ] A fork can change identity via `service-init` without manually editing more than 3 files
- [ ] A fork can reach selftest green with no unapplied patches to the kernel
- [ ] The path from clone → `selftest` → `/ui` is fully covered by existing docs, with no "mystery steps"

**This dry-run confirms:** [All criteria met | Criteria X not met]

---

## Follow-up Actions

- [ ] File friction entry in kernel repo (if gaps found)
- [ ] Update kernel docs based on findings
- [ ] Create PR for any kernel fixes needed
- [ ] Schedule next dry-run after fixes
