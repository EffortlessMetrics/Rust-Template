---
id: GUIDE-TPL-CHANGE-DOCS-001
title: How to Change Documentation Safely
doc_type: how_to
status: published
audience: developers, maintainers
tags: [documentation, governance, workflow]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-DOCS-CONSISTENCY
  - REQ-PLT-AC-MANAGEMENT
acs:
  - AC-PLT-009
  - AC-PLT-010
adrs: [ADR-0005]
last_updated: 2025-12-01
---

# How to Change Documentation Safely

This guide covers the governance-compliant workflows for modifying documentation in the Rust-as-Spec platform cell. Following these patterns ensures your changes pass `cargo xtask docs-check` and `selftest`.

## Quick Edit Loop

While editing docs, run this loop to validate your changes:

```bash
# While editing docs:
cargo xtask docs-check        # validate front-matter, doc_index, doclint
git diff docs/...             # sanity check your changes
```

CI treats doc issues as **hard failures** via `XTASK_STRICT_PRECOMMIT=1` in `tier1-selftest.yml`.
Run `docs-check` before pushing to avoid surprises.

---

## Quick Reference

| Change Type | Key Steps |
|------------|-----------|
| **Bump version** | Edit spec_ledger.yaml, run release-prepare, verify with docs-check |
| **Add new doc** | Add frontmatter, add to doc_index.yaml, verify with docs-check |
| **Move/rename doc** | Update doc_index.yaml, update frontmatter, verify with docs-check |
| **Change AC status** | Edit spec_ledger.yaml, update feature_status_notes.md, regenerate with ac-status |

---

## Changing Versioned Documents

Documents that contain version numbers (README, CLAUDE.md, ROADMAP, KERNEL_SNAPSHOT, etc.) are governed by version alignment checks.

### Steps

1. **Update the canonical version first:**
   ```bash
   # Edit specs/spec_ledger.yaml -> metadata.template_version
   ```

2. **Run release-prepare to bump all files:**
   ```bash
   nix develop
   cargo xtask release-prepare X.Y.Z
   ```
   This automatically updates version strings in:
   - README.md
   - CLAUDE.md
   - docs/ROADMAP.md
   - docs/KERNEL_SNAPSHOT.md
   - docs/explanation/TEMPLATE-CONTRACTS.md
   - specs/service_metadata.yaml
   - specs/doc_index.yaml
   - CHANGELOG.md

3. **Verify alignment:**
   ```bash
   cargo xtask docs-check
   ```
   All versioned files should show `✓` next to their version.

4. **Commit together:**
   ```bash
   git add -A
   git commit -m "chore: bump version to X.Y.Z"
   ```

### What Can Go Wrong

- **Version mismatch:** If you edit a versioned doc manually, it may drift. Always use `release-prepare` or edit all files together.
- **Partial commit:** Committing some versioned files but not others causes docs-check failure.

---

## Adding or Moving Documents

Documents in `specs/doc_index.yaml` must have synchronized frontmatter. The doc_index is the canonical registry.

### Adding a New Document

1. **Create the markdown file with frontmatter:**
   ```markdown
   ---
   id: GUIDE-TPL-MY-DOC-001
   title: My New Document
   doc_type: how_to
   status: draft
   audience: developers
   tags: [relevant, tags]
   stories: [US-TPL-PLT-001]
   requirements: [REQ-PLT-XXX]
   acs: [AC-PLT-XXX]
   adrs: []
   ---

   # My New Document

   Content here...
   ```

2. **Add entry to doc_index.yaml:**
   ```yaml
   - id: GUIDE-TPL-MY-DOC-001
     file: docs/how-to/my-doc.md
     title: "My New Document"
     doc_type: how_to
     stories: [US-TPL-PLT-001]
     requirements: [REQ-PLT-XXX]
     acs: [AC-PLT-XXX]
     adrs: []
   ```

3. **Verify synchronization:**
   ```bash
   cargo xtask docs-check
   ```
   Should show: `Doc index & front-matter... ✓ Consistent`

4. **Check doc_type contract:**
   The doc_type you choose must satisfy its contract (see [doc-sources.md Section 6.5](../reference/doc-sources.md)):
   - `how_to`: needs `requirements` or `acs`
   - `explanation`: needs `stories` or `requirements`
   - `design_doc`: needs `requirements`
   - etc.

### Moving or Renaming a Document

1. **Move the file:**
   ```bash
   git mv docs/old/path.md docs/new/path.md
   ```

