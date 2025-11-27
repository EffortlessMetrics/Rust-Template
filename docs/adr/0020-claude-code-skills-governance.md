# ADR-0020: Claude Code Skills Governance

**Status**: Accepted
**Date**: 2025-11-27
**Authors**: Claude Code Team
**Related ACs**: AC-PLT-SKILLS-001, AC-PLT-SKILLS-002, AC-PLT-SKILLS-003, AC-PLT-SKILLS-004
**Relates to**: ADR-0003 (spec-as-source-of-truth), ADR-0004 (policy-as-code), ADR-0005 (selftest-single-gate)

---

## Context

Claude Code Skills are modular bundles that extend Claude's capabilities within a project. They encapsulate workflows, tools, and domain knowledge.

Current situation:

- 5 core Skills exist (governed-feature-dev, governed-maintenance, etc.)
- They wrap major devex flows from `specs/devex_flows.yaml`
- New Skills are created ad-hoc without formal governance
- No spec, no validation, no traceability

Risk:

1. **Skills explosion**: Without governance, teams create "one Skill per command" (20+ Skills, unusable)
2. **Discovery failure**: Vague descriptions mean Claude can't select right Skill
3. **Security drift**: No validation of `allowed-tools` or secrets in Skills
4. **Orphaned Skills**: No way to retire unused Skills
5. **No auditability**: Can't trace which Skill was used, why it failed

Need:

- Skills treated as governed artifacts (not ad-hoc prose)
- Spec ledger tracks REQs/ACs for Skills
- Automated validation: name format, description quality, allowed-tools safety
- Skills map to devex flows (principle: Skills = workflows, not commands)
- Clear lifecycle: create → govern → retire

---

## Decision

We adopt **structured Skills governance** aligned with Rust-as-Spec platform principles:

### 1. Skills as Governed Artifacts

Each Skill has:

- **REQ**: Describes the workflow intent
- **ACs**: SKILL.md structure, description quality, tool safety
- **Task**: Tracks creation/maintenance work
- **Tests**: BDD scenarios verifying Skill behavior (future)
- **Docs**: Links in `.claude/skills/*/SKILL.md` and `docs/AGENT_SKILLS.md`

Example:

```yaml
# specs/spec_ledger.yaml
- id: REQ-PLT-SKILLS-EXAMPLE
  title: "Example workflow Skill"
  description: "Govern the creation of Skill-example following best practices"
  must_have_ac: true
  acceptance_criteria:
    - id: AC-PLT-SKILLS-EXAMPLE-001
      text: "SKILL.md exists with valid name (kebab-case, <=64 chars)"
      must_have_ac: true
    - id: AC-PLT-SKILLS-EXAMPLE-002
      text: "description includes both 'what' and 'when to use'"
      must_have_ac: true
```

### 2. Skill Locations

**Project Skills** (committed to git, shared):

```
.claude/skills/
├── governed-feature-dev/
│   ├── SKILL.md
│   ├── examples.md (optional)
│   └── reference.md (optional)
├── governed-maintenance/
│   └── SKILL.md
└── ...
```

Each Skill directory has a `SKILL.md` with YAML frontmatter.

### 3. SKILL.md Structure (Normative)

Every `SKILL.md` **MUST** have:

```yaml
---
name: skill-name-here
description: >
  Brief, specific description of what this Skill does and when to use it.
  Include concrete triggers (file types, user phrases).
  Max 1024 characters. Include both WHAT and WHEN.
allowed-tools: Read, Grep, Glob  # Optional: restricts tools Claude can use
---

# Skill Name

## When to Use
...
## Prerequisites
...
## Workflow
...
## Exit Criteria
...
## Error Handling
...
## Examples
...
```

**Normative requirements**:

1. **`name`** (required)
   - Lowercase letters, digits, hyphens only
   - 1-64 characters
   - Matches directory name
   - Unique within the project

2. **`description`** (required)
   - Max 1024 characters
   - MUST state both WHAT (capability) and WHEN (triggers/context)
   - MUST include concrete trigger phrases (file types, user keywords)
   - Written in third person ("This Skill..." not "I can...")

   **Good**: `AC-first feature workflow for implementing Requirements and Acceptance Criteria. Use when implementing tasks tagged 'feature', when user says 'implement AC', or when working with specs/spec_ledger.yaml.`

   **Bad**: `Helps with features`

3. **`allowed-tools`** (optional but recommended)
   - Restricts which Claude Code tools can be used while Skill is active
   - Principle: least privilege
   - Read-only Skills should use: `Read, Grep, Glob`
   - File-writing Skills should NOT include `Bash` (unscoped)
   - No secrets (API keys, tokens) in SKILL.md or supporting files

4. **Body**
   - Top-level title (`# <Name>`)
   - Sections: When, Prerequisites, Workflow, Exit Criteria, Error Handling, Examples (minimum)
   - References to `devex_flows.yaml` flows or xtask commands
   - Supporting files linked via relative paths

### 4. Skills-to-Flows Mapping (Principle)

