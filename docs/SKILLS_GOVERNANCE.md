# Skills Governance: Structured, Auditable, Scalable

**Audience**: Team leads, Skill authors, and developers creating agent capabilities
**Status**: Governed by REQ-TPL-SKILLS-GOVERNANCE, ADR-0020
**Version**: 1.0.0 (aligned with template v3.3.3)

---

## Quick Summary

Claude Code Agent Skills in this repo are **governed artifacts**, not ad-hoc documentation.

| Aspect | Rule | Example |
|--------|------|---------|
| **Creation** | Requires REQ + AC + Task in spec_ledger/tasks | REQ-PLT-SKILLS-EXAMPLE + AC-PLT-SKILLS-EXAMPLE-001 |
| **Structure** | SKILL.md with YAML frontmatter + Markdown body | See [SKILLS_TEMPLATE.md](SKILLS_TEMPLATE.md) |
| **Naming** | kebab-case, ≤64 chars, unique | ✅ `governed-feature-dev`, ❌ `MySkill` |
| **Description** | Must state WHAT + WHEN, ≤1024 chars | ✅ "AC-first workflow. Use when implementing ACs." |
| **Allowed-Tools** | Least privilege (optional but recommended) | ✅ `Read, Grep, Glob` (read-only), ❌ bare `Bash` |
| **Mapping** | Must reference devex_flows.yaml flow or xtask | Skills wrap workflows, not individual commands |
| **Validation** | `cargo xtask skills-lint` enforces rules | Runs in CI and pre-commit |
| **Lifecycle** | Create → Govern → Maintain → Retire | Clear path for deprecation |

---

## 1. Why Governance?

### Without Governance
- ❌ Teams create "one Skill per command" → 20+ unusable Skills
- ❌ Claude can't discover right Skill (vague descriptions)
- ❌ No audit trail (which Skill was used, why did it fail?)
- ❌ Orphaned Skills from deleted workflows
- ❌ Unvetted security (no checks on `allowed-tools` or secrets)

### With Governance
- ✅ Skills = workflows (from devex_flows.yaml) → manageable set
- ✅ Clear discovery signals (specific descriptions + triggers)
- ✅ Traceability (REQ/AC/Task link to changes)
- ✅ Lifecycle management (create, maintain, retire with clear process)
- ✅ Security validation (lint rules catch dangerous patterns)

---

## 2. Skill Governance Model

### 2.1 Every Skill Has REQ/AC/Task

```
User Story (US-...)
  └─ Requirement (REQ-TPL-SKILLS-EXAMPLE)
      └─ Acceptance Criteria (AC-TPL-SKILLS-EXAMPLE-001, ...-002, ...-003)
          └─ Task (TASK-TPL-SKILLS-EXAMPLE-001)
              └─ Implementation (.claude/skills/example/)
```

**Example** from spec_ledger:

```yaml
- id: REQ-TPL-SKILLS-GOVERNANCE
  title: "Agent Skills are governed artifacts"
  description: >
    Each Skill MUST map to a flow and have REQ/AC in spec_ledger.yaml,
    preventing Skill explosion and ensuring discovery.
  acceptance_criteria:
    - id: AC-TPL-SKILLS-GOVERNANCE-001
      text: "docs/SKILLS_GOVERNANCE.md exists with governance spec"
    - id: AC-TPL-SKILLS-NAME-FORMAT
      text: "Skill names kebab-case, <=64 chars, unique"
    - id: AC-TPL-SKILLS-DESCRIPTION-QUALITY
      text: "Description includes WHAT + WHEN, <=1024 chars"
```

**Why?**
- Specs link Skill creation to user need (traceability)
- ACs make success/failure explicit (not vague)
- Tasks provide work queue visibility (`/platform/agent/hints`)
- Automation (xtask, CI) can validate against ACs

### 2.2 Skills Map to Flows

**Principle**: Skills encapsulate **workflows**, not individual commands.

