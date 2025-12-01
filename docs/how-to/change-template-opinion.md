---
id: GUIDE-TPL-OVERRIDE-001
title: Change Template Opinions in Your Fork
doc_type: how-to
status: published
audience: fork-maintainers, platform-engineers
tags: [fork, customization, governance, override, opinion]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DOC-TEMPLATES, REQ-TPL-OVERRIDE-PATH, REQ-TPL-EXAMPLE-FORK]
acs: [AC-TPL-OVERRIDE-TRACEABLE]
adrs: [ADR-0005]
last_updated: 2025-11-26
---

# Change Template Opinions in Your Fork

This guide explains how to safely override or relax the template's built-in opinions when they don't fit your organization's needs.

**Time:** 15-30 minutes per opinion change
**Prerequisites:** Active fork, understanding of spec_ledger.yaml and BDD

---

## Why This Matters

This template is **opinionated by design**. Opinions like "Nix is required," "BDD scenarios must cover all ACs," and "releases require signed SBOMs" are encoded as **Acceptance Criteria (ACs)** in `specs/spec_ledger.yaml`.

When you fork the template:

- **Keep opinions** that align with your organization (zero work required)
- **Change opinions** by updating or removing ACs (this guide)
- **Add opinions** by creating new REQs and ACs (standard AC-first workflow)

**The key principle:** Don't hack around the system by patching CI scripts or adding .env bypasses. Change the spec, and let the governed system validate your changes.

---

## The Override Process

Follow these steps to change any template opinion:

### Step 1: Identify the Opinion

Find the relevant REQ and AC in `specs/spec_ledger.yaml`.

**Common opinion locations:**

- **Nix requirement:** `REQ-PLT-ONBOARDING` → `AC-PLT-001`
- **Governance artifacts (friction, forks, questions):** `REQ-TPL-GOV-ARTIFACTS` → `AC-TPL-GOV-FRICTION`, `AC-TPL-GOV-FORKS`
- **Supply chain checks:** `REQ-PLT-SUPPLY-CHAIN` → `AC-PLT-007`, `AC-PLT-008`
- **Documentation validation:** `REQ-PLT-DOCS-CONSISTENCY` → `AC-PLT-009`, `AC-PLT-010`
- **Release workflow:** `REQ-PLT-RELEASE-SAFETY` → `AC-PLT-011`, `AC-PLT-012`, `AC-PLT-013`

**Example:** Let's say you want to relax the Nix requirement for onboarding.

```yaml
# In specs/spec_ledger.yaml, find:
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance"
  tags: [kernel]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-PLT-001", file: "specs/features/xtask_devex.feature" }
```

### Step 2: Update the AC in Your Fork

Choose one of three strategies:

#### Strategy A: Relax the requirement (recommended)

Update the AC text to reflect your new rule:

```yaml
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust, git, and optionally warns about Nix and conftest"
  tags: [kernel]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-PLT-001", file: "specs/features/xtask_devex.feature" }
```

#### Strategy B: Mark as optional

Set `must_have_ac: false` to demote it to "nice to have":

```yaml
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance"
  tags: [template]  # Changed from [kernel]
  must_have_ac: false  # Changed from true
  note: "Nix is optional in our fork. Native tooling is acceptable."
  tests:
    - { type: integration, tag: "@AC-PLT-001", file: "specs/features/xtask_devex.feature" }
```

#### Strategy C: Remove entirely

Delete the AC from the ledger (rare, only if truly not applicable):

```yaml
# Remove AC-PLT-001 entirely
# Update the REQ to reflect that Nix checking is no longer a requirement
```

### Step 3: Update the BDD Scenario

Find the corresponding scenario in `specs/features/*.feature` and either:

**Option 1:** Update the scenario to match new behavior

```gherkin
# In specs/features/xtask_devex.feature
@AC-PLT-001
Scenario: Doctor command validates environment (Nix optional)
  When I run "cargo xtask doctor"
  Then the command succeeds
  And stdout contains "Rust:"
  And stdout contains "Git:"
  And if Nix is not installed, stdout contains "Nix: ⚠️  optional"
```

