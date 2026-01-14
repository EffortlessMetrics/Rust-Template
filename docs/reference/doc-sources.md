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

- The underlying REQs/ACs in `specs/spec_ledger.yaml` change, or
- The associated `cargo xtask` workflows change in behaviour.

If they drift from generated views (e.g. `docs/feature_status.md`), treat that as a bug in the docs.

## 4. Ungoverned hand-authored docs

These help humans reason about the system but are **not enforced** by ACs:

- `docs/explanation/*.md`
- Some `docs/reference/*.md` (environment, supply chain hardening, etc.)
- `docs/how-to/*` that aren't yet tied to explicit REQs/ACs

They can evolve more freely, but should not contradict:

- Generated docs
- Governed hand-authored docs

## 5. When docs disagree

If you see conflicting information:

1. Trust **generated docs** first.
2. If a governed hand-authored doc disagrees, update that doc and reference the relevant ACs.
3. Only if the generated output is wrong should you change specs or tests so that:

   - `specs/spec_ledger.yaml` matches reality, and
   - `cargo xtask ac-status` and related generators reflect the new truth.

Agents should prefer:

- `cargo xtask ac-status --json` as the machine contract view.
- `docs/feature_status.md` and `docs/KERNEL_SNAPSHOT.md` as human-oriented summaries.

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
| **Doc type contracts** | `specs/doc_index.yaml` | Each doc_type must satisfy minimal reference expectations (see Section 6.5) (soft warning) |
| **Skills definitions** | `.claude/skills/*/SKILL.md` | skills-lint validates format, descriptions, tools, and no secrets |

### 6.2. Invariants enforced by `cargo xtask ac-status`

| Invariant | What's Checked |
|-----------|----------------|
| **AC → test mapping** | Every AC in spec_ledger must have test coverage (BDD or unit) |
| **feature_status.md generation** | File is regenerated from spec_ledger + test results |
| **AC counts** | Total, passing, failing, unknown counts are computed from source |

### 6.3. Invariants enforced by `cargo xtask selftest`

Selftest runs docs-check as one of its gates. It also validates:

- Graph invariants (REQ → AC → test → doc)
- Policy tests via conftest
- BDD scenario execution
- Unit test execution

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

### 6.4.1 Example: docs-check catching README version drift <!-- doclint:disable orphan-version -->

If `README.md` claims `Template Version: vX.Y.Z` but `specs/spec_ledger.yaml` has a different `template_version` (say `X.Y.W`), the `cargo xtask docs-check` "Version alignment" gate will catch it:

```text
📚 Checking documentation consistency...

  Canonical version (specs/spec_ledger.yaml): X.Y.W
  README.md version: X.Y.Z (expected X.Y.W) ✗
  ...

Version mismatches found:
  • README.md has 'X.Y.Z', expected 'X.Y.W' (pattern: H1: # ... (vX.Y.Z))

To fix:
  1. Update the canonical version: specs/spec_ledger.yaml → metadata.template_version
  2. Or run: cargo xtask release-prepare X.Y.W to bump all files
  3. Commit changes and verify: cargo xtask selftest

Error: Version alignment failed: 1 file(s) out of sync with specs/spec_ledger.yaml (vX.Y.W)
Version alignment... ✗ Mismatch
```

**Typical fix paths:**

1. **You made a mistake (not actually releasing a new version):**
   - Revert README.md back to the canonical version from `specs/spec_ledger.yaml`
   - Run `cargo xtask docs-check` to verify alignment

2. **You're doing a proper version bump:**
   - Update `specs/spec_ledger.yaml → metadata.template_version` to the new version
   - Run `cargo xtask release-prepare X.Y.Z` to apply changes consistently to all versioned files
   - Run `cargo xtask docs-check` and `cargo xtask selftest` to verify

This demonstrates the governance principle: **`specs/spec_ledger.yaml` is THE source of truth** for the template version. All other versioned files (README.md, CLAUDE.md, ROADMAP.md, etc.) must agree with it—they are derived views, not independent sources.

### 6.5. Doc type contracts

Each `doc_type` carries light structural expectations. These are enforced as **soft checks** by `cargo xtask docs-check`:

