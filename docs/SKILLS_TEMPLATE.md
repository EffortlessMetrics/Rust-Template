# Skills Template & Checklist
<!-- doclint:disable orphan-version -->

**Purpose**: Copy-paste template for creating new Agent Skills that pass governance
**Status**: Governed by AC-TPL-SKILLS-GOVERNANCE-003

---

## Step 1: Governance Prep (Before Creating SKILL.md)

### Create REQ in specs/spec_ledger.yaml

Find your user story (US-...) or create one. Add:

```yaml
      - id: REQ-TPL-SKILLS-YOURNAME
        title: "Your-skill-name workflow Skill"
        tags: [platform, skills, devex]
        must_have_ac: true
        description: >
          A one-sentence description of what workflow this Skill encapsulates.
          Reference the flow from devex_flows.yaml this Skill wraps.
        adr: [ADR-0020]  # Skills governance
        acceptance_criteria:
          - id: AC-TPL-SKILLS-YOURNAME-001
            text: "SKILL.md exists at .claude/skills/your-skill-name/SKILL.md"
            must_have_ac: true
          - id: AC-TPL-SKILLS-YOURNAME-002
            text: >
              description includes both WHAT (capability) and WHEN (triggers/context),
              references devex_flows or xtask commands, max 1024 chars
            must_have_ac: true
          - id: AC-TPL-SKILLS-YOURNAME-003
            text: >
              allowed-tools follows least-privilege principle
              (no unscoped Bash in read-only Skills, no secrets)
            must_have_ac: true
          - id: AC-TPL-SKILLS-YOURNAME-004
            text: "SKILL.md passes cargo xtask skills-lint"
            must_have_ac: true
```

### Create Task in specs/tasks.yaml

```yaml
      - id: TASK-TPL-SKILLS-YOURNAME-001
        title: "Implement your-skill-name Skill"
        description: >
          Create and validate the your-skill-name Skill according to
          SKILLS_GOVERNANCE.md and ADR-0020. Ensure SKILL.md passes
          skills-lint and selftest.
        requirement: REQ-TPL-SKILLS-YOURNAME
        acs:
          - AC-TPL-SKILLS-YOURNAME-001
          - AC-TPL-SKILLS-YOURNAME-002
          - AC-TPL-SKILLS-YOURNAME-003
          - AC-TPL-SKILLS-YOURNAME-004
        status: Todo
        owner: ""
        labels: [skill, platform]
```

---

## Step 2: Copy SKILL.md Template

Create directory:

```bash
mkdir -p .claude/skills/your-skill-name
cd .claude/skills/your-skill-name
```

Copy the template below and fill in sections.

---

## Step 3: SKILL.md Template (Copy & Paste)

```yaml
---
name: your-skill-name
description: >
  [ONE-SENTENCE SUMMARY OF CAPABILITY]

  [WHAT THIS SKILL DOES: specific, technical description]

  [WHEN TO USE: concrete triggers - user phrases, file types, task types, status]
  Max 1024 characters. Use third-person voice.

  Example good description:
  "AC-first feature development workflow for implementing Requirements and Acceptance Criteria.
  Use when implementing tasks tagged 'feature', when user says 'implement AC', or when working
  with specs/spec_ledger.yaml and BDD scenarios."
allowed-tools: Read, Grep, Glob  # Optional: restrict tool access. Least privilege.
---

# [Human-Readable Skill Title]

## When to Use

This Skill is invoked when:

- [Trigger 1: specific user phrase or context]
- [Trigger 2: task type/status]
- [Trigger 3: file/domain indicator]

**NOT** for:

- [Anti-pattern 1]
- [Anti-pattern 2]

## Prerequisites

Before using this Skill:

- [ ] [Prerequisite 1, e.g., "Platform running (check `GET /platform/status`)"]
- [ ] [Prerequisite 2, e.g., "Specs valid (no syntax errors in spec_ledger.yaml)"]
- [ ] [Prerequisite 3, e.g., "Task exists or can be created"]

## Workflow

This Skill follows the **[FLOW NAME]** flow from `specs/devex_flows.yaml`.

### Step 1: Discover Work

[How to identify work - e.g., call `/platform/agent/hints`, check task board]

```bash
# Example command
curl http://localhost:8080/platform/agent/hints | jq '.tasks[] | select(.status == "Todo")'
```

Output: List of prioritized tasks.

### Step 2: Claim/Start Work

[How to claim/start - e.g., update task status, check out branch]

```bash
# Example command
TASK_ID="TASK-TPL-YOURNAME-001"
curl -X POST "http://localhost:8080/platform/tasks/${TASK_ID}/status" \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'
```

### Step 3: Execute Workflow

[Core workflow steps, in detail, with actual commands/APIs used]

1. **Action 1**

   [Description of what happens, why it's important]

   ```bash
   cargo xtask [command] [options]
   ```

   [Expected output or success indicator]

2. **Action 2**

   [Description]

   [Commands, APIs, file edits]

3. **Action 3**

   [Description]

   [Commands, APIs]

[Reference devex_flows.yaml flow or explain workflow sequence]

### Step 4: Validation

[How to confirm success]

```bash
# Run validation
cargo xtask selftest