**Option 2:** Mark scenario as `@skip` if AC is now `must_have_ac: false`

```gherkin
@AC-PLT-001 @skip
Scenario: Doctor command validates environment (Nix required)
  # Skipped in fork - Nix is optional for us
```

**Option 3:** Remove the scenario entirely if AC was deleted

```gherkin
# Delete scenario entirely
```

### Step 4: Update Implementation (if needed)

If you relaxed or changed the behavior, update the code:

```rust
// In crates/xtask/src/commands/doctor.rs
fn check_nix() -> Result<String> {
    // Before: Required check, fail if missing
    // After: Optional check, warn if missing
    match which::which("nix-shell") {
        Ok(_) => Ok("✓ Nix installed".to_string()),
        Err(_) => Ok("⚠️  Nix not found (optional)".to_string()),  // Changed from Err
    }
}
```

### Step 5: Validate with the Ladder

Run the validation ladder to ensure your changes are clean:

```bash
# Quick validation
cargo xtask check

# Run affected tests
cargo xtask test-changed

# Verify AC status (should show your changes)
cargo xtask ac-status

# Full governance check (this is the final gate)
cargo xtask selftest
```

**Expected outcome:** Selftest should be **green** after your changes. If it's red, either:

- You missed updating a test or implementation
- The AC is referenced by other parts of the system (check dependencies)
- You found a legitimate issue (see Step 6)

### Step 6 (Optional): Log Friction or Questions

If changing the opinion was harder than expected, or if the right choice is unclear:

**For process/tooling friction:**

```bash
cargo xtask friction-new
# Title: "Relaxing Nix requirement required manual BDD changes"
# Severity: medium
# Description: "Had to manually update 3 feature files. Would be nice if ac-status suggested which scenarios to update."
```

**For ambiguity:**

```bash
# Create questions/Q-001-nix-optional.md
cat > questions/Q-001-nix-optional.md <<EOF
---
id: Q-001
title: Should CI enforce Nix or allow native tooling?
status: decided
decision_date: 2025-11-26
decided_by: team-platform
---

## Question
Should our fork require Nix for CI, or allow both Nix and native Rust tooling?

## Context
Template assumes Nix. We have Windows-heavy teams.

## Options
1. Keep Nix required (ensure consistency)
2. Make Nix optional, test both paths in CI (flexibility)

## Decision
Option 2. AC-PLT-001 changed to make Nix optional.
CI will run both Tier-1 (Nix) and Tier-2 (native) checks.

## Rationale
Developer friction > environment purity. We'll catch integration issues in staging.
EOF
```

**For documentation:**

Update `docs/adr/` if the change has architectural implications:

```bash
# Create ADR-FORK-001.md
cat > docs/adr/ADR-FORK-001.md <<EOF
---
id: ADR-FORK-001
title: Nix is Optional in Onboarding
status: accepted
date: 2025-11-26
supersedes: [ADR-0002]  # Template's "Nix-first" ADR
---

## Context
Our fork operates in Windows-heavy environments where Nix adoption is slow.

## Decision
Make Nix optional for local development. CI will run both Nix and native tests.

## Consequences
- Faster onboarding for Windows developers
- Slightly more CI complexity (two matrix jobs)
- Risk of environment drift between Nix and native (mitigated by testing both)

## Affected ACs
- AC-PLT-001: Updated to make Nix optional
EOF
```

---

## Worked Examples

This section shows **complete, real-world override patterns** with clear motivations. These examples demonstrate when and why a fork would make specific changes.

---

### Worked Example A: Turn Off JSON CLI Core

**When to use this:** Your fork doesn't need AI/IDP integration hooks yet. You're building a traditional service and don't want the `/platform/*` endpoints to expose JSON CLI data for agent consumption.

**Why a fork would do this:**