```yaml
# specs/devex_flows.yaml
devex_flows:
  - id: ac_first
    name: "AC-first Feature Development"
    commands: [ac-new, bundle, bdd, check, selftest]
```

Maps to Skill:

```yaml
# .claude/skills/governed-feature-dev/SKILL.md
---
name: governed-feature-dev
description: >
  AC-first feature development workflow. Use when implementing
  features, adding ACs, or working on tasks with status=Todo.
```

**Anti-patterns**:
- ❌ `skill-ac-new` → wraps single `xtask ac-new`
- ❌ `skill-bdd` → wraps single `xtask bdd`
- ❌ `skill-check` → wraps single `xtask check`

**Correct**:
- ✅ `governed-feature-dev` → encapsulates entire AC-first workflow
- ✅ `governed-release` → encapsulates release sequence
- ✅ `governed-governance-debug` → encapsulates troubleshooting

---

## 3. SKILL.md Contract

### 3.1 Frontmatter (Required)

```yaml
---
name: your-skill-name
description: >
  Brief, specific description. MUST include:
  1. WHAT this Skill does (capability)
  2. WHEN to use it (triggers, context, file types)
  Max 1024 characters. Use third-person voice.
allowed-tools: Read, Grep, Glob  # Optional: restrict tools
---
```

### 3.2 Body (Minimum Sections)

```markdown
# Skill Title

## When to Use
- Explicit criteria for invocation
- User phrases (e.g., "implement feature", "fix governance")
- Task types/statuses (e.g., "status=Todo")

## Prerequisites
- Platform running? (`GET /platform/status`)
- Specs valid?
- Required tools installed?

## Workflow
1. Discover work
2. Claim task (if task-based)
3. Execute workflow steps
4. Validate success

## Exit Criteria
- Selftest passes
- Task marked Done
- No new policy violations

## Error Handling
- Common failure modes
- Recovery steps
- When to escalate

## Examples
- Real-world scenarios showing Skill in action
```

### 3.3 Optional: Supporting Files

```
.claude/skills/governed-feature-dev/
├── SKILL.md                    # Required: frontmatter + body
├── examples.md                 # Optional: detailed examples
├── reference.md                # Optional: API reference
└── scripts/
    └── helper.sh               # Optional: utility scripts
```

---

## 4. Validation Rules (Enforced by `skills-lint`)

### 4.1 Name Format

