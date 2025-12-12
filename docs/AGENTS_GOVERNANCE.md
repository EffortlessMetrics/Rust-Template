# Agents Governance: Structured, Auditable, Safe

**Audience**: Team leads, agent authors, repo maintainers
**Status**: Governed by REQ-TPL-AGENTS-GOVERNANCE, ADR-0021
**Version**: v3.3.11

## 1. Quick Summary

Agents (Claude Code agents defined in `.claude/agents/*.md`) are not ad-hoc prompts but **governed artifacts** with:

- **YAML frontmatter** (name, description, tools, model, permissionMode, skills)
- **System prompt body** (role, workflow, constraints)
- **Project-level REQ/AC** in `specs/spec_ledger.yaml`
- **Validation via** `cargo xtask agents-lint`
- **CI/CD + precommit integration** for continuous enforcement

This ensures agents are:
- **Discoverable**: Catalogued and linked to requirements
- **Safe**: Validated for least-privilege tools and permissions
- **Maintainable**: Aligned with Skills and DevEx flows
- **Auditable**: Traced from decision (ADR-0021) to implementation

---

## 2. Governance Principles

1. **Agents are workflows, not prompts**
   Each agent solves a specific problem with clear scope. Not a general-purpose persona.

2. **Least-privilege tools & permissions**
   Only include tools and permissionMode settings necessary for the workflow.

3. **Model usage is explicit and cost-aware**
   - Default to `inherit` or repo-approved aliases
   - Justify expensive models (opus) or warn on cost

4. **Skills references resolve**
   If an agent lists `skills`, those Skills must exist in `.claude/skills/`

5. **Kernel vs. template ACs**:
   - **Kernel ACs** (errors): Name format, description non-empty, permissionMode valid, skills exist
   - **Template ACs** (warnings): Description quality, broad tool combos, expensive models

---

## 3. File Layout

### Project Agents (Governed)
- Location: `.claude/agents/*.md`
- Scope: Repo agents, enforced by selftest, CI, precommit
- Naming: `kebab-case.md`, max 64 chars
- REQ/AC: Each must have an entry in `specs/spec_ledger.yaml`

### User Agents (Out-of-Scope)
- Location: `~/.claude/agents/*.md`
- Scope: Local only, not governed by repo

### Plugin Agents (External)
- Shipped via Claude Code plugin ecosystem
- Documented but not validated by this repo's governance

---

## 4. Frontmatter Requirements

All agents MUST have YAML frontmatter with these fields:

```yaml
---
name: agent-identifier
description: >
  WHAT this agent does and WHEN to use it.
  Max 1024 characters. Include both purpose and trigger context.

# Optional fields:
tools: Read, Glob, Grep           # List or comma-separated string
model: sonnet                       # sonnet|opus|haiku|inherit (default: inherit)
permissionMode: default             # default|acceptEdits|bypassPermissions|plan|ignore
skills: governed-feature-dev        # Comma-separated or list
---
```

### Field Details

**`name`** (required, error on fail)
- Format: kebab-case (lowercase, digits, hyphens only)
- Length: 1–64 characters
- Uniqueness: Must be unique within the project
- Match file: name MUST equal filename without extension

**`description`** (required, error on fail)
- Minimum: Non-empty, ≤1024 chars
- Quality check (warning):
  - Should include both "what" and "when to use" or trigger context
  - Look for keywords: "when", "use when", "trigger", "if", "context"

**`tools`** (optional)
- Format: YAML list or comma-separated string
- Valid examples: `Read, Grep, Glob` or `[Read, Grep, Glob]`
- Error: Empty if specified, or invalid format

**`model`** (optional, defaults to inherit)
- Allowed: `sonnet`, `opus`, `haiku`, `inherit`
- Error: Unknown model name
- Warning: `opus` (expensive) without justification

**`permissionMode`** (optional, defaults to `default`)
- Allowed: `default`, `acceptEdits`, `bypassPermissions`, `plan`, `ignore`
- Error: Unknown mode
- Warning: `bypassPermissions` (high-risk) without review

**`skills`** (optional)
- Format: YAML list or comma-separated string
- Validation: Each skill name MUST have a matching `.claude/skills/<name>/SKILL.md` file
- Error: Referenced skill does not exist

---

## 5. Safety & Permission Modes

### Permission Modes Explained

- **`default`**: Standard read-only access to provided context
- **`acceptEdits`**: Agent can suggest edits to files (but not commit)
- **`bypassPermissions`**: Full write access (high-risk, requires justification)
- **`plan`**: Plan-mode execution (no file modifications)
- **`ignore`**: Explicitly disabled (use when agent exists but should not run)

### Tool Safety

Least-privilege checklist:
- ✅ Read-only agents should have `Read`, `Grep`, `Glob` only
- ✅ Code editors should have `Read`, `Edit`, `Glob`, not `Write` to arbitrary paths
- ⚠️ Broad combinations (Bash + Edit + Write + full permissions) require justification
- ❌ Never include secrets, API keys, or credentials in agent body

agents-lint will:
- **ERROR** on invalid permissionMode or unknown tools
- **WARN** on broad tool combos or high-risk modes without context

---

## 6. Lifecycle

### Create an Agent

1. **Define in spec**: Add REQ + AC(s) to `specs/spec_ledger.yaml`
   - Link to workflow/flow it supports
   - Define acceptance criteria (e.g., "correctly implements AC-XYZ")

2. **Copy template**: Use `docs/AGENTS_TEMPLATE.md` as starting point

3. **Write frontmatter + body**:
   - YAML frontmatter (use template checklist)
   - Markdown body: role, workflow, tools usage, safety constraints

4. **Validate locally**:
   ```bash
   cargo run -p xtask -- agents-lint
   ```