- The service is human-operated only (no AI agents or internal developer portals)
- You want to reduce surface area and complexity
- You're using the template for a simpler use case that doesn't need the AI-first features

**The change:**

```yaml
# In your fork's specs/spec_ledger.yaml
- id: AC-TPL-CLI-JSON-CORE
  text: >
    All xtask commands that provide structured output implement --json,
    surfaced via /platform/devex/flows and used by AI agents and IDPs
    for programmatic access.
  tags: [template, ai, idp]  # Keep as template, not kernel
  must_have_ac: false  # Changed from true - this is now optional in our fork
  note: "JSON CLI disabled in this service; portal uses /platform/status only for runtime metrics."
  tests:
    - { type: integration, tag: "@AC-TPL-CLI-JSON-CORE", file: "specs/features/xtask_json.feature" }
```

**What this achieves:**

- The AC remains in the ledger (for template sync), but is **not enforced**
- Your fork can skip implementing `--json` flags for xtask commands
- `/platform/devex/flows` can be simplified or removed
- Selftest won't fail if JSON CLI features are missing

**Follow-up steps:**

1. Mark BDD scenarios as `@skip` in `specs/features/xtask_json.feature`
2. Optionally remove JSON serialization code from xtask commands
3. Update `/platform/devex/flows` handler to return a minimal response or 404
4. Run `cargo xtask selftest` to confirm the change is clean

**When NOT to do this:**