**Skills wrap workflows, not individual commands.**

From `specs/devex_flows.yaml`:

```yaml
devex_flows:
  - id: ac_first
    name: "AC-first Feature Development"
    description: "..."
    commands: [ac-new, bundle, bdd, selftest]
```

Maps to Skill:

```yaml
---
name: governed-feature-dev
description: "AC-first feature development workflow..."
---
```

**Anti-pattern**: Creating `skill-ac-new` for just `xtask ac-new` command.

**Correct**: `governed-feature-dev` encapsulates the entire AC-first flow.

### 5. Validation Rules

Automated checks in CI and `xtask selftest`:

#### a. SKILL.md Syntax

- [ ] YAML frontmatter valid (opening/closing `---`)
- [ ] Required fields present: `name`, `description`
- [ ] `name` format: kebab-case, <=64 chars, unique
- [ ] `description` format: <=1024 chars, includes "what" + "when"

#### b. Allowed-Tools Safety

- [ ] If `allowed-tools` specified:
  - [ ] Contains only valid tool names (Read, Grep, Glob, Edit, Write, Bash, Task, etc.)
  - [ ] Read-only Skills DON'T include Write/Edit/Bash
  - [ ] Unscoped `Bash` justified in SKILL.md comments
- [ ] No hardcoded secrets (API keys, tokens, passwords)

#### c. References

- [ ] All relative file paths exist (e.g., `reference.md`, `scripts/`)
- [ ] Markdown links resolve
- [ ] xtask commands referenced actually exist

#### d. Governance Alignment

- [ ] REQ/AC exists in `spec_ledger.yaml`
- [ ] Task exists in `specs/tasks.yaml` linking to REQ
- [ ] SKILL.md version matches task version (if applicable)

### 6. Skill Lifecycle

#### Create

1. **Create REQ in `spec_ledger.yaml`**

   ```yaml
   - id: REQ-PLT-SKILLS-MYCORP
     title: "MyCorp-specific workflow Skill"
     tags: [platform, skills, devex]
     must_have_ac: true
     acceptance_criteria:
       - id: AC-PLT-SKILLS-MYCORP-001
         text: "SKILL.md exists at .claude/skills/mycorp-workflow/"
         must_have_ac: true
       - id: AC-PLT-SKILLS-MYCORP-002
         text: "description includes 'what' and 'when to use' per governance spec"
         must_have_ac: true
   ```

2. **Create Task in `specs/tasks.yaml`**

   ```yaml
   - id: TASK-PLT-SKILLS-MYCORP-001
     title: "Implement mycorp-workflow Skill"
     requirement: REQ-PLT-SKILLS-MYCORP
     acs: [AC-PLT-SKILLS-MYCORP-001, AC-PLT-SKILLS-MYCORP-002]
     status: Todo
   ```

3. **Use `governed-feature-dev` Skill** to implement

   ```bash
   # Get task
   curl http://localhost:8080/platform/agent/hints | jq '.tasks[] | select(.id == "TASK-PLT-SKILLS-MYCORP-001")'

   # Claim task
   curl -X POST http://localhost:8080/platform/tasks/TASK-PLT-SKILLS-MYCORP-001/status \
     -d '{"status": "InProgress"}'
   ```

4. **Create and validate**

   ```bash
   mkdir -p .claude/skills/mycorp-workflow
   # Write SKILL.md following template

   cargo xtask skills-lint
   cargo xtask check
   cargo xtask selftest
   ```

5. **Mark done** when selftest passes

#### Maintain

- Keep `devex_flows.yaml` in sync with Skill workflows
- Update `allowed-tools` only if workflow genuinely needs expanded capabilities
- Log friction if Skill needs changes due to kernel changes
- Document version history in SKILL.md body

#### Retire

1. Mark corresponding REQ as deprecated
2. Remove or archive SKILL.md
3. Update `docs/AGENT_SKILLS.md`
4. Run `xtask selftest` to ensure no orphaned references

### 7. Tooling: `skills-lint` and `skills-fmt`

**Implemented in xtask**:

#### `cargo xtask skills-lint`

Validates all Skills against governance rules.

```bash
$ cargo xtask skills-lint

Linting .claude/skills/...

✓ governed-feature-dev: Valid
✓ governed-maintenance: Valid
✗ mycorp-workflow:
  - name must be kebab-case (got: myCorpWorkflow)
  - description too vague (must include "when to use")

Lint failed: 1 error
```

**Runs in CI and `selftest` step 4.**

#### `cargo xtask skills-fmt`

Normalizes SKILL.md formatting (idempotent).

```bash
$ cargo xtask skills-fmt
✓ Formatted .claude/skills/governed-feature-dev/SKILL.md
✓ Formatted .claude/skills/governed-maintenance/SKILL.md
```

Use before committing SKILL.md changes.

### 8. Documentation

**In `.claude/skills/` (for Claude)**:
- Each SKILL.md is self-contained with examples and prerequisites

