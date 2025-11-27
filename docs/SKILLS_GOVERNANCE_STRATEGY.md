# Skills Governance Strategy: Kernel vs Template ACs

**Date**: 2025-11-27
**Related**: ADR-0020, AC-TPL-SKILLS-*, SKILLS_VALIDATION.md, SKILLS_TEMPLATE.md

---

## Decision

We adopt a **two-level governance model** for Skills:

1. **Kernel ACs** (must_have_ac: true) — Hard enforcement, blocks commits
2. **Template ACs** (must_have_ac: false) — Soft guidance, provides education

This aligns with Rust-as-Spec principles: the contract is strict and narrow at the core, while guidance and best practices are expansive.

---

## Kernel ACs (Errors = Blocking)

These ACs define the **minimum viable contract** that every Skill MUST satisfy:

### AC-TPL-SKILLS-NAME-FORMAT ✅

**Enforcement**: `skills-lint` ERROR (non-zero exit)

**Rule**: Skill names MUST be kebab-case, 1-64 chars, unique.

**Rationale**: Name format is low-cost validation that prevents obvious errors and ensures consistency.

### AC-TPL-SKILLS-DESCRIPTION-QUALITY ✅

**Enforcement**: `skills-lint` ERROR for empty or >1024 chars; WARNING for vague descriptions

**Rules**:
- Description MUST be non-empty (ERROR)
- Description MUST be ≤1024 characters (ERROR)
- Description SHOULD include "when to use" keywords (WARNING, non-blocking)

**Rationale**: Enforcing minimum quality (non-empty, bounded length) prevents obvious trash. Warning on vagueness educates without blocking.

### AC-TPL-SKILLS-GOVERNANCE-001/002/003 ✅

**Enforcement**: Manual + documentation check

**Rules**:
- Each Skill has corresponding REQ in spec_ledger.yaml
- Each Skill has ACs in spec_ledger.yaml
- SKILLS_TEMPLATE.md exists and is up-to-date

**Rationale**: Governance structure is a contract between code and spec. Must be explicit.

---

## Template ACs (Warnings = Guidance)

These ACs define **best practices** and **quality guidance** that help developers create better Skills, but don't block commits.

### AC-TPL-SKILLS-DESCRIPTION-WHAT-WHEN (New: must_have_ac: false)

**Enforcement**: `skills-lint` WARNING (non-blocking); documented in SKILLS_TEMPLATE.md

**Guidance**: Descriptions SHOULD clearly state both WHAT (capability) and WHEN (context/triggers).

**Examples**:
- ✅ "AC-first feature workflow. Use when implementing Requirements and ACs, or when working with spec_ledger.yaml."
- ⚠️ "Workflow for development tasks" (WHAT is clear, WHEN is missing)
- ❌ "Does stuff" (neither WHAT nor WHEN is clear)

**Rationale**: This is a design principle, not a hard requirement. Devs learn by seeing good examples in the template and receiving feedback.

### AC-TPL-SKILLS-ALLOWED-TOOLS-SAFETY (Revised: must_have_ac: false)

**Enforcement**: `skills-lint` WARNING (Phase 2); template guidance in SKILLS_TEMPLATE.md

**Planned checks** (as warnings, not errors):
- Read-only Skills should not include Write/Edit tools
- Unscoped `Bash` should be justified
- No hardcoded secrets (reserved for ERROR-level, Phase 2)

**Rationale**: Tool safety is important but highly context-dependent. Start with warnings to guide, then escalate to errors if patterns emerge.

### AC-TPL-SKILLS-FLOW-MAPPING (Revised: must_have_ac: false)

**Enforcement**: `skills-lint` WARNING (Phase 2); anti-pattern guidance in SKILLS_TEMPLATE.md

**Planned checks** (as warnings):
- Descriptions SHOULD reference devex_flows or xtask commands
- Warn if Skill name suggests single-command wrapping (e.g., `skill-check`)

**Rationale**: Multi-command workflows vs. single-command aliases is a **design principle** that devs need to internalize through feedback, not hard rules.

---

## Implementation Status

### Phase 1: ✅ Complete (MVP)

- ✅ `skills-lint` returns (errors, warnings)
- ✅ Only errors cause non-zero exit
- ✅ Name format validation (ERROR)
- ✅ Description non-empty and max length (ERROR)
- ✅ Description vagueness detection (WARNING)
- ✅ Allowed-tools acceptance of list and string formats
- ✅ Unit tests for all Phase 1 checks
- ✅ SKILLS_VALIDATION.md updated with current implementation

### Phase 2: 🚧 In Plan

- [ ] Hardcoded secret detection (ERROR)
- [ ] Least-privilege tool validation (WARNING)
- [ ] Flow reference detection (WARNING)
- [ ] Anti-pattern detection for single-command Skills (WARNING)
- [ ] Wire `skills-lint` into `cargo xtask selftest`

### Phase 3: 📋 Future

- [ ] File integrity checks (SKILL.md exists, references valid)
- [ ] Advanced metadata tracking (skill usage, maintenance history)

---

## Mental Model for Reviewers & Maintainers

When reviewing Skills or governance changes, ask:

**For KERNEL ACs** (must_have_ac: true):
- Is this a low-cost check that catches obvious errors?
- Can it be enforced automatically by a tool?
- Does it prevent unsafe or obviously bad outcomes?
- If yes to all: make it a hard ERROR.

**For TEMPLATE ACs** (must_have_ac: false):
- Is this a design principle or best practice?
- Does it depend on context or judgment?
- Would hard enforcement frustrate contributors?
- Should violations be **reported but not blocking**?
- If yes: make it a warning in `skills-lint` and add guidance to SKILLS_TEMPLATE.md.

---

## References

- **ADR-0020**: Claude Code Skills Governance
- **SKILLS_VALIDATION.md**: Current implementation details and roadmap
- **SKILLS_TEMPLATE.md**: Best practices and authoring guidance
- **spec_ledger.yaml**: AC definitions (kernel vs template)
- **crates/xtask/src/commands/skills.rs**: Implementation code + unit tests
