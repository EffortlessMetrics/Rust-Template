# Contributing to Claude Code Agents

**Quick Reference for Developers**
**See also**: `docs/AGENTS_GOVERNANCE.md`, `docs/AGENTS_TEMPLATE.md`, `docs/AGENTS_VALIDATION.md`

---

## 5-Step Workflow

### 1. Plan in Spec (5 mins)

Add your agent to `specs/spec_ledger.yaml`:

```yaml
- id: REQ-YOUR-TEAM-AGENTS-NAME
  title: "Short description of agent"
  tags: [platform, agent, governance]
  must_have_ac: true
  description: >
    What problem does it solve? When should it be used?
  acceptance_criteria:
    - id: AC-YOUR-TEAM-AGENTS-001
      text: "Agent correctly implements workflow X"
      tags: [kernel]
      must_have_ac: true
      tests:
        - { type: bdd, tag: "@AC-YOUR-TEAM-AGENTS-001", file: "specs/features/xtask_devex.feature" }
```

### 2. Create Agent File (10 mins)

Copy `docs/AGENTS_TEMPLATE.md` to `.claude/agents/your-agent-name.md`:

```bash
cp docs/AGENTS_TEMPLATE.md .claude/agents/your-agent-name.md
```

Edit the file:
- Fill in YAML frontmatter (name, description, tools, model, permissionMode, skills)
- Write system prompt: Role, Workflow, Tool Usage, Safety & Constraints
- Follow the template checklist

### 3. Validate Locally (2 mins)

```bash
# Run agents-lint
cargo run -p xtask -- agents-lint

# Check precommit
cargo run -p xtask -- precommit

# Run selftest (full validation)
cargo run -p xtask -- selftest
```

All should pass (agents-lint errors block, warnings are OK).

### 4. Test in Practice (varies)

Use your agent in a Claude Code session or script:
- Does it behave as documented?
- Does it handle edge cases?
- Are tools sufficient (no permission issues)?

### 5. Commit & PR (5 mins)

```bash
git add .claude/agents/your-agent-name.md specs/spec_ledger.yaml
git commit -m "Add your-agent-name agent (REQ-YOUR-TEAM-AGENTS-NAME, AC-YOUR-TEAM-AGENTS-001)"
git push && gh pr create --title "Add your-agent-name agent" --body "..."
```

CI will run:
- `agents-lint` (via .github/workflows/ci-agents.yml)
- `selftest` (Step 3/10)
- All agents governance checks

---

## Troubleshooting

### agents-lint says "name must match filename"
File is `.claude/agents/my-agent.md` but frontmatter says `name: my_agent`?
- **Fix**: Make them match: `name: my-agent`

### agents-lint says "Skill X not found"
You referenced `skills: my-skill` but it doesn't exist?
- **Fix**: Either create the Skill in `.claude/skills/my-skill/SKILL.md` or remove the reference

### agents-lint says "Tabs found"
Your editor inserted tabs instead of spaces?
- **Fix**: Open agent file, replace all tabs with spaces (configure editor)

### Precommit fails on "agents-lint"
Changes to `.claude/agents/` or spec triggered lint?
- **Fix**: Run `cargo xtask agents-lint` to see errors and fix them

### selftest Step 3 fails (Agents governance)
One or more agents are invalid?
- **Fix**: Run `cargo xtask agents-lint` to list issues; fix each one

---

## Common Questions

**Q: What if I just want to add a simple read-only agent?**
A: Use tools: `Read, Grep, Glob` and `permissionMode: default`. No special justification needed.

**Q: Should I always add to spec_ledger.yaml?**
A: Yes. If the agent is worth including in `.claude/agents/`, it's worth governing. (Skip only for local experiments in `~/.claude/agents/`.)

**Q: What's the difference between an Agent and a Skill?**
A:
- **Skill**: Reusable capability (e.g., "governed-feature-dev workflow")
- **Agent**: Uses skills to solve a specific problem (e.g., "code-reviewer agent")

**Q: Can my agent use multiple Skills?**
A: Yes. List them comma-separated: `skills: governed-feature-dev, governed-maintenance`

**Q: What if I want to use opus model?**
A: Document why in the `description` field. agents-lint will warn but allow it. PR reviewer will check the justification.

**Q: Can I make my agent private or hidden?**
A: Yes, local agents in `~/.claude/agents/` are not governed. But if it's useful for the team, consider adding to the repo.

---

## File Structure

```
.claude/agents/
  ├── code-reviewer.md           # Read-only code analysis
  ├── test-runner.md             # Runs tests and reports
  ├── feature-implement.md       # Full feature development
  └── ...

docs/
  ├── AGENTS_GOVERNANCE.md       # Governance principles
  ├── AGENTS_TEMPLATE.md         # Copy-paste template
  ├── AGENTS_VALIDATION.md       # Lint rules
  └── AGENTS_CONTRIBUTING.md     # This file (quickstart)

specs/
  ├── spec_ledger.yaml           # REQs and ACs for all agents
  └── features/
      └── xtask_devex.feature    # BDD scenarios for agents (planned)
```

---

## Key Commands

```bash
# Lint agents
cargo run -p xtask -- agents-lint

# Pre-commit checks (runs agents-lint if changed)
cargo run -p xtask -- precommit

# Full selftest (includes Step 3: Agents governance)
cargo run -p xtask -- selftest

# View agent governance docs
less docs/AGENTS_GOVERNANCE.md

# Copy agent template
cp docs/AGENTS_TEMPLATE.md .claude/agents/my-new-agent.md
```

---

## Governance Gates

Your agent must pass:

1. **Local validation**: `cargo xtask agents-lint` (no errors)
2. **Precommit**: `cargo xtask precommit` (agents lint + all other checks)
3. **CI**: `.github/workflows/ci-agents.yml` (path-filtered job for agent changes)
4. **Selftest**: Step 3/10 in full governance suite
5. **PR review**: Team lead verifies justification for high-risk modes (opus, bypassPermissions)

---

## When to NOT Create an Agent

- **Use an existing agent** if one already does the job
- **Use a Skill** if you're building a reusable workflow (not a specific tool)
- **Use local ~/.claude/agents/** if it's just for personal experimentation

---

## Checklists

### Before Commit
- [ ] `cargo xtask agents-lint` passes (no errors)
- [ ] `cargo xtask precommit` passes
- [ ] Agent file matches spec_ledger.yaml REQ/AC IDs
- [ ] Commit message includes REQ/AC IDs
- [ ] No hardcoded secrets (API keys, tokens, credentials)

### Before PR Merge
- [ ] CI job `.github/workflows/ci-agents.yml` passed
- [ ] `selftest` Step 3/10 passed
- [ ] PR review approved (especially for high-risk modes)
- [ ] BDD scenarios exist (if AC requires them)

---

**Questions?** See the full docs in `docs/AGENTS_GOVERNANCE.md` or ask the team.

**Version**: 1.0.0