- If you're building an IDP or agent-first platform (keep it `must_have_ac: true`)
- If you want to preserve the option for future AI integration (keep the AC, just don't prioritize it)

---

### Worked Example B: Harden Friction to Kernel

**When to use this:** Your fork treats **process friction** as a first-class governance gate. You want to **require** friction logging, not just recommend it.

**Why a fork would do this:**

- You're building a high-governance environment (finance, healthcare, infrastructure)
- You want mandatory process improvement tracking, not optional
- You need to prove to auditors that process issues are captured and reviewed

**The change:**

```yaml
# In your fork's specs/spec_ledger.yaml
- id: AC-TPL-GOV-FRICTION
  text: >
    Friction log entries are stored as structured files under friction/,
    validated by cargo xtask friction-validate, and surfaced via
    /platform/friction for review by maintainers and automation.
  tags: [kernel, governance]  # Changed from [template] to [kernel] - now part of core contracts
  must_have_ac: true  # Already true in template, but emphasized here
  note: "This service treats friction logging as a first-class gate. All process issues must be logged."
  tests:
    - { type: integration, tag: "@AC-TPL-GOV-FRICTION", file: "specs/features/friction.feature" }
```

**What this achieves:**

- Friction logging is now **kernel behavior** (can't be turned off in downstream forks)
- CI can fail if friction entries are malformed or missing required fields
- `/platform/friction` becomes a mandatory API endpoint
- Selftest validates that friction validation is working

**Follow-up steps:**

1. Update CI to enforce `cargo xtask friction-validate` in Tier-1 gate
2. Add linting rules to require friction entries for certain types of PRs (e.g., "if PR touches >5 files, must include friction log or exemption note")
3. Integrate `/platform/friction` with your incident review process
4. Run `cargo xtask selftest` to confirm enforcement is active

**When NOT to do this:**

- If your team is small and process tracking is informal (keep it `[template]`)
- If you use external tools like Jira for process feedback (see Example 2 in the guide)
- If you're not ready to enforce this level of governance rigor

---

## Examples

### Example 1: Relax the Nix Requirement

**Goal:** Allow developers to use native Rust tooling without Nix.

**Changes:**

1. **Update AC-PLT-001** in `specs/spec_ledger.yaml`:

   ```yaml
   - id: AC-PLT-001
     text: "`cargo xtask doctor` validates Rust and git, optionally checks Nix and conftest"
     tags: [kernel]
     must_have_ac: true
     note: "Nix is recommended but not required in this fork"
   ```

2. **Update BDD** in `specs/features/xtask_devex.feature`:

   ```gherkin
   @AC-PLT-001
   Scenario: Doctor validates core tools (Nix optional)
     When I run "cargo xtask doctor"
     Then the command succeeds
     And stdout contains "Rust:"
     And stdout contains "Git:"
   ```

3. **Update implementation** in `crates/xtask/src/commands/doctor.rs`:

   ```rust
   // Change Nix check from hard requirement to soft warning
   ```

4. **Validate:**

   ```bash
   cargo xtask check
   cargo xtask test-ac AC-PLT-001
   cargo xtask selftest
   ```

5. **Document:**

   ```bash
   echo "## Fork Differences\n- Nix is optional (AC-PLT-001 relaxed)" >> README.md
   ```

---

### Example 2: Remove a Governance Artifact Type

**Goal:** Your fork doesn't need the friction log (you use Jira for process feedback).

**Changes:**

1. **Mark AC as optional** in `specs/spec_ledger.yaml`:

   ```yaml
   - id: AC-TPL-GOV-FRICTION
     text: "Friction log entries are stored as structured files under friction/, ..."
     tags: [template]  # Changed from [kernel]
     must_have_ac: false  # Changed from true
     note: "We use Jira for process feedback, not friction log"
   ```

2. **Skip BDD** in `specs/features/friction.feature`:

   ```gherkin
   @AC-TPL-GOV-FRICTION @skip
   Feature: Friction Log Management
     # Skipped in fork - we use Jira
   ```

3. **Remove from platform API** (optional):

   ```rust
   // In crates/platform-http/src/routes.rs
   // Comment out or remove:
   // .route("/platform/friction", get(handlers::get_friction))
   ```

4. **Validate:**

   ```bash
   cargo xtask ac-status  # Should show AC-TPL-GOV-FRICTION as optional
   cargo xtask selftest
   ```

5. **Document:**

   ```bash
   cat > docs/adr/ADR-FORK-002.md <<EOF
   ---
   id: ADR-FORK-002
   title: Use Jira Instead of Friction Log
   ---
   We disable AC-TPL-GOV-FRICTION and use Jira for process feedback.
   EOF
   ```

---

### Example 3: Disable Fork Registry Visibility in Platform Status

**Goal:** Your fork doesn't want fork registry data exposed in `/platform/status` (you use an internal registry instead).

**Changes:**

1. **Mark AC as optional** in `specs/spec_ledger.yaml`:

   ```yaml
   - id: AC-TPL-FORKS-STATUS-SUMMARY
     text: >
       /platform/status includes governance.forks.total and a forks.ids
       array when forks/fork_registry.yaml exists, and
       `cargo xtask fork-list --json` reflects that state.
     tags: [template]  # Changed from [kernel, governance, idp]
     must_have_ac: false  # Changed from true
     note: "We use internal registry. Fork visibility removed from platform status."
   ```

2. **Update or skip BDD** in `specs/features/platform_schema.feature`:

   ```gherkin
   @AC-TPL-FORKS-STATUS-SUMMARY @skip
   Scenario: Platform status includes fork registry
     # Skipped in fork - we use internal registry
   ```

3. **Update implementation** in `crates/platform-http/src/handlers/platform_status.rs`:

   ```rust
   // Remove or comment out fork registry inclusion
   // governance.forks = Some(fork_data);
   ```

4. **Validate:**

   ```bash
   cargo xtask ac-status  # Should show AC-TPL-FORKS-STATUS-SUMMARY as optional
   cargo xtask test-changed
   cargo xtask selftest
   ```

5. **Document:**

   ```bash
   cat > docs/adr/ADR-FORK-003.md <<EOF
   ---
   id: ADR-FORK-003
   title: Use Internal Registry for Forks
   status: accepted
   ---
   We disable AC-TPL-FORKS-STATUS-SUMMARY as we use our internal fork registry.
   Platform status won't expose fork data via /platform/status.
   EOF
   ```

---

### Example 4: Relax BDD Harness Exit Code Requirements

**Goal:** Your fork wants @wip scenarios to cause CI failures (stricter than template default).

**Changes:**

1. **Update AC** in `specs/spec_ledger.yaml`:

   ```yaml
   - id: AC-TPL-BDD-EXIT-CODES
     text: >
       The acceptance test binary returns exit 0 when all scenarios pass
       (including @wip scenarios must pass), and returns non-zero if any
       scenario fails.
     tags: [kernel, testing]
     must_have_ac: true
     note: "Fork requires @wip scenarios to pass in CI."
   ```

2. **Update BDD** in `specs/features/bdd_harness.feature`:

   ```gherkin
   @AC-TPL-BDD-EXIT-CODES
   Scenario: WIP scenarios cause non-zero exit
     Given I have a feature with @wip scenarios
     When I run the BDD harness
     Then the exit code is non-zero
   ```

3. **Update implementation** in BDD test runner configuration:

   ```rust
   // Configure harness to fail on @wip scenarios
   // (implementation details depend on your BDD framework)
   ```

4. **Validate:**

   ```bash
   cargo xtask test-ac AC-TPL-BDD-EXIT-CODES
   cargo xtask selftest
   ```

---

### Example 5: Remove Artifact Refs Requirement

**Goal:** Your fork doesn't need traceability between governance artifacts and REQ/AC IDs (you use Jira for traceability).

**Changes:**

1. **Mark AC as optional** in `specs/spec_ledger.yaml`:

   ```yaml
   - id: AC-TPL-ARTIFACTS-HAVE-REFS
     text: >
       Questions and friction entries support a 'refs' field for REQ-*/AC-* IDs,
       allowing governance artifacts to reference the contracts they relate to.
     tags: [template]  # Changed from [kernel, traceability, governance]
     must_have_ac: false  # Changed from true
     note: "We use Jira for traceability. Artifact refs not required."
   ```

2. **Update schema** (optional) - remove refs validation from YAML schemas:

   ```yaml
   # In schemas for questions/friction artifacts
   # refs: # ← Remove or make optional
   ```

3. **Update BDD** (if tests exist):

   ```gherkin
   @AC-TPL-ARTIFACTS-HAVE-REFS @skip
   Feature: Artifact References
     # Skipped - we use Jira for traceability
   ```

4. **Validate:**

   ```bash
   cargo xtask ac-status  # Should show AC-TPL-ARTIFACTS-HAVE-REFS as optional
   cargo xtask selftest
   ```

5. **Document:**

   ```bash
   cat > docs/adr/ADR-FORK-004.md <<EOF
   ---
   id: ADR-FORK-004
   title: Use Jira for Artifact Traceability
   status: accepted
   ---
   We disable AC-TPL-ARTIFACTS-HAVE-REFS as we use Jira for linking
   governance artifacts to requirements. The 'refs' field is optional.
   EOF
   ```

---

### Example 6: Add a New Required Check

**Goal:** Your organization requires FIPS-compliant cryptography. Add an AC to enforce it.

**Changes:**

1. **Add AC** under an existing or new REQ in `specs/spec_ledger.yaml`:

   ```yaml
   - id: REQ-FORK-SECURITY
     title: "FIPS Compliance"
     tags: [security, compliance]
     must_have_ac: true
     acceptance_criteria:
       - id: AC-FORK-SEC-001
         text: "`cargo xtask audit` fails if non-FIPS crypto crates are detected"
         tags: [kernel]
         must_have_ac: true
         tests:
           - { type: integration, tag: "@AC-FORK-SEC-001", file: "specs/features/security.feature" }
   ```

2. **Add BDD** in `specs/features/security.feature` (create if needed):

   ```gherkin
   @AC-FORK-SEC-001
   Scenario: Audit rejects non-FIPS crypto
     Given I add "ring = 0.16" to Cargo.toml
     When I run "cargo xtask audit"
     Then the command fails
     And stderr contains "FIPS violation"
   ```

3. **Implement** in `crates/xtask/src/commands/audit.rs`:

   ```rust
   // Add FIPS crypto check to audit command
   ```

4. **Validate:**

   ```bash
   cargo xtask test-ac AC-FORK-SEC-001
   cargo xtask selftest
   ```

5. **Update CI:**

   ```yaml
   # In .github/workflows/tier1-selftest.yml
   # Ensure `cargo xtask audit` is part of the Tier-1 gate
   ```

---

## What NOT to Do

These are **anti-patterns** that break governance:

### ❌ Don't Patch CI to Skip Checks

**Bad:**

```yaml
# .github/workflows/tier1-selftest.yml
- name: Run selftest
  run: cargo xtask selftest --skip-nix  # NO! This bypasses the spec
```

**Good:**
Change the spec (`AC-PLT-001`) and let `selftest` reflect your new rule.

---

### ❌ Don't Delete Feature Files Without Updating the Ledger

**Bad:**

```bash
rm specs/features/friction.feature  # NO! This creates orphaned ACs
```

**Good:**

1. Mark ACs as `must_have_ac: false` in `spec_ledger.yaml`
2. Then update or remove feature files
3. Run `cargo xtask ac-status` to verify no orphans

---

### ❌ Don't Add .env Hacks to Bypass Validation

**Bad:**

```bash
# .env
SKIP_SELFTEST=true  # NO! This makes governance optional
```

**Good:**

Change the ACs so your desired behavior is the **governed behavior**.

---

### ❌ Don't Silently Change Behavior Without Updating Specs

**Bad:**

```rust
// In doctor.rs - just remove Nix check without updating spec
// fn check_nix() { /* deleted */ }
```

**Good:**

1. Update `AC-PLT-001` text
2. Update BDD scenario
3. Update implementation
4. Run `cargo xtask selftest` to validate alignment

---

## Getting Help

### If the Override Process is Hard

**Use the friction log:**

```bash
cargo xtask friction-new
# Title: "Changing AC-PLT-001 required touching 5 files"
# Severity: medium
# Description: "Would be helpful to have `cargo xtask ac-relax AC-PLT-001` command."
```

This helps the upstream template improve the override DX for future forks.

### If the Right Choice is Unclear

**Create a question artifact:**

```bash
# questions/Q-00X-topic.md
---
id: Q-00X
title: Should we require X or make it optional?
status: open
---

## Context
...

## Options
1. ...
2. ...

## Recommendation
(or leave blank for team discussion)
```

Then reference it in your PR or team Slack for async decision-making.

### If You're Unsure About an AC's Purpose

**Check linked ADRs:**

```yaml
# In spec_ledger.yaml
- id: AC-PLT-001
  adr: ADR-0002  # ← Read this to understand why Nix was chosen
```

Read `docs/adr/ADR-0002.md` to understand the original reasoning. Your fork may have different constraints that justify a different choice.

---

## Summary

**Changing template opinions is a first-class workflow:**

1. **Find the AC** in `specs/spec_ledger.yaml`
2. **Update or remove it** (change text, set `must_have_ac: false`, or delete)
3. **Update BDD scenarios** to match
4. **Update implementation** if needed
5. **Run `cargo xtask selftest`** to validate
6. **Log friction or questions** if the process was unclear

**The system is designed for this.** Don't hack around it—change the spec, and let governance validate your new rules.

---

## Related Guides

- [docs/how-to/new-service-from-template.md](./new-service-from-template.md) - Forking basics
- [docs/how-to/pre-fork-checklist.md](./pre-fork-checklist.md) - Before forking
- [docs/how-to/report-fork-feedback.md](./report-fork-feedback.md) - Sending feedback upstream
- [TEMPLATE-CONTRACTS.md](../../TEMPLATE-CONTRACTS.md) - What the kernel guarantees
- [docs/explanation/governance-model.md](../explanation/governance-model.md) - How governance works