5. **Run precommit & selftest**:
   ```bash
   cargo run -p xtask -- precommit
   cargo run -p xtask -- selftest
   ```

6. **Submit PR**: Link REQ/AC IDs in commit message

### Maintain an Agent

- Keep `description` and `tools` aligned with actual workflow
- If adding `skills`, ensure those Skills still exist
- Update `model` policy if repo defaults change
- Test with `cargo xtask agents-lint` after edits
- Precommit will auto-validate on commit

### Retire an Agent

1. Mark REQ as `deprecated: true` in `specs/spec_ledger.yaml`
2. Move agent file to `.claude/agents_archived/` (optional)
3. Or delete completely if no historical value
4. Update any docs/flows that referenced the agent

---

## 7. Integration Points

### `cargo xtask agents-lint`
- Validates all agents in `.claude/agents/`
- Errors block agent use; warnings improve quality
- Exit code: 0 (pass), 1 (errors found)
- Run manually: `cargo run -p xtask -- agents-lint`

### Selftest (Step 3/10)
- Runs agents-lint as part of full governance check
- Skips if `.claude/agents/` directory does not exist
- Failures in agents-lint will fail selftest

### Precommit
- Runs automatically when `.claude/agents/**` changes
- Also triggers on spec_ledger.yaml or agents.rs changes
- Skipped if no changes detected (fast-path)
- Errors fail the commit; warnings are informational

### CI Workflow (`.github/workflows/ci-agents.yml`)
- Path-filtered job: only runs when agents or spec changes
- Runs `cargo xtask agents-lint`
- Blocks PR if errors found
- Encourages review of governance changes

---

## 8. Examples

### ✅ Good Agent: Code Reviewer

```yaml
---
name: code-reviewer
description: >
  Reviews pull requests for style, correctness, and security issues.
  Use when you need feedback on code quality before merge.
tools: Read, Grep, Glob
model: sonnet
permissionMode: default
skills: governed-maintenance
---

# Code Reviewer Agent

## Role

Specialized in identifying bugs, style violations, and security risks in code...

## Workflow

1. Understand the code change scope (files, domain).
2. Use Read/Grep to inspect modified files.
3. Check for common anti-patterns and best practices.
4. Summarize findings with specific file:line references.

## Tool Usage

- **Read**: Inspect file contents to understand logic
- **Grep**: Search for related code patterns
- **Glob**: Discover test files and related modules

## Safety & Constraints

- Read-only agent: no modifications to code
- Does not make commits or push branches
```

### ❌ Bad Agent: Missing WHEN clause

```yaml
---
name: debugger
description: >
  Debugs code issues.  # ERROR: No "when" context!
  # Should say: "...when you encounter unexpected behavior or test failures"
tools: Read, Grep, Glob, Edit, Write, Bash
model: opus  # WARNING: expensive without justification
permissionMode: bypassPermissions  # WARNING: high-risk without doc reference
---
```

---

## 9. Comparison: Agents vs. Skills

| Aspect | Agents | Skills |
|--------|--------|--------|
| **Location** | `.claude/agents/*.md` | `.claude/skills/*/SKILL.md` |
| **What it is** | A tool/workflow | A reusable capability |
| **Used by** | Claude Code (xtask invoke agent) | Agents (via `skills:` field) |
| **Governance** | REQ + AC per agent | REQ + AC per skill |
| **Example** | "code-reviewer", "test-runner" | "governed-feature-dev", "audit-log" |

Agents often *use* Skills to accomplish their workflows. Both are governed.

---

## 10. Troubleshooting

### agents-lint says "name must equal file name"
- Your frontmatter `name` doesn't match the filename
- Example: File is `my-agent.md`, but frontmatter says `name: myagent`
- **Fix**: Make them match: `name: my-agent`

### agents-lint says "Skill 'X' not found"
- Agent references `skills: my-skill` but `.claude/skills/my-skill/` doesn't exist
- **Fix**: Either create the Skill or remove the reference

### agents-lint says "invalid permissionMode"
- You used an undefined mode like `permissionMode: sudo`
- **Fix**: Use one of: `default`, `acceptEdits`, `bypassPermissions`, `plan`, `ignore`

### agents-lint says "Tabs found in YAML"
- YAML must use spaces, not tabs
- **Fix**: Replace tabs with spaces (2 or 4)

---

## 11. Related Documents

- **ADR-0021**: Architecture decision for agents governance
- **AGENTS_TEMPLATE.md**: Copy-paste starter for new agents
- **AGENTS_VALIDATION.md**: Detailed lint rules and phases
- **AGENTS_CONTRIBUTING.md**: Quick workflow for developers
- **CLAUDE.md**: Platform cell orientation (how to use agents in workflows)

---

## Appendix: Validation Checklist

Before committing an agent:

- [ ] Name is kebab-case, ≤64 chars, unique
- [ ] Description includes both "what" and "when to use" (≤1024 chars)
- [ ] File name matches frontmatter `name`
- [ ] `tools` field is valid (list or comma-separated string)
- [ ] `model` is one of: sonnet, opus, haiku, inherit
- [ ] `permissionMode` is valid and justified if high-risk
- [ ] All `skills` references exist in `.claude/skills/`
- [ ] No hardcoded secrets (API keys, tokens, credentials)
- [ ] Markdown body has at least one heading (#...)
- [ ] Markdown body documents: Role, Workflow, Tool Usage, Safety
- [ ] `cargo xtask agents-lint` passes with no errors
- [ ] `cargo xtask precommit` passes
- [ ] Linked REQ/AC in spec_ledger.yaml

---

**Last Updated**: 2025-12-01
**Template Version**: 3.3.8
