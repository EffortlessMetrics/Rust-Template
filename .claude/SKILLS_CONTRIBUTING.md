# Contributing Agent Skills to This Repository

Welcome! This guide shows how to create new Agent Skills for this Rust-as-Spec platform cell.

**Quick Links**:
- 📋 [Skills Governance Guide](../docs/SKILLS_GOVERNANCE.md) — Full governance model
- 📝 [Skills Template](../docs/SKILLS_TEMPLATE.md) — Copy-paste template
- ✅ [Skills Validation Rules](../docs/SKILLS_VALIDATION.md) — What skills-lint checks
- 🏗️ [ADR-0020](../docs/adr/0020-claude-code-skills-governance.md) — Architecture decision

---

## TL;DR: 5-Step Process

### 1. Create REQ/AC in `specs/spec_ledger.yaml`

```yaml
- id: REQ-TPL-SKILLS-YOURNAME
  title: "Your workflow Skill"
  tags: [platform, skills, devex]
  must_have_ac: true
  description: "One-sentence description of workflow"
  adr: [ADR-0020]
  acceptance_criteria:
    - id: AC-TPL-SKILLS-YOURNAME-001
      text: "SKILL.md exists at .claude/skills/your-name/"
      must_have_ac: true
    # Add 3-4 more ACs defining what makes it valid
```

### 2. Create Task in `specs/tasks.yaml`

```yaml
- id: TASK-TPL-SKILLS-YOURNAME-001
  title: "Implement your-name Skill"
  requirement: REQ-TPL-SKILLS-YOURNAME
  acs: [AC-TPL-SKILLS-YOURNAME-001, ...]
  status: Todo
  owner: ""
  labels: [skill, platform]
```

### 3. Create Directory & SKILL.md

```bash
mkdir -p .claude/skills/your-skill-name
cd .claude/skills/your-skill-name

# Use template as starting point
cp ../../docs/SKILLS_TEMPLATE.md SKILL.md
# Edit SKILL.md following template instructions
```

### 4. Validate Locally

```bash
# Format
cargo xtask skills-fmt

# Lint
cargo xtask skills-lint

# Full governance check
cargo xtask selftest
```

### 5. Create PR

```bash
git add .claude/skills/your-skill-name/ specs/spec_ledger.yaml specs/tasks.yaml
git commit -m "feat: implement your-skill-name Skill (REQ-TPL-SKILLS-YOURNAME)"
git push origin feature/your-skill-name
# Create PR with link to REQ/AC IDs
```

---

## Detailed Walkthrough

### Before You Start

**Ask yourself**:

1. ✅ **Is this a workflow?** (sequence of steps, not single command)
2. ✅ **Maps to devex_flows.yaml?** (check `specs/devex_flows.yaml`)
3. ✅ **Multiple team members need this?** (worth Skill vs. inline docs)

**If YES to all**: Proceed. **If NO**: Probably not a Skill.

### Step 1: Define Requirements

Add to **`specs/spec_ledger.yaml`** (find appropriate user story):

```yaml
      - id: REQ-TPL-SKILLS-MYCORP
        title: "MyCorp onboarding workflow Skill"
        tags: [platform, skills, devex]
        must_have_ac: true
        description: >
          The mycorp-onboarding Skill encapsulates the complete workflow
          for setting up a new MyCorp developer environment, including
          secret injection, deployment credentials, and local tools.
        adr: [ADR-0020]
        acceptance_criteria:
          - id: AC-TPL-SKILLS-MYCORP-001
            text: "SKILL.md exists at .claude/skills/mycorp-onboarding/SKILL.md"
            must_have_ac: true
          - id: AC-TPL-SKILLS-MYCORP-002
            text: >
              description states WHAT (workflow) and WHEN (triggers),
              max 1024 chars, includes devex_flows reference or xtask commands
            must_have_ac: true
          - id: AC-TPL-SKILLS-MYCORP-003
            text: >
              allowed-tools follows least-privilege (no unscoped Bash for
              read-only Skill, no hardcoded secrets)
            must_have_ac: true
          - id: AC-TPL-SKILLS-MYCORP-004
            text: "SKILL.md passes cargo xtask skills-lint with no errors"
            must_have_ac: true
```

Add to **`specs/tasks.yaml`**:

```yaml
      - id: TASK-TPL-SKILLS-MYCORP-001
        title: "Implement mycorp-onboarding Skill"
        description: >
          Create and validate the mycorp-onboarding Skill following
          SKILLS_GOVERNANCE.md. Ensure all ACs pass and selftest green.
        requirement: REQ-TPL-SKILLS-MYCORP
        acs:
          - AC-TPL-SKILLS-MYCORP-001
          - AC-TPL-SKILLS-MYCORP-002
          - AC-TPL-SKILLS-MYCORP-003
          - AC-TPL-SKILLS-MYCORP-004
        status: Todo
        owner: ""
        labels: [skill, platform, devex]
```