| doc_type          | Purpose                      | Minimal expectations                          |
|-------------------|------------------------------|-----------------------------------------------|
| `how_to`          | Step-by-step runbooks        | `requirements` or `acs` must be non-empty     |
| `explanation`     | Conceptual background        | `stories` or `requirements` must be non-empty |
| `design_doc`      | Architecture / decisions     | `requirements` must be non-empty              |
| `reference`       | Commands / APIs / schemas    | Should reference ≥1 REQ or AC                 |
| `status`          | Snapshots / roadmaps         | `requirements` and `acs` must be non-empty    |
| `adr`             | Architecture decision record | `requirements` must be non-empty              |
| `guide`           | User-facing documentation    | `requirements` or `acs` should be non-empty   |
| `impl_plan`       | Implementation plan          | `requirements` and `acs` must be non-empty    |
| `requirements_doc`| Requirements specification   | `requirements` must be non-empty              |
| `ci_workflow`     | CI workflow YAML             | No frontmatter validation (YAML file)         |

**Note:** Use `how_to` (underscore), not `how-to` (hyphen), for consistency.

When a doc type contract is violated, `docs-check` emits a warning. To fix:
- Add the missing `requirements` / `acs` / `stories` in frontmatter + `doc_index.yaml`, or
- Adjust `doc_type` if the doc was misclassified

---

## 7. Markdown Lint Policy

This section documents the markdown linting rules and enforcement levels applied to the repository.

### 7.1. Enforced Rules

The repository uses markdownlint with the following configuration:

| Rule | Setting | Rationale |
|------|---------|-----------|
| **Line length (MD013)** | 120 chars for prose/headings, unlimited for code blocks | Balance readability with git diffs; allow long lines in technical content |
| **Inline HTML (MD033)** | Allowed | Some docs need HTML for formatting (tables, badges, etc.) |
| **Default rules** | Enabled | Standard markdownlint rules apply unless explicitly overridden |

Configuration is in `.markdownlint.json`:

```json
{
  "extends": "default",
  "ignore-path": ".markdownlintignore",
  "no-inline-html": false,
  "line-length": {
    "line_length": 120,
    "heading_line_length": 120,
    "headers": true,
    "code_blocks": false,
    "code_inline": false
  }
}
```

### 7.2. Ignored Files

The `.markdownlintignore` file excludes:

- `bundle/**` – Generated context bundles (ephemeral, not versioned)
- `node_modules/` – Third-party dependencies

These directories contain either generated content or external code that should not be linted.

### 7.3. Enforcement Levels

| Context | Enforcement | Behavior |
|---------|-------------|----------|
| **Pre-commit hook** | Soft (warnings only) | `cargo xtask precommit` runs markdownlint but only warns on violations; does not block commits |
| **CI** | Not enforced separately | CI runs `cargo xtask docs-check` for versioning/frontmatter sync, but markdownlint is not a hard gate |
| **Manual checks** | Optional | Run `markdownlint '**/*.md'` locally if desired; not required for PR approval |

The soft enforcement in pre-commit means:
- You'll see lint warnings before committing
- Warnings don't block your workflow
- CI focuses on structural doc invariants (versioning, frontmatter) rather than markdown style

### 7.4. When to Act on Lint Warnings

**Fix lint warnings when:**
- The fix is trivial (e.g., adding blank lines, fixing list indentation)
- The warning indicates a real readability issue (e.g., inconsistent heading levels)
- You're already editing the file for other reasons

**Ignore lint warnings when:**
- Fixing would break flow or disrupt a larger change
- The warning is stylistic and the current format is clearer (e.g., long URLs, technical tables)
- You're in rapid iteration mode and don't want context switches

**Add to `.markdownlintignore` when:**
- A file is generated by tooling (should be in `bundle/` or similar)
- A file consistently produces false positives despite being well-formatted
- A doc needs to violate style rules for technical accuracy (rare; document why in a comment)

### 7.5. Running markdownlint Manually

If you want to check markdown style locally:

```bash
# Check all markdown files (respects .markdownlintignore)
markdownlint '**/*.md'

# Check specific files
markdownlint docs/ROADMAP.md README.md

# Auto-fix simple issues
markdownlint --fix '**/*.md'
```

**Note:** Auto-fix is safe for whitespace/formatting but review changes before committing.

### 7.6. Future Evolution

If markdown lint becomes a hard gate:
1. CI will be updated to run `markdownlint` explicitly and fail on violations
2. Pre-commit hook will be updated to block commits (not just warn)
3. This section will be updated with the new enforcement level
4. A grace period with warnings will precede the hard enforcement

For now, treat markdownlint as a **quality signal** rather than a compliance requirement.
