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