# Expected output: All 7 steps pass
```

### Step 5: Mark Complete

[How to finish - e.g., update task status, create PR]

```bash
# Example command
curl -X POST "http://localhost:8080/platform/tasks/${TASK_ID}/status" \
  -H "Content-Type: application/json" \
  -d '{"status": "Done"}'
```

## Exit Criteria

This Skill succeeds when:

- [ ] [Explicit success indicator 1, e.g., "Selftest passes (7/7 steps)"]
- [ ] [Explicit success indicator 2, e.g., "Task status = Done"]
- [ ] [Explicit success indicator 3, e.g., "No new policy violations"]
- [ ] [Domain-specific indicator, e.g., "AC mapped to tests"]

## Error Handling

### Common Failure 1: [Problem]

**Symptom**: [What you observe]

**Root cause**: [Why it happens]

**Fix**:

```bash
# Recovery steps
```

**Prevention**: [How to avoid next time]

### Common Failure 2: [Problem]

[Repeat structure above]

### Escalation

If you can't resolve:

1. [First escalation step, e.g., "Use governed-governance-debug Skill"]
2. [Second escalation, e.g., "Run xtask -v for verbose output"]
3. [Further help, e.g., "Open issue in repo with logs"]

## Examples

### Example 1: [Scenario Title]

**Goal**: [What the user wants to accomplish]

**Trigger**: User says: "[phrase]"

**Workflow**:

```bash
# Command sequence user can follow
[command 1]
[command 2]
[command 3]
```

**Result**: [Outcome]

### Example 2: [Another Scenario]

[Repeat structure]

## References

- **Flow**: `specs/devex_flows.yaml#[flow-id]`
- **Related ACs**: AC-TPL-SKILLS-YOURNAME-001, etc.
- **Commands**: `cargo xtask help-flows`
- **Platform APIs**: `GET http://localhost:8080/platform/status`
- **Governance**: [SKILLS_GOVERNANCE.md](./SKILLS_GOVERNANCE.md)

---

## Version History

- v1.0.0 (2025-11-27): Initial release aligned with v3.3.6 template

```

---

## Step 4: Pre-Commit Checklist

Before you commit, verify:

### Name & Structure

- [ ] Directory: `.claude/skills/your-skill-name/`
- [ ] File: `.claude/skills/your-skill-name/SKILL.md`
- [ ] Name is kebab-case (lowercase + hyphens only)
- [ ] Name is ≤64 characters
- [ ] Name is unique in project (`git grep` to check)

### Frontmatter

- [ ] YAML valid (no tab characters, proper indentation)
- [ ] `name` field: kebab-case, ≤64 chars
- [ ] `description` field: ≤1024 chars
- [ ] `description` includes WHAT + WHEN
- [ ] `description` mentions flow or commands from devex_flows.yaml
- [ ] `allowed-tools` (if present): least-privilege
- [ ] No hardcoded secrets (API keys, passwords, tokens)

### Body Sections

- [ ] Top-level title (`# Skill Title`)
- [ ] "When to Use" section with explicit triggers
- [ ] "Prerequisites" section with checklist
- [ ] "Workflow" section with detailed steps + actual commands
- [ ] "Exit Criteria" with explicit success indicators
- [ ] "Error Handling" with common failures + recovery
- [ ] "Examples" with 1-2 real scenarios
- [ ] "References" with links to flows, ACs, docs
- [ ] "Version History" with semantic version

### Validation

- [ ] Format:
  ```bash
  cargo xtask skills-fmt
  ```

