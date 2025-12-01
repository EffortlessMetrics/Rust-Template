---
id: REF-TPL-DOC-SOURCES-001
title: Documentation Sources and Governance
doc_type: reference
status: published
audience: developers, maintainers, agents
tags: [documentation, governance, source-of-truth]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-AC-MANAGEMENT]
acs: [AC-PLT-003, AC-PLT-004]
---

This template treats documentation the same way it treats specs and tests: some files are
**generated views over truth**, some are **governed hand-authored summaries**, and some are
**plain explanation**.

When in doubt:

> Generated docs win. If prose disagrees with generated output, fix the prose.

## 1. Documentation types

We use three categories:

- **Generated docs** – Produced by `cargo xtask` commands. These are views over specs, tests,
  or runtime behaviour.
- **Governed hand-authored docs** – Curated markdown explicitly covered by acceptance criteria.
- **Ungoverned hand-authored docs** – Explanations and guides that are helpful but not enforced
  by ACs.

## 2. Generated docs

These are treated as the **canonical view** of the current kernel state.

| Doc                         | How it's generated              | Notes                            |
|----------------------------|---------------------------------|----------------------------------|
| `docs/feature_status.md`   | `cargo xtask ac-status`        | AC health by story/requirement   |
| `docs/SELECTIVE_TESTING.md`| `cargo xtask docs-selective-testing` (if present) | Change-aware testing ladder |
| `docs/AGENT_GUIDE.md`      | `cargo xtask docs-agent-guide` (if present)       | Agent view over kernel      |

If you suspect drift:

```bash
cargo xtask ac-status
# and any other docs-* commands as they're added
```

**Important:** Generated docs should never be hand-edited. They are overwritten by their generator.

## 3. Governed hand-authored docs

These are written by humans but **anchored in the spec** via REQs/ACs:

| Doc                                          | Purpose                                | Backed by                                                |
| -------------------------------------------- | -------------------------------------- | -------------------------------------------------------- |
| `docs/KERNEL_SNAPSHOT.md`                    | Snapshot of kernel guarantees & limits | REQ-TPL-OPINIONATED-DEFAULTS, AC-TPL-OPINIONS-DOCUMENTED |
| `docs/feature_status_notes.md`               | How to read feature status & AC types  | REQ-PLT-AC-MANAGEMENT                                    |
| `docs/how-to/change-acceptance-criterion.md` | Day-2 contract change process          | REQ-PLT-AC-MANAGEMENT                                    |
| `docs/how-to/FIRST_FORK.md`                  | First fork runbook                     | REQ-PLT-ONBOARDING                                       |

These files should be updated whenever:

* The underlying REQs/ACs in `specs/spec_ledger.yaml` change, or
* The associated `cargo xtask` workflows change in behaviour.

If they drift from generated views (e.g. `docs/feature_status.md`), treat that as a bug in the docs.

## 4. Ungoverned hand-authored docs

These help humans reason about the system but are **not enforced** by ACs:

* `docs/explanation/*.md`
* Some `docs/reference/*.md` (environment, supply chain hardening, etc.)
* `docs/how-to/*` that aren't yet tied to explicit REQs/ACs

They can evolve more freely, but should not contradict:

* Generated docs
* Governed hand-authored docs

## 5. When docs disagree

If you see conflicting information:

1. Trust **generated docs** first.
2. If a governed hand-authored doc disagrees, update that doc and reference the relevant ACs.
3. Only if the generated output is wrong should you change specs or tests so that:

   * `specs/spec_ledger.yaml` matches reality, and
   * `cargo xtask ac-status` and related generators reflect the new truth.

Agents should prefer:

* `cargo xtask ac-status --json` as the machine contract view.
* `docs/feature_status.md` and `docs/KERNEL_SNAPSHOT.md` as human-oriented summaries.

---

## 6. Docs-as-Code Invariants

This section documents the **code-enforced invariants** that keep docs in sync with specs. When in doubt about what's enforced, look here.

### 6.1. Invariants enforced by `cargo xtask docs-check`

| Invariant | Files Affected | What's Checked |
|-----------|----------------|----------------|
| **Version alignment (AC-PLT-009)** | README, CLAUDE, ROADMAP, KERNEL_SNAPSHOT, TEMPLATE-CONTRACTS, service_metadata, doc_index, CHANGELOG | `specs/spec_ledger.yaml → metadata.template_version` must match all versioned docs |
| **Doc index ↔ frontmatter sync (AC-PLT-DOC-INDEX-FRONTMATTER)** | All indexed docs in `specs/doc_index.yaml` | Every indexed doc's frontmatter must match doc_index; every doc with frontmatter must appear in the index |
| **Feature status header (AC-PLT-010 extension)** | `docs/feature_status.md` | Header must contain Template Version matching spec_ledger |
| **ADR structure** | `docs/adr/ADR-*.md` | ADR format and numbering validated |
| **Kernel REQs must have docs** | `specs/spec_ledger.yaml`, `specs/doc_index.yaml` | Every REQ with `must_have_ac: true` (or containing such ACs) must appear in `requirements:` for at least one doc (currently soft warning) |
| **Skills definitions** | `.claude/skills/*/SKILL.md` | skills-lint validates format, descriptions, tools, and no secrets |

### 6.2. Invariants enforced by `cargo xtask ac-status`

| Invariant | What's Checked |
|-----------|----------------|
| **AC → test mapping** | Every AC in spec_ledger must have test coverage (BDD or unit) |
| **feature_status.md generation** | File is regenerated from spec_ledger + test results |
| **AC counts** | Total, passing, failing, unknown counts are computed from source |

### 6.3. Invariants enforced by `cargo xtask selftest`

Selftest runs docs-check as one of its gates. It also validates:

* Graph invariants (REQ → AC → test → doc)
* Policy tests via conftest
* BDD scenario execution
* Unit test execution

### 6.4. Doc maintenance runbook

When you change docs, follow this pattern:

**Changing versioned docs (README, ROADMAP, KERNEL_SNAPSHOT, etc.):**
1. Update `specs/spec_ledger.yaml → metadata.template_version` first
2. Run `cargo xtask release-prepare X.Y.Z` to bump all files
3. Run `cargo xtask docs-check` to verify alignment

**Adding/modifying indexed docs:**
1. Add frontmatter with required fields (`id`, `doc_type`, `stories`, `requirements`, `acs`, `adrs`)
2. Add entry to `specs/doc_index.yaml` with matching values
3. Run `cargo xtask docs-check` to verify sync

**Changing AC classifications or soft AC docs:**
1. Update `specs/spec_ledger.yaml` (AC definitions, `must_have_ac` flags)
2. Update `docs/feature_status_notes.md` with reasoning
3. Run `cargo xtask ac-status` to regenerate feature_status.md
4. Run `cargo xtask selftest` to verify governance

**Changing Skills or Agents:**
1. Edit `.claude/skills/*/SKILL.md` or `.claude/agents/*.md`
2. Run `cargo xtask skills-lint` or `cargo xtask agents-lint`
3. Run `cargo xtask docs-check` to verify
