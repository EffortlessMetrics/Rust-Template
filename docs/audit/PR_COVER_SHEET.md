---
id: GUIDE-PR-COVER-SHEET-001
title: PR Cover Sheet Format
doc_type: guide
status: published
audience: contributors, reviewers
tags: [pr, review, receipts, claims]
stories: [US-TPL-PLT-001]
requirements: []
acs: []
adrs: []
last_updated: 2026-01-07
---

# PR Cover Sheet Format

This document defines the canonical format for PR descriptions in this repository.

---

## Why a Cover Sheet?

Large AI-assisted PRs are normal. Reviewing every line isn't scalable.

Instead, we review:
- **Scope** — what changed (bounded)
- **Receipts** — evidence that gates passed
- **Errata** — what was wrong and how we fixed it
- **Reproduce** — how to verify locally

The cover sheet is **machine-updatable and idempotent**: tools can regenerate it from receipts.

---

## Canonical Format

```markdown
## Cover Sheet

### What changed
- <1-3 sentences describing the change>

### Where to look (review map)
| Area | Files | Why |
|------|-------|-----|
| <domain> | `path/to/files` | <what changed here> |

### Proof (receipts)
| Check | Status | Receipt |
|-------|--------|---------|
| Tests | PASS | `.runs/pr/<n>/<run>/receipts/tests.json` |
| Security | PASS | `.runs/pr/<n>/<run>/receipts/security.json` |
| Policy | PASS | `.runs/pr/<n>/<run>/receipts/gate.json` |
| Perf | N/A | <not applicable / link> |

### Errata (what we got wrong)
- **Wrong:** <what was incorrect>
  - Detected by: <gate / reviewer / receipt> (link)
  - Fix: <commit link>
  - Prevention: <new gate/test added> (link)
- Still open: <issue link if any>

### Unified budget (DevLT dominates)
| Metric | Value | Notes |
|--------|-------|-------|
| DevLT (author) | ~X min | <what the time bought> |
| DevLT (review) | ~X min | |
| Compute spend | ~$X | <what compute reduced: uncertainty, rework> |

### Reproduce locally
```bash
nix develop
cargo xtask selftest
```

<!-- swarm-meta (machine-updated; do not hand edit)
run_id: <run-id>
receipts:
  gate: <path>
  tests: <path>
  security: <path>
  perf: <path>
devlt_minutes:
  author: <n>
  review: <n>
compute:
  tokens_usd: <n>
-->
```

---

## Rules for Each Section

### What changed

- 1-3 sentences maximum
- Focus on *what*, not *why* (that's in the linked issue/AC)
- No marketing language

### Where to look (review map)

- Group by logical area, not file path
- Include the *why* — what changed in this area
- Highlight hotspots (files with most changes)

### Proof (receipts)

- Every claim must have a receipt link
- Use N/A explicitly if a check doesn't apply
- Never claim PASS without a receipt

### Errata

- If nothing was wrong: say so explicitly
- Use the taxonomy from `FAILURE_MODES.md`
- Link the prevention (new gate/test)

### Unified budget

- DevLT is the primary metric (human time dominates)
- Compute is secondary but tracked
- Bands are acceptable ("~10-20 min")
- Note what the time/compute *bought* (reduced uncertainty, caught issues)

### Reproduce locally

- Must be copy-pasteable
- Must work on a fresh clone
- Include environment setup (`nix develop`)

### swarm-meta block

- Machine-updated by `xtask pr-cover` and `xtask pr-update`
- Do not hand-edit
- Used for idempotent updates

---

## When Nothing Was Wrong

Use this errata section:

```markdown
### Errata (what we got wrong)
- Nothing identified in this PR's scope.
- (If you find something later, add an addendum here and link the fixing PR.)
```

---

## Anti-Patterns (Don't Do This)

### Vague scope
```markdown
### What changed
- Updated some files and fixed some issues
```
**Why bad:** Unbounded, reviewer can't focus

### Claims without receipts
```markdown
### Proof
- Tests pass ✅
- Security is good ✅
```
**Why bad:** No evidence, no links

### Hidden wrongness
```markdown
### Errata
- (none)
```
**Why bad if something was wrong:** Silent drift, no factory improvement

### Fake precision
```markdown
### Unified budget
- DevLT: 47 minutes
- Compute: $3.27
```
**Why bad:** False precision suggests certainty we don't have. Use bands.

---

## Generating Cover Sheets

Future tooling (see ROADMAP "Publishing & Forensics" track):

```bash
# Generate cover sheet from receipts
cargo xtask pr-cover --pr <n> --run-dir .runs/pr/<n>/<run-id>

# Update PR description with cover sheet
cargo xtask pr-update --pr <n>
```

Until then, copy this template and fill it manually.

---

## Version-Controlled Exhibits

After a PR merges, the cover sheet can be copied to:

```
docs/audit/EXHIBITS/PR-<n>.md
```

This creates a durable, version-controlled record of the change and its evidence.
