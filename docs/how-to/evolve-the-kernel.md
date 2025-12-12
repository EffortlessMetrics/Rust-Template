---
id: HOWTO-EVOLVE-KERNEL-001
title: How to Evolve the Kernel
doc_type: how_to
status: published
# doclint:disable orphan-version
audience: maintainers, kernel-developers
tags: [kernel, evolution, versioning, adr, release]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-009, AC-PLT-010, AC-PLT-011]
adrs: [ADR-0005, ADR-0017]
last_updated: 2025-12-09
---

# How to Evolve the Kernel

> **For kernel maintainers:** This playbook shows the complete ADR → AC → version → tag sequence for evolving the kernel contract.

**Kernel Version:** v3.3.8-kernel

---

## Overview

The kernel is the **stable contract** that forks and IDPs rely on. Evolving it requires care:

```
ADR → Implement → Version Bump → Validate → Tag → Push
```

This playbook walks through each step.

---

## 1. Decision: Is This a Kernel Change?

**Ask yourself:** Does this change affect something downstream consumers depend on?

### Kernel changes (require this playbook):

- Adding/removing/modifying `must_have_ac` ACs in `specs/spec_ledger.yaml`
- Changing `/platform/*` endpoint response shapes
- Modifying `xtask` governance commands (selftest, ac-status, kernel-status, idp-snapshot)
- Updating schema files (`specs/openapi/**`, `specs/platform_schema.yaml`)
- Editing kernel documentation (`docs/KERNEL_SNAPSHOT.md`, `docs/IDP_CELL_CONTRACT.md`)

### NOT kernel changes (skip this playbook):

- Adding new non-kernel ACs (`must_have_ac: false`)
- Updating internal implementation without API changes
- Adding new documentation that doesn't affect contracts
- Fixing typos in non-contract docs

---

## 2. Draft an ADR

**Why:** ADRs document the reasoning so future maintainers understand why changes were made.

```bash
cargo xtask adr-new "Add new platform endpoint for X"
```

**Fill in the ADR:**

```markdown
# ADR-NNNN: Add new platform endpoint for X

## Status
Accepted

## Context
[Why is this change needed? What problem does it solve?]

## Decision
[What exactly are we changing in the kernel contract?]

## Consequences
- Forks will need to [update X / handle new response shape / ...]
- IDP integrations [will / won't] need updates
- Backward compatibility: [yes / no / partial]

## Compliance
- Enforced by: selftest step N, AC-XXX-YYY
- Validated by: BDD scenario in specs/features/xxx.feature
```

---

## 3. Implement the Change

### 3.1 Update spec_ledger.yaml (if adding ACs)

```yaml
# specs/spec_ledger.yaml
acceptance_criteria:
  - id: AC-NEW-001
    description: "New capability description"
    must_have_ac: true  # <-- Makes it a kernel AC
    tests:
      - type: bdd
        file: specs/features/new_feature.feature
        scenario: "Scenario name"
```

### 3.2 Add BDD scenarios

```gherkin
# specs/features/new_feature.feature
@AC-NEW-001
Scenario: New capability works as expected
  Given the service is running
  When I call the new endpoint
  Then I receive the expected response
```

### 3.3 Implement code

- Keep changes scoped to what the AC needs
- Follow existing patterns in the codebase
- Add unit tests mapped to the AC

### 3.4 Update OpenAPI (if adding endpoints)

```yaml
# specs/openapi/openapi.yaml
paths:
  /platform/new-endpoint:
    get:
      summary: New endpoint description
      # ...
```

---

## 4. Choose Version Increment

| Change Type | Example | Bump |
|-------------|---------|------|
| **Patch** | Bug fix, doc clarification, no API change | 3.3.8 → 3.3.9 |
| **Minor** | New AC, new endpoint, backward-compatible | 3.3.8 → 3.4.0 |
| **Major** | Breaking change, removed endpoint, renamed AC | 3.3.8 → 4.0.0 |

**Guidance:**
- If forks can upgrade without code changes → Patch or Minor
- If forks must update their code → Minor or Major
- If existing integrations will break → Major

---

## 5. Bump Version

### 5.1 Update canonical version

```yaml
# specs/spec_ledger.yaml
metadata:
  template_version: "3.4.0"  # New version
  last_updated: "2025-12-09"
```

### 5.2 Run release automation

```bash
cargo xtask release-prepare 3.4.0
```

This updates:
- `README.md` version badge
- `CLAUDE.md` header
- `docs/ROADMAP.md` header
- `docs/KERNEL_SNAPSHOT.md` header
- `docs/explanation/TEMPLATE-CONTRACTS.md` version
- `specs/service_metadata.yaml`
- `specs/doc_index.yaml`
- `CHANGELOG.md` (adds new section)

---

## 6. Validate

### 6.1 Check docs alignment

```bash
cargo xtask docs-check
```

**Expected:** `Version alignment... OK`

### 6.2 Run full selftest

```bash
cargo xtask selftest
```

**Expected:** All 11 steps pass

### 6.3 Check AC coverage

```bash
cargo xtask ac-status --summary
```

**Expected:** All kernel ACs (`must_have_ac=true`) showing `[PASS]`

### 6.4 Verify kernel status

```bash
cargo xtask kernel-status
```

**Expected:** Version shows new number, all sections green

---

## 7. Generate Release Evidence

```bash
cargo xtask release-bundle 3.4.0
```

This creates:
- `release_evidence/v3.4.0.md` – Human-readable release notes
- Links ADR, lists AC changes, includes git changelog

**Review the evidence file** to ensure it accurately reflects the changes.

---

## 8. Commit and Tag

### 8.1 Commit all changes

