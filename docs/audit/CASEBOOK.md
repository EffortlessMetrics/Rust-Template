---
id: GUIDE-CASEBOOK-001
title: Casebook (Curated Exhibits)
doc_type: guide
status: published
audience: auditors, reviewers, new-contributors
tags: [casebook, exhibits, examples, archaeology]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: []
last_updated: 2026-01-07
---

# Casebook: Curated Exhibits

This casebook contains curated examples of governed change in this repository.

Each exhibit demonstrates:
- **Bounded scope** — reviewers know where to look
- **Receipt-backed claims** — evidence, not prose
- **Earnest wrongness** — what was fixed and hardened
- **Factory delta** — how the repo got better

> **Note:** This casebook is transitioning to generated output. Once `cargo xtask casebook-gen`
> is implemented, entries will be auto-generated from dossier receipts (`receipts/dossier.json`).
> Manual curation will remain for exhibit selection and scoring commentary.
> See [`RECEIPTS.md`](RECEIPTS.md) for dossier schema details.

---

## How to Read an Exhibit

Each entry follows this structure:

```
### PR #<n> — <Title>
- **What:** 1-sentence summary
- **Scope:** Top directories changed
- **Failure mode:** <from taxonomy> or "none identified"
- **Detection:** How issues were found
- **Prevention:** What gate was added
- **DevLT:** ~X min | **Compute:** ~$Y
- **Exhibit link:** `docs/audit/EXHIBITS/PR-<n>.md`
```

---

## Exhibit Selection Criteria

PRs become exhibits when they demonstrate:

1. **Scope clarity** — bounded change area
2. **Proof completeness** — all claims have receipts
3. **Errata quality** — honest about what was wrong
4. **Factory delta** — improved gates/contracts

Score: 0-25 (5 per criterion + overall coherence)

---

## Curated Exhibits

### PR #76 — Pagination error contract BDD scenarios

- **What:** Added BDD coverage for `/platform/issues` pagination error responses (400s)
- **Scope:** `specs/features/`, `crates/gov-http/`
- **Failure mode:** `claim_drift` risk — 400 behavior could silently regress
- **Detection:** New BDD scenarios exercise mixed params, invalid cursor, oversized cursor, unknown version
- **Prevention:** Contract tests in `specs/features/platform_issues.feature`
- **DevLT:** ~15 min | **Compute:** ~$2
- **Exhibit link:** (pending)

---

### PR #75 — xtask/gov-http hardening after issues merge

- **What:** Stabilized git calls and improved error handling post-issues-endpoint merge
- **Scope:** `crates/xtask/`, `crates/gov-http/`
- **Failure mode:** `env_global_state` — git calls could fail in unexpected CWD
- **Detection:** Integration tests caught edge cases
- **Prevention:** Hardened error handling, explicit CWD management
- **DevLT:** ~20 min | **Compute:** ~$3
- **Exhibit link:** (pending)

---

### PR #74 — Unified issues endpoint and CLI search

- **What:** Added `/platform/issues` aggregating friction, questions, tasks with pagination
- **Scope:** `crates/gov-http/`, `crates/xtask/`, `specs/`
- **Failure mode:** None identified
- **Detection:** Full BDD coverage for pagination, filtering, cross-artifact search
- **Prevention:** Pagination contract with explicit 400 responses
- **DevLT:** ~45 min | **Compute:** ~$8
- **Exhibit link:** (pending)

---

### PR #61 — OpenAPI endpoint

- **What:** Added `/platform/openapi` returning OpenAPI 3.0 spec
- **Scope:** `crates/gov-http/`
- **Failure mode:** `spec_drift` risk — schema could diverge from implementation
- **Detection:** Schema validation in tests
- **Prevention:** OpenAPI generated from same types as handlers
- **DevLT:** ~30 min | **Compute:** ~$4
- **Exhibit link:** (pending)

---

### PR #45 — Blocking pre-commit hook with staged-only semantics

- **What:** Pre-commit hook now blocks on failure; staged-only mode added
- **Scope:** `.git/hooks/`, `crates/xtask/`
- **Failure mode:** `env_global_state` — worktree pollution from fmt on unstaged files
- **Detection:** User friction reports
- **Prevention:** Staged-only Rust policy, clean worktree check
- **DevLT:** ~25 min | **Compute:** ~$3
- **Exhibit link:** (pending)

---

## Pending Exhibits

PRs that meet criteria but need cover sheets:

| PR | Title | Score | Blocker |
|----|-------|-------|---------|
| #43 | Faster precommit defaults | ~18 | Needs receipts |
| #40 | Docs version alignment | ~16 | Needs exhibit file |
| #38 | Security configuration doc | ~15 | Needs exhibit file |
| #33 | Security middleware | ~20 | Needs full cover sheet |

---

## Adding an Exhibit

1. **Ensure the PR has a cover sheet** following `PR_COVER_SHEET.md`
2. **Score the PR** using the selection criteria
3. **Create the exhibit file** at `docs/audit/EXHIBITS/PR-<n>.md`
4. **Add an entry** to this casebook with the summary
5. **Link to the exhibit** from the entry

Template for exhibit file:

```markdown
# PR #<n> — <Title>

## Cover Sheet

[Copy from PR description]

## Errata

[Detailed wrongness and correction]

## Factory Delta

[What improved in the repo]

## Receipts

[Links to evidence in .runs/ or committed artifacts]
```

---

## Casebook Maintenance

This casebook is updated:

- After significant PRs merge (>=3 exhibit score)
- During release preparation (audit recent merges)
- When failure modes are corrected (document the fix)

### Future Automation

`cargo xtask casebook-gen` will build this from dossiers:

1. Read all `receipts/dossier.json` files from merged PRs
2. Score each PR using the selection criteria
3. Generate exhibit entries for PRs meeting the threshold
4. Update this file with new entries, preserving manual commentary

Dossier schema: `specs/schemas/dossier.schema.json`

Until `casebook-gen` exists, maintain this file manually following the process above.
