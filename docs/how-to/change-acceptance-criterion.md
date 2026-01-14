---
id: GUIDE-TPL-CHANGE-AC-001
title: Change an Acceptance Criterion
doc_type: how-to
status: published
audience: developers
tags: [contract, ac, governance, spec]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-AC-MANAGEMENT]
acs: [AC-PLT-003, AC-PLT-004]
adrs: [ADR-0003]
last_updated: 2025-11-27
---

# Change an Acceptance Criterion

**When to use this guide:**
- You're modifying an existing AC's behavior (text, tests, implementation)
- You're demoting a kernel AC to optional
- You're promoting a template AC to kernel
- You're removing an AC entirely

**Time:** 10–30 minutes depending on scope.

---

## Quick Decision Matrix

| Scenario | Path |
|----------|------|
| **Change AC text or behavior** | [Change AC Behavior](#change-ac-behavior) |
| **Update BDD tests** | [Update BDD Tests](#update-bdd-tests) |
| **Demote kernel AC to optional** | [Demote Kernel AC](#demote-kernel-ac) |
| **Promote template AC to kernel** | [Promote to Kernel](#promote-to-kernel) |
| **Remove an AC** | [Remove an AC](#remove-an-ac) |
| **Add a new AC** | See [add-acceptance-criterion.md](./add-acceptance-criterion.md) |

---

## Before You Start

**Know the difference:**

- **Kernel AC** (`must_have_ac: true`) – Non-negotiable. Selftest FAILS if broken.
- **Template AC** (`must_have_ac: false`) – Best practice. Selftest PASSES even if broken, but forks should handle it.
- **Meta/CI AC** – Governance infrastructure ACs. Usually template.

Check the current AC status:

```bash
cargo xtask ac-status          # See all ACs and their status
```

---

## Change AC Behavior

### 1. Update the spec ledger

Open `specs/spec_ledger.yaml` and find your AC:

```yaml
- id: AC-MYSERV-001
  text: "Users can list todos by status filter"
  status: pending
  tests:
    - { type: bdd, tag: "@AC-MYSERV-001", file: "specs/features/todos.feature" }
```

**Edit the text:**

```yaml
- id: AC-MYSERV-001
  text: "Users can list todos by status filter and sort by date"  # Updated text
  tests:
    - { type: bdd, tag: "@AC-MYSERV-001", file: "specs/features/todos.feature" }
```

### 2. Update BDD scenarios

Find the matching `@AC-MYSERV-001` scenario in your feature file (`specs/features/todos.feature`):

```gherkin
@AC-MYSERV-001
Scenario: List todos with status filter
  Given I have todos with various statuses
  When I request todos with status=done
  Then I receive only completed todos
```

Update the scenario to match the new AC text:

```gherkin
@AC-MYSERV-001
Scenario: List todos with status filter and sort by date
  Given I have todos with various statuses and dates
  When I request todos with status=done and sort=date
  Then I receive only completed todos sorted by date descending
```

### 3. Update implementation + tests

Modify your code to match the new behavior. Run focused tests:

```bash
cargo xtask test-ac AC-MYSERV-001    # Test just this AC
```

### 4. Verify the change

```bash
cargo xtask ac-status                # Recompute AC status
cargo xtask test-changed             # Test everything you touched
cargo xtask selftest                 # Full gate
```

**Expected:** Your AC now shows updated text with passing tests.

---

## Update BDD Tests

When you need to fix, skip, or restructure BDD scenarios **without changing the AC definition**:

### Case 1: Fix a broken scenario

Find the failing scenario in `specs/features/*.feature`:

```gherkin
@AC-MYSERV-001
Scenario: List todos with status filter
  Given I have todos with status done
  When I request GET /todos?status=done
  Then status code is 200
  And response contains 5 todos  # ← This assertion is wrong
```

Fix the assertion:

```gherkin
@AC-MYSERV-001
Scenario: List todos with status filter
  Given I have todos with status done
  When I request GET /todos?status=done
  Then status code is 200
  And response contains 3 todos  # ← Fixed count
```

Run:

```bash
cargo xtask test-ac AC-MYSERV-001
```

### Case 2: Skip a flaky scenario temporarily

Use `@skip` tag:

```gherkin
@skip @AC-MYSERV-001
Scenario: List todos (flaky on CI)
  # Scenario marked for later investigation
```

**Note:** Skipped scenarios don't fail selftest. File an issue to track the fix:

```bash
cargo xtask question-new "AC-MYSERV-001: Flaky scenario on CI" \
  --refs AC-MYSERV-001
```

### Case 3: Restructure scenarios for clarity

Split one large scenario into multiple:

```gherkin
@AC-MYSERV-001
Scenario: List todos returns 200
  When I request GET /todos
  Then status code is 200

@AC-MYSERV-001
Scenario: List todos filters by status
  Given I have todos with status done and pending
  When I request GET /todos?status=done
  Then response contains only done todos
```

Run:

```bash
cargo xtask test-ac AC-MYSERV-001
```

---

## Demote Kernel AC

When a kernel AC (`must_have_ac: true`) is no longer essential, demote it to optional.

### 1. Understand the impact

**Before demoting, ask:**
- Why is this no longer kernel?
- What happens if a fork removes it?
- Will CI accept its failure?

**Document your reasoning.** You'll need an ADR or issue.

### 2. Change the spec ledger

```yaml
- id: AC-TPL-GOVERNANCE-ARTIFACTS
  text: "..."
  must_have_ac: true              # ← Change to false
  # Rest of AC definition
```

### 3. Create an ADR or issue

Explain why you're demoting:

```bash
cargo xtask adr-new "Demote AC-TPL-GOVERNANCE-ARTIFACTS to template"
```

Or:

```bash
gh issue create \
  --title "Demote AC-TPL-GOVERNANCE-ARTIFACTS from kernel" \
  --body "This AC is no longer essential because..."
```

### 4. Verify selftest still passes

```bash
cargo xtask selftest
```

**Expected:** Selftest passes even if the demoted AC fails.

### 5. Commit your decision

```bash
git add specs/spec_ledger.yaml docs/adr/ADR-XXXX.md
git commit -m "chore: demote AC-TPL-GOVERNANCE-ARTIFACTS to template

Rationale: This feature is useful but not core to the kernel.
See ADR-XXXX for full reasoning."
```

---

## Promote to Kernel

Rare, but sometimes a template AC becomes essential.

### 1. Change the spec ledger

```yaml
- id: AC-MYSERV-CRITICAL-FEATURE
  text: "..."
  must_have_ac: false             # ← Change to true
```

### 2. Ensure AC is green

Your AC must PASS to promote it to kernel:

```bash
cargo xtask test-ac AC-MYSERV-CRITICAL-FEATURE
# Expected: ✅ PASS
```

If it's red, fix it first.

### 3. Verify selftest passes

```bash
cargo xtask selftest
```

### 4. Commit

```bash
git add specs/spec_ledger.yaml
git commit -m "chore: promote AC-MYSERV-CRITICAL-FEATURE to kernel

This is now essential for the service contract."
```

---

## Remove an AC

Only remove an AC if you're certain it's no longer needed and no other ACs depend on it.

### 1. Verify no dependencies

Check `specs/spec_ledger.yaml` for references to the AC:

```bash
grep -r "AC-MYSERV-OLD-001" specs/
```

### 2. Remove from spec ledger

Delete the entire AC definition from `specs/spec_ledger.yaml`.

### 3. Remove or update BDD scenarios

Delete all scenarios tagged with `@AC-MYSERV-OLD-001`, or retag them if they serve a different purpose:

```gherkin
# Delete this:
@AC-MYSERV-OLD-001
Scenario: Old behavior
  ...

# Or retag if still useful:
@AC-MYSERV-NEW-001  # New AC covers this now
Scenario: Related behavior
  ...
```

### 4. Update docs

Search for AC references in `docs/`:

```bash
grep -r "AC-MYSERV-OLD-001" docs/
```

Remove or update them.

### 5. Verify selftest

```bash
cargo xtask selftest
```

### 6. Commit

```bash
git add specs/spec_ledger.yaml specs/features/ docs/
git commit -m "chore: remove AC-MYSERV-OLD-001 (no longer needed)"
```

---

## Checklist for Any AC Change

Use this before committing:

```
[ ] AC text updated in specs/spec_ledger.yaml (if changed)
[ ] BDD scenarios updated in specs/features/*.feature
[ ] Implementation code matches new AC behavior
[ ] Unit tests updated if needed
[ ] cargo xtask test-ac <AC-ID> passes
[ ] cargo xtask test-changed passes
[ ] cargo xtask ac-status recomputed
[ ] If kernel AC was demoted:
    [ ] ADR or issue filed explaining why
    [ ] docs/feature_status_notes.md updated if needed
[ ] If promoting, changing scope, or removing:
    [ ] Docs (design docs, tutorials) checked for stale references
[ ] cargo xtask selftest passes
```

---

## Examples

### Example 1: Minor text clarification

**AC before:**

```yaml
- id: AC-MYSERV-001
  text: "Service responds to health check"
```

**AC after:**

```yaml
- id: AC-MYSERV-001
  text: "Service responds with 200 OK to GET /health"
```

**BDD update:**

```gherkin
@AC-MYSERV-001
Scenario: Health check succeeds
  When I GET /health
  Then status code is 200
  And response includes {"status": "ok"}
```

**Run:**

```bash
cargo xtask test-ac AC-MYSERV-001
```

### Example 2: Demote optional feature

**Before:**

```yaml
- id: AC-TPL-METRICS-EXPORT
  text: "Platform metrics can be exported to Prometheus"
  must_have_ac: true
```

**After:**

```yaml
- id: AC-TPL-METRICS-EXPORT
  text: "Platform metrics can be exported to Prometheus"
  must_have_ac: false           # Now optional for forks
```

**ADR:**

```bash
cargo xtask adr-new "Metrics export is nice-to-have, not core"

# Edit ADR to explain:
# - Reason: Not all services use Prometheus
# - Impact: Forks can safely remove this feature
# - Implementation: Set must_have_ac: false
```

**Run:**

```bash
cargo xtask selftest  # Should still pass
```

---

## When to Ask for Help

- **AC is kernel and blocking many things?** File an issue with `refs: AC-XXX`.
- **Not sure if change will break contracts?** Check CONSTITUTION.md and TEMPLATE-CONTRACTS.md.
- **Need to coordinate AC change across multiple services?** Use friction-new to log the coordination need.

---

## See Also

- [add-acceptance-criterion.md](./add-acceptance-criterion.md) – Adding new ACs
- [specs/spec_ledger.yaml](../../specs/spec_ledger.yaml) – Current AC definitions
- [docs/CONSTITUTION.md](../CONSTITUTION.md) – Core contracts you can't break
- [docs/KERNEL_SNAPSHOT.md](../KERNEL_SNAPSHOT.md) – Which ACs are kernel
