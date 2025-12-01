<!-- doclint:disable orphan-version -->
<!-- ADR: This document contains historical version references as part of the decision record. -->
# ADR-0021: Claude Code Agents Governance

**Status**: ACCEPTED
**Decision Date**: 2025-11-27
**Supersedes**: None
**Related**: ADR-0003 (Spec as Source of Truth), ADR-0005 (Selftest), ADR-0020 (Skills Governance)

---

## 1. Problem Statement

Claude Code agents (`.claude/agents/*.md`) are increasingly used by the platform for automations, developer workflows, and CI tasks. Without governance:

- **Discovery**: Agents are scattered and undocumented; developers don't know which agents exist or when to use them
- **Safety**: Agents can have overly broad permissions (bypassPermissions, all tools) without justification
- **Maintenance**: Agents reference non-existent Skills or outdated workflows
- **Auditability**: No traceability from agent decision (why it exists) to implementation
- **Quality**: Descriptions are vague ("Helper", "Agent") instead of specific (what + when)

### Example Failure Modes

1. Two agents with identical names in different locations → confusion
2. Agent uses `bypassPermissions` with no documented rationale
3. Agent references Skills that no longer exist
4. Agent description is "I'm a helpful assistant" (unmeasurable, undiscoverable)
5. No REQ/AC for the agent → can't be tested or verified

---

## 2. Decision: Treat Agents as Governed Artifacts

**We will apply the same governance model to agents as we do to Skills (ADR-0020):**

### 2.1 Specification

Each project agent (in `.claude/agents/*.md`) must have:

1. **REQ + AC** in `specs/spec_ledger.yaml`
   - REQ: Why does this agent exist? What problem does it solve?
   - AC: What must be true for the agent to be "correct"? (e.g., "correctly implements feature workflow")

2. **YAML frontmatter** with structured metadata
   ```yaml
   ---
   name: my-agent                    # kebab-case, ≤64 chars, unique
   description: >                    # WHAT + WHEN, ≤1024 chars
     Analyzes code for bugs.
     Use when reviewing PRs.
   tools: Read, Grep, Glob           # Least-privilege: only what's needed
   model: sonnet                      # sonnet|opus|haiku|inherit
   permissionMode: default            # default|acceptEdits|bypassPermissions|plan|ignore
   skills: governed-feature-dev       # Comma-separated or list
   ---
   ```

3. **Markdown body** with role, workflow, tool usage, safety constraints

### 2.2 Validation (agents-lint)

A new xtask command, `cargo xtask agents-lint`, validates:

**Kernel ACs (errors):**
- Name: kebab-case, unique, ≤64 chars, matches filename
- Description: non-empty, ≤1024 chars
- PermissionMode: valid value
- Model: known value (sonnet|opus|haiku|inherit)
- Tools: valid format, non-empty if specified
- Skills: all referenced Skills exist in `.claude/skills/`
- YAML: valid syntax, no tabs

**Template ACs (warnings):**
- Description quality: includes WHEN context
- Broad tool combos: warns if Bash + Edit + Write without justification
- Expensive models: warns if opus (not an error, but flag for review)
- Body structure: should have at least one heading

### 2.3 Integration

1. **xtask command**: `cargo xtask agents-lint`
2. **Selftest**: Step 3/10 (after Skills, before BDD)
3. **Precommit**: Change-aware lint (runs if `.claude/agents/` changed)
4. **CI**: Separate job (`.github/workflows/ci-agents.yml`) with path filters
5. **Spec**: REQ-TPL-AGENTS-GOVERNANCE with 8 acceptance criteria

---

## 3. Rationale

### Why Govern Agents Like Skills?

Both are **governed workflows**:
- **Skills**: Reusable capabilities (e.g., "implement feature with tests and docs")
- **Agents**: Tools that use Skills to solve specific problems (e.g., "code reviewer agent")

Both need:
- **Discovery**: Listed in spec, docs, and governance views
- **Safety**: Validated for least-privilege and clear scope
- **Traceability**: REQ → AC → implementation → tests
- **Maintainability**: Updated when dependencies (Skills) change

### Why Not Treat Agents as Ad-Hoc?

Ad-hoc agents (no governance) lead to:
- **Sprawl**: 50+ agents with unclear purposes
- **Duplication**: Multiple agents solving the same problem
- **Bitrot**: Agents referencing deleted Skills or deprecated APIs
- **Risk**: Agents with `bypassPermissions` used without understanding trade-offs

Governance enables:
- **Catalog**: Agents are discoverable and well-documented
- **Safety**: Permissions are justified and validated
- **Trust**: Team understands when and why each agent exists
- **Auditability**: REQ → AC → implementation linked in spec

### Why the Specific Validation Rules?

1. **Name**: kebab-case matches CLI conventions (xtask commands, Skills names). Uniqueness prevents confusion.
2. **Description**: WHAT + WHEN makes agents discoverable and self-documenting.
3. **Tools**: Explicit list (or none) enforces least-privilege.
4. **PermissionMode**: Constrained enum prevents typos and forces explicit choice.
5. **Skills**: Validation ensures agents don't break when Skills are refactored.
6. **REQ/AC**: Traceability ensures agents are tested and maintained.