2. **Update doc_index.yaml:**
   Change the `file:` field to the new path.

3. **Verify:**
   ```bash
   cargo xtask docs-check
   ```

### What Can Go Wrong

- **Frontmatter/index mismatch:** If `id`, `doc_type`, or reference arrays differ between frontmatter and doc_index.yaml, docs-check fails.
- **Doc_type contract violation:** Using a doc_type that doesn't match the doc's content. Review Section 6.5 in doc-sources.md.
- **Orphaned index entry:** If doc_index.yaml references a file that doesn't exist.

---

## Changing AC Classifications

AC (Acceptance Criteria) status is tracked in `specs/spec_ledger.yaml` and visualized in `docs/feature_status.md`.

### Changing AC Status or Text

1. **Edit spec_ledger.yaml:**
   Find the AC under its requirement and modify:
   ```yaml
   acceptance_criteria:
     - id: AC-PLT-XXX
       text: "Updated AC description"
       must_have_ac: true  # or false
   ```

2. **Update reasoning in feature_status_notes.md:**
   If changing `must_have_ac` or AC classification, document why:
   ```markdown
   ### AC-PLT-XXX: Title

   **Status:** Soft (intentionally)
   **Reason:** This AC is aspirational for v4.0...
   ```

3. **Regenerate feature_status.md:**
   ```bash
   cargo xtask ac-status
   ```

4. **Verify governance:**
   ```bash
   cargo xtask docs-check
   cargo xtask selftest
   ```

### What Can Go Wrong

- **Stale feature_status.md:** If you edit spec_ledger.yaml without running ac-status, the generated file becomes out of sync.
- **Missing test coverage:** Changing an AC to `must_have_ac: true` requires test coverage or it fails governance.
- **Undocumented soft AC:** If you demote an AC to soft without explaining why in feature_status_notes.md, future maintainers won't understand the reasoning.

---

## Changing Skills or Agents

Skills (`.claude/skills/*/SKILL.md`) and Agents (`.claude/agents/*.md`) are governed artifacts.

### Steps

1. **Edit the SKILL.md or agent file**

2. **Format (for Skills):**
   ```bash
   cargo xtask skills-fmt
   ```

3. **Lint to verify:**
   ```bash
   cargo xtask skills-lint  # for Skills
   cargo xtask agents-lint  # for Agents
   ```

4. **Run docs-check:**
   ```bash
   cargo xtask docs-check
   ```
   Skills validation is included in docs-check.

### What Can Go Wrong

- **Invalid frontmatter:** Skills and Agents have required fields (name, description, tools, etc.)
- **Hardcoded secrets:** Linters detect patterns like `API_KEY=`, `password:`, etc.
- **Missing skill reference:** If an Agent references a Skill that doesn't exist.

---

## Pre-Commit Workflow

The repository has a git pre-commit hook that automatically:

1. Runs `cargo fmt` and auto-stages formatted files
2. Runs Skills format and auto-stages changes
3. Runs clippy + tests
4. Runs Skills/Agents governance checks
5. Regenerates and stages `docs/feature_status.md`
6. Runs docs-check and spellcheck (soft by default)

**To commit safely:**
```bash
git add <your-changes>
git commit -m "Your message"
# Pre-commit hook runs automatically
```

**If pre-commit fails:**
```bash
# See what failed
cargo xtask precommit

# Fix issues, then try again
git commit -m "Your message"
```

---

## Validation Ladder

Use these checks in order of increasing strictness:

```bash
# Quick syntax check
cargo xtask docs-check

# Full governance validation
cargo xtask selftest

# CI-equivalent (strict)
XTASK_STRICT_PRECOMMIT=1 cargo xtask precommit
```

---

## Summary Checklist

Before committing documentation changes:

- [ ] Version strings aligned (use `release-prepare` for version bumps)
- [ ] Frontmatter matches doc_index.yaml (id, doc_type, references)
- [ ] Doc_type contract satisfied (see Section 6.5 in doc-sources.md)
- [ ] `cargo xtask docs-check` passes
- [ ] If changing ACs: `cargo xtask ac-status` regenerated feature_status.md
- [ ] If changing Skills/Agents: `skills-lint` or `agents-lint` passes

---

## Related Documentation

- [Documentation Sources and Governance](../reference/doc-sources.md) - Full invariants reference
- [Change Acceptance Criterion](./change-acceptance-criterion.md) - Detailed AC workflow
- [Maintain Kernel](./maintain-kernel.md) - Kernel maintenance workflows