```bash
git add -A
git commit -m "$(cat <<'EOF'
release: Kernel v3.4.0

- [Brief description of major change]
- ADR-NNNN: [ADR title]
- New ACs: AC-NEW-001, AC-NEW-002
- See release_evidence/v3.4.0.md for details
EOF
)"
```

### 8.2 Tag the kernel

```bash
# Signed tag (recommended)
git tag -s "v3.4.0-kernel" -m "Kernel release v3.4.0"

# Or unsigned (if GPG not configured)
git tag "v3.4.0-kernel" -m "Kernel release v3.4.0"
```

### 8.3 Push

```bash
git push origin main --tags
```

---

## 9. Post-Release

### 9.1 Verify CI

- Check that `tier1-selftest` passes on the new commit
- Check that the tag triggered supply-chain workflow (if configured)

### 9.2 Update downstream docs (if needed)

- Notify fork maintainers of breaking changes
- Update IDP integration docs if response shapes changed

### 9.3 Update roadmap

Add the release to `docs/ROADMAP.md` completed section.

---

## Quick Reference

```bash
# Complete kernel evolution sequence
cargo xtask adr-new "Description"           # 1. ADR
# ... implement changes ...                  # 2. Code
# ... update spec_ledger.yaml ...           # 3. Spec
cargo xtask release-prepare X.Y.Z           # 4. Version
cargo xtask docs-check                      # 5. Validate
cargo xtask selftest                        # 5. Validate
cargo xtask release-bundle X.Y.Z            # 6. Evidence
git add -A && git commit -m "release: ..."  # 7. Commit
git tag -s "vX.Y.Z-kernel" -m "..."         # 8. Tag
git push origin main --tags                 # 9. Push
```

---

## Common Pitfalls

| Pitfall | Prevention |
|---------|------------|
| Forgot to update ADR status | Check ADR is "Accepted" before tagging |
| Version mismatch in docs | Run `cargo xtask docs-check` before commit |
| Missing release evidence | Run `cargo xtask release-bundle` before tag |
| Unsigned tag | Use `git tag -s` for GPG-signed tags |
| CI fails after push | Always run `selftest` locally first |

---

## Kernel AC Guardrails

> **Since v3.3.8:** The kernel has achieved **zero unknowns** (all 72 kernel ACs have test evidence).
> `KERNEL_UNKNOWN_BUDGET=0` is enforced in CI. See [ADR-0024](../adr/0024-ac-evidence-and-kernel-gate.md).

### Adding a New Kernel AC

When adding a new AC with `must_have_ac: true`, you **must** include test mappings:

```yaml
# specs/spec_ledger.yaml
acceptance_criteria:
  - id: AC-NEW-001
    description: "New kernel capability"
    must_have_ac: true
    tags: [kernel]
    tests:  # <-- REQUIRED for kernel ACs
      - type: unit
        file: crates/xtask/src/commands/new.rs
        function: test_new_capability
      # Or BDD:
      - type: bdd
        file: specs/features/new.feature
        scenario: "New capability scenario"
```

**If you cannot provide tests immediately:**

1. **Option A (Recommended):** Use `must_have_ac: false` during development, promote to kernel AC once tests exist
2. **Option B (Requires Review):** Temporarily increase `KERNEL_UNKNOWN_BUDGET` in CI with justification

### Demoting a Kernel AC

To demote an existing kernel AC (set `must_have_ac: false`):

1. Draft an ADR explaining why the AC is being demoted
2. Update `specs/spec_ledger.yaml` with `must_have_ac: false`
3. Document the demotion in the release evidence
4. This requires review approval as it changes the kernel contract

### Verification

Before committing kernel AC changes:

```bash
# Lint the spec ledger for structural issues
cargo xtask ac-lint --strict

# Check kernel AC coverage
cargo xtask ac-status --summary

# Verify zero unknowns
cargo xtask ac-ensure-kernel-mapped

# Run full selftest (will fail if budget exceeded)
KERNEL_UNKNOWN_BUDGET=0 cargo xtask selftest
```

### Spec Ledger Lint (`ac-lint`)

The `ac-lint` command validates `specs/spec_ledger.yaml` for structural issues:

| Check | Description |
|-------|-------------|
| **Duplicate IDs** | No story, REQ, or AC can have the same ID |
| **Naming conventions** | Stories must start with `US-`, REQs with `REQ-`, ACs with `AC-` |
| **Test types** | Only allowed types: `unit`, `bdd`, `integration`, `docs`, `manual`, `ci`, `contract`, `e2e` |
| **Kernel coverage** | Kernel ACs (`must_have_ac: true`) must have at least one test mapping |

**Usage:**

```bash
# Basic lint (errors only)
cargo xtask ac-lint

# Strict mode (warnings become errors)
cargo xtask ac-lint --strict

# Also check that test files exist on disk
cargo xtask ac-lint --strict --check-files
```

**Integrated into selftest:** Step 10 runs `ac-lint --strict` automatically.

---

## Related Documentation

- **[maintain-kernel.md](./maintain-kernel.md)** – Day-to-day kernel maintenance
- **[CONTRIBUTING.md](../../CONTRIBUTING.md#9-how-to-evolve-the-kernel)** – Contributor guide kernel section
- **[KERNEL_SNAPSHOT.md](../KERNEL_SNAPSHOT.md)** – Current kernel baseline
- **[RELEASE_PLAYBOOK.md](../RELEASE_PLAYBOOK.md)** – Full release process
- **[ADR-0005](../adr/0005-xtask-selftest-single-gate.md)** – Selftest as single gate
- **[ADR-0024](../adr/0024-ac-evidence-and-kernel-gate.md)** – AC evidence model and kernel gate