---

## 4. Alternatives Considered

### A. No Governance (Status Quo)

**Pros**: Agents can be created freely; fast iteration
**Cons**: No discovery, no safety validation, no traceability, maintenance nightmare

**Rejected**: Incompatible with platform reliability goals (ADR-0005)

### B. Lightweight Governance (Lint Only, No REQ/AC)

**Pros**: Faster creation; less spec overhead
**Cons**: No traceability; agents can be orphaned or poorly tested

**Rejected**: Traceability (REQ/AC linking) is essential for maintenance. Skills governance (ADR-0020) already established the pattern.

### C. Full Governance (This ADR)

**Pros**: Complete traceability, safety validation, discovery, testability
**Cons**: More work to create agents; spec overhead

**Chosen**: Aligns with platform philosophy (ADR-0003, ADR-0005); enables autonomous workflows

---

## 5. Implementation Plan

### Phase 1: Core Validation (MVP)
- ✅ agents-lint command with kernel AC checks (name, description, permissions, skills)
- ✅ Selftest integration (Step 3/10)
- ✅ Precommit integration (change-aware)
- ✅ Spec: REQ-TPL-AGENTS-GOVERNANCE + 8 ACs
- ✅ Docs: AGENTS_GOVERNANCE.md, AGENTS_TEMPLATE.md, AGENTS_VALIDATION.md

### Phase 2: Enhanced Validation (Planned)
- Secret detection (error on hardcoded API keys)
- Least-privilege warnings (read-only agent + write tools)
- Description quality heuristics (generic vs. specific)

### Phase 3: Integration (Planned)
- CI workflow (`.github/workflows/ci-agents.yml`)
- BDD scenarios for agents (in `specs/features/xtask_devex.feature`)
- Agent registry/dashboard (if needed)

---

## 6. Consequences

### Positive

1. **Discoverability**: Agents are catalogued in spec; `agents-lint` is the source of truth
2. **Safety**: Permissions validated; overly broad tools flagged for review
3. **Maintainability**: REQ/AC ensure agents are updated with Skills/flows
4. **Traceability**: Every agent links to decision (REQ) and tests (AC)
5. **Autonomy**: Clear contract enables safe agent invocation in CI/automation

### Negative

1. **Overhead**: Creating an agent requires adding REQ/AC to spec (5 mins extra)
2. **Strictness**: Agents can't be created ad-hoc in `.claude/agents/`; must go through spec first
3. **Learning Curve**: New team members must understand governance model

### Mitigation

- **Overhead**: Template and checklist make it trivial
- **Strictness**: Local experiments can still use `~/.claude/agents/` (not governed)
- **Learning Curve**: Clear docs and examples in AGENTS_GOVERNANCE.md

---

## 7. Precedent & Alignment

This ADR follows the same pattern established by **ADR-0020 (Skills Governance)**:

| Aspect | Skills (ADR-0020) | Agents (ADR-0021) |
|--------|-------------------|-------------------|
| Artifact | `.claude/skills/*/SKILL.md` | `.claude/agents/*.md` |
| Governance | REQ + AC in spec | REQ + AC in spec |
| Lint tool | `skills-lint` | `agents-lint` |
| Validation | Kernel (errors) + template (warnings) | Kernel + template |
| Integration | Selftest + precommit + CI | Selftest + precommit + CI |
| Traceability | Mapped to devex flows | Mapped to workflows/Skills |

---

## 8. Testing Strategy

1. **Unit Tests**: agents-lint validation rules (same as skills-lint)
2. **BDD Scenarios**: Acceptance tests for ACs (in `specs/features/xtask_devex.feature`)
3. **Integration Tests**: Selftest step 3/10 validates full flow
4. **CI Tests**: GitHub Actions workflow validates on PR

---

## 9. Related Decisions

- **ADR-0003**: Spec as source of truth (agents in spec_ledger.yaml)
- **ADR-0005**: Selftest as single gate (agents-lint is selftest step 3)
- **ADR-0020**: Skills governance (agents governance follows same pattern)
- **ADR-0017**: DevEx contract (agents mentioned as platform artifacts)

---

## 10. Questions & Answers

**Q: What about user-local agents (~/.claude/agents/)?**
A: User agents are out-of-scope for repo governance. They're personal tools and not validated by CI/selftest.

**Q: Do I need REQ/AC for every agent?**
A: Only for project agents in `.claude/agents/`. Local agents can be ad-hoc.

**Q: What if I disagree with the permission restrictions?**
A: File an issue or ADR arguing for change. Current policy (least-privilege, justified high-risk modes) is intentional per security best practices (principle of least privilege).

**Q: Can Skills reference Agents?**
A: No. Skills are standalone capabilities. Agents use Skills, not vice versa.

**Q: How do I deprecate an agent?**
A: Mark REQ as `deprecated: true` in spec. Optionally move agent file to `.claude/agents_archived/` for preservation.

---

## 11. Approval & Sign-Off

- **Decision Owner**: Platform Governance Team
- **Approved**: [Date]
- **Implementation**: Phase 1 complete in v3.3.3; Phase 2/3 planned for v3.4.x

---

**Version**: 1.0.0
**Last Updated**: 2025-11-27