### Step 2: Create Skill Directory

```bash
mkdir -p .claude/skills/mycorp-onboarding
cd .claude/skills/mycorp-onboarding
```

### Step 3: Write SKILL.md

Copy from [SKILLS_TEMPLATE.md](../docs/SKILLS_TEMPLATE.md) and customize:

```yaml
---
name: mycorp-onboarding
description: >
  Complete MyCorp developer environment setup workflow. Guides new team
  members through secret injection, deployment credential access, and
  local tooling configuration. Use when onboarding new developers,
  when user says "set up my MyCorp environment", or when joining team.
allowed-tools: Read, Bash(git:*), Bash(nix:*), Bash(aws:*)
---

# MyCorp Onboarding Workflow

## When to Use

This Skill is invoked when:

- Onboarding new MyCorp team members
- User says "help me set up my environment"
- First-time MyCorp developer setup on new machine

## Prerequisites

- [ ] Git installed (`git --version`)
- [ ] Nix installed (`nix --version`)
- [ ] AWS CLI installed (`aws --version`)
- [ ] Access to MyCorp AWS account

## Workflow

This Skill follows the **mycorp_onboarding** flow from `specs/devex_flows.yaml`.

### Step 1: Validate Prerequisites

```bash
git --version
nix --version
aws --version
aws sts get-caller-identity
```

Expected: All commands succeed, AWS shows your identity.

### Step 2: Clone & Setup

```bash
git clone https://github.com/mycorp/platform.git
cd platform
nix develop
```

Expected: Nix shell loads with all tools available.

### Step 3: Inject Secrets

```bash
# Download secrets from AWS Secrets Manager
aws secretsmanager get-secret-value \
  --secret-id mycorp/dev-credentials \
  --query SecretString \
  --output text > ~/.mycorp/credentials.json

# Set permissions
chmod 600 ~/.mycorp/credentials.json
```

### Step 4: Verify Setup

```bash
cargo xtask doctor
./scripts/verify-mycorp-setup.sh
```

Expected: All checks pass.

## Exit Criteria

- [ ] Git repo cloned
- [ ] Nix shell active
- [ ] AWS credentials working
- [ ] Local verification script passes
- [ ] `cargo xtask doctor` shows all green

## Error Handling

### Can't authenticate to AWS

```bash
# Check credentials
cat ~/.mycorp/credentials.json

# Or use AWS SSO
aws sso login --profile mycorp-dev
aws sts get-caller-identity --profile mycorp-dev
```

### Nix shell fails to load

```bash
# Update Nix
nix flake update

# Clear cache and retry
rm -rf .nix-cache
nix develop
```

## Examples

### Example 1: First-time setup for new developer

Trigger: "I just joined MyCorp, help me set up"

```bash
# Follow Workflow steps 1-4
# Should take ~10 minutes
```

Result: New developer has working environment

### Example 2: Re-setup after credentials expire

Trigger: "My AWS credentials expired"

```bash
# Run Step 3: Inject Secrets
# Then verify with Step 4
```

Result: Credentials refreshed

## References

- **Flow**: `specs/devex_flows.yaml#mycorp_onboarding`
- **Related ACs**: AC-TPL-SKILLS-MYCORP-001, ...-004
- **Governance**: [SKILLS_GOVERNANCE.md](../../docs/SKILLS_GOVERNANCE.md)
- **Setup Script**: `./scripts/verify-mycorp-setup.sh`

---

## Version History

- v1.0.0 (2025-11-27): Initial release
```

### Step 4: Format & Validate

```bash
# Format
cargo xtask skills-fmt

# Check for errors
cargo xtask skills-lint

# Expected output:
# ✓ mycorp-onboarding: Valid
#   - name: mycorp-onboarding (kebab-case, 19 chars)
#   - description: 240 chars, includes WHAT + WHEN
#   - allowed-tools: appropriate scoping
#   - references: mentions mycorp_onboarding flow

# Full governance check
cargo xtask check
cargo xtask selftest

# Expected: Step 4 "Skills governance validation" passes
```

### Step 5: Create PR

```bash
# Verify clean state
git status

# Stage changes
git add .claude/skills/mycorp-onboarding/
git add specs/spec_ledger.yaml
git add specs/tasks.yaml

# Commit
git commit -m "feat: implement mycorp-onboarding Skill (REQ-TPL-SKILLS-MYCORP)"

# Push
git push origin feature/mycorp-onboarding