- ✅ kebab-case: `governed-feature-dev`
- ✅ 1-64 characters
- ✅ Alphanumeric + hyphens only
- ✅ Unique within project
- ❌ camelCase, snake_case, UPPERCASE
- ❌ >64 characters
- ❌ Special characters (@, #, %, etc.)
- ❌ Duplicate names

**Test**: `cargo xtask skills-lint`

```bash
$ cargo xtask skills-lint

✗ mycorp-workflow: name 'myCorpWorkflow' must be kebab-case
✗ skill-check: name suggests one-command Skill (anti-pattern)

Lint failed: 2 errors
```

### 4.2 Description Quality

- ✅ Includes WHAT (capability) + WHEN (triggers)
- ✅ ≤1024 characters
- ✅ Third-person voice: "This Skill..." or "Use when..."
- ✅ Specific triggers: file types, user phrases, task states
- ❌ Vague: "Helps with features"
- ❌ First-person: "I can implement ACs"
- ❌ Missing "when to use"

**Examples**:

Good:
```yaml
description: >
  AC-first feature development workflow for implementing Requirements
  and Acceptance Criteria. Use when implementing tasks tagged 'feature',
  when user says 'implement AC', or when working with specs/spec_ledger.yaml.
```

Bad:
```yaml
description: Helps with development
description: Does feature stuff
description: For implementing things
```

**Test**: `cargo xtask skills-lint` flags if description doesn't mention "when"

### 4.3 Allowed-Tools Safety

- ✅ Least privilege: only tools Skill needs
- ✅ Read-only Skills: `Read, Grep, Glob`
- ✅ File-writing Skills: `Read, Edit, Write` (not unscoped `Bash`)
- ✅ If `Bash` used: scoped and documented (e.g., `Bash(git status:*)`)
- ❌ Unscoped `Bash` in read-only Skill
- ❌ All tools together (kitchen sink)
- ❌ Hardcoded secrets (API keys, tokens, passwords)

**Examples**:

Read-only (governance debug):
```yaml
allowed-tools: Read, Grep, Glob
```

Feature development (needs file writing):
```yaml
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
```

Git operations (scoped):
```yaml
allowed-tools: Read, Bash(git status:*), Bash(git diff:*), Bash(git log:*)
```

**Test**: `cargo xtask skills-lint` warns on unsafe patterns

### 4.4 Flow/Command References

- ✅ Description mentions at least one devex_flows flow
- ✅ OR description mentions specific xtask commands used
- ❌ No reference to workflow scope
- ❌ Name suggests single-command wrapping

**Examples**:

Good:
```yaml
description: >
  AC-first feature development workflow. Follows ac_first flow from
  specs/devex_flows.yaml using xtask ac-new, bundle, bdd, selftest.
```

Bad:
```yaml
description: Runs checks and tests
# ^ Doesn't mention which flow/workflow
```

**Test**: `cargo xtask skills-lint` checks description mentions flow/command

---

## 5. Full Lifecycle

### 5.1 Create a New Skill

#### Step 1: Create REQ in spec_ledger.yaml

```yaml
- id: REQ-PLT-SKILLS-MYCORP
  title: "MyCorp-specific workflow Skill"
  tags: [platform, skills, devex]
  must_have_ac: true
  description: >
    The mycorp-workflow Skill encapsulates the complete MyCorp onboarding
    process, guiding new team members through environment setup, secret
    injection, and deployment access.
  acceptance_criteria:
    - id: AC-PLT-SKILLS-MYCORP-001
      text: "SKILL.md exists at .claude/skills/mycorp-workflow/SKILL.md"
      must_have_ac: true
    - id: AC-PLT-SKILLS-MYCORP-002
      text: >
        description includes both WHAT (workflow capability) and WHEN
        (triggers), with reference to relevant flows or xtask commands
      must_have_ac: true
    - id: AC-PLT-SKILLS-MYCORP-003
      text: "allowed-tools follows least-privilege (no unscoped Bash for read-only Skill)"
      must_have_ac: true
    - id: AC-PLT-SKILLS-MYCORP-004
      text: "SKILL.md passes cargo xtask skills-lint with no errors or warnings"
      must_have_ac: true
```

#### Step 2: Create Task in tasks.yaml

```yaml
- id: TASK-PLT-SKILLS-MYCORP-001
  title: "Implement mycorp-workflow Skill"
  description: "Create Skill.md following governance spec and ADR-0020"
  requirement: REQ-PLT-SKILLS-MYCORP
  acs:
    - AC-PLT-SKILLS-MYCORP-001
    - AC-PLT-SKILLS-MYCORP-002
    - AC-PLT-SKILLS-MYCORP-003
    - AC-PLT-SKILLS-MYCORP-004
  status: Todo
  owner: ""  # Empty - claimed during implementation
  labels: [skill, platform, devex]
```

#### Step 3: Use governed-feature-dev Skill to Implement

```bash
# Get task from platform hints
curl http://localhost:8080/platform/agent/hints | \
  jq '.tasks[] | select(.requirement == "REQ-PLT-SKILLS-MYCORP")'

# Claim task
curl -X POST http://localhost:8080/platform/tasks/TASK-PLT-SKILLS-MYCORP-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# Create Skill directory and SKILL.md
mkdir -p .claude/skills/mycorp-workflow
# Use SKILLS_TEMPLATE.md as starting point
cp docs/SKILLS_TEMPLATE.md .claude/skills/mycorp-workflow/SKILL.md
# Edit...

# Format
cargo xtask skills-fmt

# Validate
cargo xtask skills-lint

# Full governance check
cargo xtask selftest
```

#### Step 4: Mark Done When Tests Pass

```bash
# All checks pass
cargo xtask selftest  # Must show ✓ all steps pass

# Mark task done
curl -X POST http://localhost:8080/platform/tasks/TASK-PLT-SKILLS-MYCORP-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "Done"}'

# Create PR / commit
git add .claude/skills/mycorp-workflow/ specs/spec_ledger.yaml specs/tasks.yaml
git commit -m "feat: implement mycorp-workflow Skill (REQ-PLT-SKILLS-MYCORP)"
```

### 5.2 Maintain an Existing Skill

**When**: Workflow changes, kernel changes, documentation updates

**Process**:

1. **Identify issue**
   - Policy/linting failure: `cargo xtask skills-lint`
   - Broken workflow: User report or test failure
   - Drift: Flow in devex_flows.yaml changed

2. **Update SKILL.md**
   - Edit workflow steps to match current flow
   - Update examples
   - Normalize with `cargo xtask skills-fmt`

3. **Validate**
   ```bash
   cargo xtask skills-lint
   cargo xtask skills-fmt
   cargo xtask selftest
   ```

4. **If major change**: Update version in SKILL.md body
   ```markdown
   ## Version History
   - v2.0.0 (2025-11-27): Updated for release-v3.3.3 workflow changes
   - v1.0.0 (2025-09-01): Initial release
   ```

5. **Commit**
   ```bash
   git commit -m "fix: update governed-feature-dev Skill to match devex_flows changes"
   ```

### 5.3 Retire a Skill

**When**: Flow no longer exists, merged into another Skill, obsolete workflow

**Process**:

1. **Mark REQ as deprecated** in spec_ledger.yaml
   ```yaml
   - id: REQ-TPL-SKILLS-DEPRECATED
     title: "[DEPRECATED] Old workflow Skill"
     status: deprecated
     description: "This Skill is deprecated as of v3.4.0. Use X instead."
   ```

2. **Archive SKILL.md** (optional, or remove)
   ```bash
   rm .claude/skills/old-skill/SKILL.md
   # OR move to archive/
   ```

3. **Update docs**
   - Remove from `docs/AGENT_SKILLS.md`
   - Update `docs/SKILLS_GOVERNANCE.md` if needed

4. **Run selftest**
   ```bash
   cargo xtask selftest  # Should still pass (no dangling refs)
   ```

5. **Commit**
   ```bash
   git commit -m "deprecate: retire old-skill Skill (use new-skill instead)"
   ```

---

## 6. Tooling: skills-lint, skills-fmt

### 6.1 `cargo xtask skills-lint`

**What it does**: Validates all Skills against governance rules

```bash
$ cargo xtask skills-lint

Linting .claude/skills/...

✓ governed-feature-dev: Valid
  - name: governed-feature-dev (kebab-case, 20 chars)
  - description: 187 chars, includes WHAT + WHEN
  - allowed-tools: Read, Grep, Glob, Edit, Write, Bash (appropriate for feature dev)

✓ governed-maintenance: Valid

✗ mycorp-workflow:
  ERROR: name 'myCorpWorkflow' must be kebab-case
  ERROR: description missing "when to use" triggers
  WARNING: allowed-tools includes unscoped Bash (justify in SKILL.md comments)

Lint failed: 1 error, 1 warning
```

**Runs in**:
- ✅ Pre-commit hooks (if configured)
- ✅ `cargo xtask selftest` (step 4: skills validation)
- ✅ CI (blocks PR if Skills invalid)

**How to run**:
```bash
# Lint all
cargo xtask skills-lint

# Lint specific Skill
cargo xtask skills-lint .claude/skills/my-skill/SKILL.md

# Verbose output
cargo xtask skills-lint -v
```

### 6.2 `cargo xtask skills-fmt`

**What it does**: Normalizes SKILL.md formatting (idempotent)

```bash
$ cargo xtask skills-fmt

Formatting .claude/skills/...

✓ Formatted .claude/skills/governed-feature-dev/SKILL.md
  - Reordered frontmatter fields (name, description, allowed-tools)
  - Normalized section spacing
  - Fixed relative link paths

✓ Formatted .claude/skills/governed-maintenance/SKILL.md

Formatted 2 files
```

**Run before committing**:
```bash
cargo xtask skills-fmt
git add .claude/skills/
git commit -m "chore: format Skills"
```

---

## 7. Common Questions

### Q: When should I create a new Skill?

**Check**:
1. Is this a **workflow** (sequence of commands/steps)?
2. Does it map to a flow in `specs/devex_flows.yaml`?
3. Will multiple team members invoke this?

**If YES to all**: Create a Skill (with REQ/AC/Task).
**If NO**: Probably not a Skill → inline in docs or xtask command instead.

**Examples**:

✅ **Create a Skill**:
- "Implement AC-first feature development" → maps to `ac_first` flow
- "Debug governance failures" → maps to `governance_debug` flow
- "Cut a release" → maps to `release` flow

❌ **Don't create a Skill**:
- "Run `xtask check`" → too granular, not a workflow
- "Format Rust code" → Clippy/rustfmt, not a Skill
- "One Skill per command" → leads to explosion

### Q: My Skill isn't being invoked by Claude. Why?

**Check description specificity**:
- ❌ "Helps with development"
- ✅ "AC-first feature workflow. Use when implementing ACs or tasks tagged 'feature'."

Include concrete trigger words:
- File types: `.toml`, `.yaml`, `.md`
- User phrases: "implement", "add AC", "fix bug"
- Task states: "Todo", "InProgress"

**Test**:
```bash
# Ask Claude directly
"Can you help me implement AC-TPL-SKILLS-001?"

# Claude should suggest your Skill if description matches
```

### Q: Can a Skill call another Skill?

No. Skills are autonomous capabilities, not chaining mechanisms.

**Instead**:
1. Skill A completes
2. In "Error Handling" or "Next Steps", recommend Skill B
3. User or Claude invokes Skill B next

### Q: What if `skills-lint` is too strict?

**If a rule is genuinely wrong for your Skill**:
1. Document justification in SKILL.md (comment)
2. Raise issue in your repo linking to the rule
3. ADR process to override (if team agrees)

**Example**:
```markdown
## Tool Access Justification
This Skill uses unscoped Bash because workflows in MyCorp require
direct shell access for secret injection. Risk accepted by team (ADR-XXX).
```

---

## 8. Anti-Patterns (What NOT to Do)

### ❌ One Skill Per Command

**Bad**:
```
.claude/skills/
├── skill-check/SKILL.md       # Wraps xtask check
├── skill-bdd/SKILL.md         # Wraps xtask bdd
├── skill-bundle/SKILL.md      # Wraps xtask bundle
└── skill-selftest/SKILL.md    # Wraps xtask selftest
```

**Problem**: 20+ Skills, Claude can't choose → all Skills ignored

**Good**:
```
.claude/skills/
└── governed-feature-dev/SKILL.md  # Wraps entire ac_first flow
```

### ❌ Vague Descriptions

**Bad**:
```yaml
description: Helps with features
description: For development
description: Does stuff with files
```

**Good**:
```yaml
description: >
  AC-first feature development workflow. Use when implementing tasks
  tagged 'feature', when user says 'implement AC', or when working
  with specs/spec_ledger.yaml.
```

### ❌ Ad-Hoc Skills (No Governance)

**Bad**:
Create `.claude/skills/quick-fix/SKILL.md` without:
- REQ in spec_ledger.yaml
- AC defining structure
- Task tracking work

**Good**:
1. Create REQ-TPL-SKILLS-QUICKFIX in spec_ledger
2. Define ACs for Skill structure
3. Create Task (so work is discoverable)
4. Implement SKILL.md

### ❌ Tools Without Justification

**Bad**:
```yaml
allowed-tools: Bash, Read, Write, Edit, WebSearch, Task, Agent
# ^ Kitchen sink, no justification
```

**Good**:
```yaml
allowed-tools: Read, Grep, Glob
# ^ Least privilege for read-only governance Skill
```

**Or with justification**:
```yaml
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
# Justified in SKILL.md: "Feature dev needs file writing + shell for tests"
```

### ❌ No Reference to Flows

**Bad**:
```yaml
description: Workflow for stuff
# ^ Doesn't mention which flow or commands
```

**Good**:
```yaml
description: >
  AC-first feature development workflow. Follows ac_first flow from
  devex_flows.yaml using xtask ac-new, bundle, bdd, selftest.
```

---

## 9. Checklist: Creating a Skill

Use this before committing:

- [ ] **REQ created** in `specs/spec_ledger.yaml`
- [ ] **ACs created** defining SKILL.md structure requirements
- [ ] **Task created** in `specs/tasks.yaml` linking to REQ/ACs
- [ ] **Directory created** at `.claude/skills/<name>/`
- [ ] **SKILL.md written**:
  - [ ] YAML frontmatter: `name`, `description`, `allowed-tools` (optional)
  - [ ] `name`: kebab-case, ≤64 chars, unique
  - [ ] `description`: includes WHAT + WHEN, ≤1024 chars, triggers/file types
  - [ ] Sections: When, Prerequisites, Workflow, Exit Criteria, Error Handling, Examples
  - [ ] References devex_flows or xtask commands
  - [ ] No hardcoded secrets
- [ ] **Supporting files** (optional):
  - [ ] `examples.md` if examples are long
  - [ ] `reference.md` if reference material is long
  - [ ] Relative paths correct
- [ ] **Format**:
  ```bash
  cargo xtask skills-fmt
  ```
- [ ] **Validate**:
  ```bash
  cargo xtask skills-lint          # No errors/warnings
  cargo xtask check                # Fast checks
  cargo xtask selftest             # Full governance
  ```
- [ ] **Commit**:
  ```bash
  git add .claude/skills/<name>/ specs/spec_ledger.yaml specs/tasks.yaml
  git commit -m "feat: implement <name> Skill (REQ-TPL-SKILLS-...)"
  ```

---

## 10. References

- **ADR-0020**: [Claude Code Skills Governance](../adr/0020-claude-code-skills-governance.md)
- **ADR-0003**: [Spec and BDD as Source of Truth](../adr/0003-spec-and-bdd-as-source-of-truth.md)
- **ADR-0005**: [xtask + selftest as Single Gate](../adr/0005-xtask-selftest-single-gate.md)
- **AGENT_SKILLS.md**: [Best practices and anti-patterns](AGENT_SKILLS.md)
- **SKILLS_TEMPLATE.md**: [Copy-paste template for new Skills](SKILLS_TEMPLATE.md)
- **spec_ledger.yaml**: [REQ-TPL-SKILLS-* requirements](../specs/spec_ledger.yaml)
- **devex_flows.yaml**: [Workflow definitions](../specs/devex_flows.yaml)
- **Anthropic's Skills Docs**: https://docs.anthropic.com/claude/docs/agent-skills

---

## Questions or Feedback?

If this governance model is unclear or overly burdensome:
1. Open an issue in your repo linking to this doc
2. Add to `FRICTION_LOG.md` if it's a process/tooling pain point
3. Suggest ADR to override governance (with justification)

Remember: **Governance enables scale**. With 5 well-governed Skills, Claude is effective. With 50 ad-hoc Skills, Claude is paralyzed.