**In `docs/` (for humans)**:
- `docs/AGENT_SKILLS.md`: Best practices, anti-patterns, full reference
- `docs/SKILLS_GOVERNANCE.md`: This governance spec + checklist
- `docs/SKILLS_TEMPLATE.md`: Copy-paste template for new Skills

**In `CLAUDE.md`** (for agents):
- Reference to Skills as governed workflows
- Guidance: "When to create a Skill" → check if it maps to `devex_flows.yaml`

### 9. Compliance & Enforcement

**Automated (CI + Local)**:

1. **Pre-commit**: `skills-lint` prevents invalid SKILL.md
2. **`xtask check`**: Basic syntax check
3. **`xtask skills-fmt`**: Normalize formatting
4. **`xtask selftest` step 4**: Full governance validation
5. **CI**: Fails PR if Skills invalid

**Manual**:

- Code review: Check that REQ/AC exist for new Skills
- Quarterly audit: Review Skills for drift/orphaning

### 10. Examples: When to Create a Skill

**✅ Good (maps to flow)**:

- "Implement authenticated user signup" → REQ exists → AC-first flow → Skill wraps it
- "Debug selftest failures" → Ad-hoc troubleshooting flow → `governed-governance-debug`
- "Cut a release" → Release flow → `governed-release`

**❌ Bad (too granular)**:

- "Run `xtask check`" → Too small, not a workflow
- "Format Rust code" → Clippy/rustfmt, not a Skill
- "One Skill per xtask command" → Creates 20+ Skills, unusable

**Principle**: "If this workflow is discoverable in `devex_flows.yaml`, it's a candidate Skill."

---

## Consequences

### Positive

- **Auditability**: REQ/AC traceability for every Skill
- **Consistency**: All Skills follow same structure, validation rules
- **Discovery**: Specific descriptions + testing ensure Claude finds right Skill
- **Safety**: Automated validation catches vague descriptions, dangerous tool access
- **Governance alignment**: Skills = workflows (from `devex_flows.yaml`), not ad-hoc scripts
- **Scalability**: Clear rules prevent Skill explosion

### Negative

- **Overhead**: Creating Skill requires REQ + AC + Task (not just SKILL.md)
- **Validation strictness**: Teams may resist governance at first
- **Tooling maturity**: `skills-lint` / `skills-fmt` require implementation

### Neutral

- **Learning curve**: Teams need to understand Skill governance model
- **Not automatic**: Skills are intentional, not one-per-command

---

## Compliance

**Automated**:

- `xtask skills-lint`: Validates all Skills locally and in CI
- `xtask selftest` step 4: Skills governance check
- `cargo xtask docs-check`: Skills listed, versions match spec_ledger
- Pre-commit hooks prevent invalid SKILL.md

**Manual**:

- Code review: Verify REQ/AC exist before merging new Skills
- Quarterly review: Audit Skills for drift, orphaning, or vagueness

**Detection**:

- CI logs show Skill validation errors with file/line
- `xtask skills-lint --trace` for debugging

---

## Notes

**Why governance overhead for Skills?**

Skills are **not** ad-hoc documentation. They are platform capabilities with:
- Autonomous invocation (Claude decides to use them)
- Tool access (can invoke read/write/network tools)
- Workflow implications (may invoke xtask, APIs, modify state)

Governance ensures they don't become unmaintained, vague, or dangerous.

**Why REQ + AC for a SKILL.md file?**

- REQ tracks *intent* (what workflow should this Skill support?)
- AC tracks *contract* (specific structure, naming, tool safety)
- Task tracks *work* (someone owns creating/maintaining it)
- Traceability: when Skills break, we can pinpoint which change caused it

**Can I create Skills without governance?**

- **Local/personal**: Yes, use `~/.claude/skills/` (personal Skills, no CI gate)
- **Project Skills** (`.claude/skills/`): Must follow governance; checked in CI

**How do I migrate existing ad-hoc Skills?**

1. Create REQ + AC in spec_ledger
2. Create Task in tasks.yaml
3. Update SKILL.md to pass `skills-lint`
4. Run `xtask selftest`
5. PR as normal

**What if a Skill breaks due to a kernel change?**

1. Run `xtask selftest` to identify the issue
2. Update SKILL.md and relevant refs
3. Create ADR/issue documenting the breaking change
4. Update Skill version in body

**When should I retire a Skill?**

- No tasks reference it
- No users invoke it (track via `/platform/logs` or survey)
- Flow is no longer needed or merged with another flow

---

## References

- [Claude Code Agent Skills Documentation](https://docs.anthropic.com/claude/docs/agent-skills)
- [Anthropic Skills Authoring Spec](https://github.com/anthropics/claude-code/blob/main/docs/agent-skills-spec.md)
- `specs/devex_flows.yaml` — Workflow definitions
- `docs/AGENT_SKILLS.md` — Best practices and anti-patterns
- `docs/SKILLS_GOVERNANCE.md` — Governance spec + checklist
- ADR-0003 (spec-as-source-of-truth)
- ADR-0004 (policy-as-code)
- ADR-0005 (selftest-single-gate)