# Create PR with description linking to REQ/AC IDs:
# - REQ-TPL-SKILLS-MYCORP
# - AC-TPL-SKILLS-MYCORP-001, 002, 003, 004
```

---

## Checklist Before Committing

Use this checklist to ensure your Skill passes governance:

### Governance

- [ ] REQ created in `specs/spec_ledger.yaml` (REQ-TPL-SKILLS-YOURNAME)
- [ ] ACs defined for SKILL.md structure (AC-TPL-SKILLS-YOURNAME-001, ...)
- [ ] Task created in `specs/tasks.yaml` linking to REQ/ACs

### SKILL.md Structure

- [ ] Located at `.claude/skills/your-skill-name/SKILL.md`
- [ ] YAML frontmatter valid (no tabs, proper `---` delimiters)
- [ ] `name` field: kebab-case, ≤64 chars, unique
- [ ] `description` field: ≤1024 chars, includes WHAT + WHEN

### Content Quality

- [ ] "When to Use" section with explicit triggers
- [ ] "Prerequisites" with checklist items
- [ ] "Workflow" with detailed steps and actual commands
- [ ] "Exit Criteria" with explicit success indicators
- [ ] "Error Handling" with common failures + recovery
- [ ] "Examples" with 1-2 real scenarios
- [ ] "References" linking to flows, ACs, docs
- [ ] "Version History" with semantic version

### Validation

- [ ] `cargo xtask skills-fmt` runs without errors
- [ ] `cargo xtask skills-lint` shows ✓ all checks pass
- [ ] `cargo xtask check` passes
- [ ] `cargo xtask selftest` passes (all 7 steps, especially step 4)

### Git

- [ ] No merge conflicts
- [ ] Commit message references REQ/AC IDs
- [ ] PR description explains motivation
- [ ] PR links to Skill governance doc

---

## Common Mistakes (Don't Do These)

### ❌ Creating Skill without REQ/AC

**Wrong**:
```bash
mkdir .claude/skills/my-skill
# Create SKILL.md without governance
```

**Right**:
```yaml
# First, add REQ + AC to spec_ledger.yaml
# Then create SKILL.md
# Then validate with selftest
```

### ❌ Vague description

**Wrong**:
```yaml
description: "Helps with development"
```

**Right**:
```yaml
description: >
  AC-first feature development workflow. Use when implementing tasks
  tagged 'feature', when user says 'implement AC', or when working
  with specs/spec_ledger.yaml.
```

### ❌ One Skill per command

**Wrong**:
```
.claude/skills/skill-check/SKILL.md
.claude/skills/skill-bdd/SKILL.md
.claude/skills/skill-bundle/SKILL.md
```

**Right**:
```
.claude/skills/governed-feature-dev/SKILL.md  # Wraps entire ac_first flow
```

### ❌ No validation before PR

**Wrong**: Push without running `skills-lint` and `selftest`

**Right**:
```bash
cargo xtask skills-lint    # No errors
cargo xtask selftest       # All steps pass
# Then create PR
```

---

## Questions?

1. **"Should I create a Skill?"** → Read [SKILLS_GOVERNANCE.md](../docs/SKILLS_GOVERNANCE.md) §7 (Common Questions)

2. **"skills-lint is failing"** → Check [SKILLS_VALIDATION.md](../docs/SKILLS_VALIDATION.md) for specific rules

3. **"My description is vague"** → Use formula in [SKILLS_TEMPLATE.md](../docs/SKILLS_TEMPLATE.md) Quick Reference

4. **"I need an exception to governance"** → Open issue linking to ADR-0020 with justification

---

## Resources

| Resource | Purpose |
|----------|---------|
| [SKILLS_GOVERNANCE.md](../docs/SKILLS_GOVERNANCE.md) | Full governance model, lifecycle, FAQ |
| [SKILLS_TEMPLATE.md](../docs/SKILLS_TEMPLATE.md) | SKILL.md template + detailed checklist |
| [SKILLS_VALIDATION.md](../docs/SKILLS_VALIDATION.md) | Exact lint rules and implementation details |
| [ADR-0020](../docs/adr/0020-claude-code-skills-governance.md) | Architecture decision and rationale |
| [AGENT_SKILLS.md](../docs/AGENT_SKILLS.md) | Best practices, anti-patterns, examples |

---

## Success Example

Here's what a complete, governance-compliant Skill looks like:

- ✅ `REQ-TPL-SKILLS-EXAMPLE` in `spec_ledger.yaml` with 4 ACs
- ✅ `TASK-TPL-SKILLS-EXAMPLE-001` in `tasks.yaml`
- ✅ `.claude/skills/example-workflow/SKILL.md` with all sections
- ✅ `cargo xtask skills-lint` passes ✓
- ✅ `cargo xtask selftest` passes ✓
- ✅ PR with clear description linking governance artifacts

You're ready to contribute! 🚀