- [ ] Lint:

  ```bash
  cargo xtask skills-lint
  # Expected: No errors or warnings
  ```

- [ ] Governance:

  ```bash
  cargo xtask selftest
  # Expected: All 7 steps pass
  ```

- [ ] No merge conflicts in `spec_ledger.yaml` or `tasks.yaml`

### Git

- [ ] REQ and ACs added to `specs/spec_ledger.yaml`
- [ ] Task added to `specs/tasks.yaml`
- [ ] SKILL.md created and formatted
- [ ] Commit message: `feat: implement your-skill-name Skill (REQ-TPL-SKILLS-YOURNAME)`

---

## Step 5: Common Pitfalls (Avoid These)

### ❌ Vague Description

```yaml
description: Helps with development
```

✅ **Fix**:

```yaml
description: >
  AC-first feature development workflow. Use when implementing tasks
  tagged 'feature', when user says 'implement AC', or when working
  with specs/spec_ledger.yaml.
```

### ❌ Name Not Kebab-Case

```
your_skill_name          # ❌ Underscore
YourSkillName            # ❌ CamelCase
your-skill-name-is-long-and-exceeds-64-characters-in-length  # ❌ Too long
```

✅ **Fix**:

```
your-skill-name          # ✅ Kebab-case, ≤64 chars
```

### ❌ Missing "When to Use" in Description

```yaml
description: "Runs xtask commands and workflows"
```

✅ **Fix**:

```yaml
description: >
  AC-first feature development workflow. Use when implementing features,
  adding ACs, or working on tasks with status=Todo.
```

### ❌ Unscoped Bash in Read-Only Skill

```yaml
allowed-tools: Read, Grep, Glob, Bash
# ❌ Read-only Skill shouldn't have unscoped Bash
```

✅ **Fix**:

```yaml
allowed-tools: Read, Grep, Glob
# ✅ Minimal tools for read-only governance Skill
```

### ❌ No Reference to devex_flows

```
Workflow

This Skill runs some commands.
```

✅ **Fix**:

```
Workflow

This Skill follows the **ac_first** flow from `specs/devex_flows.yaml`.

1. Discover work
2. Create/update AC
3. Get context bundle
4. Implement code + tests
5. Run BDD tests
6. Validate with selftest
```

### ❌ Commands Without Actual Syntax

```
Step 1: Run the check command

Run the command to validate code quality.
```

✅ **Fix**:

```
Step 1: Validate code quality

```bash
cargo xtask check
```

Expected output: All checks pass

```

---

## Quick Reference: Skill Description Formula

Use this formula to write descriptions that pass governance:

```

[CAPABILITY]: [ACTION DESCRIPTION]
Use when: [TRIGGER 1], [TRIGGER 2], [TRIGGER 3].
Maps to: [FLOW/COMMAND].

```

**Example**:

```

AC-first feature development: Encapsulates the complete workflow for
implementing Requirements and Acceptance Criteria using spec-driven development.
Use when: implementing features, adding new ACs, or working on tasks with status=Todo.
Maps to: ac_first flow from devex_flows.yaml (ac-new, bundle, bdd, selftest).

```

---

## Resources

- **Full Governance Guide**: [SKILLS_GOVERNANCE.md](SKILLS_GOVERNANCE.md)
- **Best Practices & Anti-Patterns**: [AGENT_SKILLS.md](AGENT_SKILLS.md)
- **Architecture Decision Record**: [ADR-0020](adr/0020-claude-code-skills-governance.md)
- **Validation Rules**: `cargo xtask skills-lint -v`
- **Formatting Tool**: `cargo xtask skills-fmt`
- **Examples**: `.claude/skills/governed-*/SKILL.md` in this repo

---

## Need Help?

**Skills-lint error?**
```bash
cargo xtask skills-lint -v  # Verbose output
```

**YAML syntax issues?**
- Use a YAML validator: <https://www.yamllint.com>
- Check indentation (spaces only, no tabs)
- Ensure closing `---` before Markdown body

**Selftest failure?**

```bash
cargo xtask selftest -v  # Verbose selftest
# Check step 4 (Skills validation)
```

**Governance question?**
- Read [SKILLS_GOVERNANCE.md](SKILLS_GOVERNANCE.md) fully
- Check [ADR-0020](adr/0020-claude-code-skills-governance.md)
- Ask team for feedback in PR

---

Good luck! Your Skill is now governed, auditable, and discoverable. 🚀
