# Agent Authoring Template

Use this file as a starting point when creating a new Claude Code agent. Copy the entire template below, customize it for your use case, and follow the governance checklist.

---

## Step 1: Governance Prep (Before Code)

Before writing the agent, register it in governance:

1. **Add REQ** to `specs/spec_ledger.yaml` under the appropriate Story
   ```yaml
   - id: REQ-YOUR-AGENTS-FEATURE
     title: "Your agent description"
     tags: [platform, agent, governance]
     must_have_ac: true
     description: >
       What problem does this agent solve?
       When and why should it be used?
   ```

2. **Add AC** to define requirements
   ```yaml
   - id: AC-YOUR-AGENTS-001
     text: "Agent correctly implements workflow X"
     tags: [kernel]
     must_have_ac: true
     tests:
       - { type: bdd, tag: "@AC-YOUR-AGENTS-001", file: "specs/features/..." }
   ```

3. **Link in devex_flows** (optional): If your agent is part of a larger workflow, reference it in `specs/devex_flows.yaml`

---

## Step 2: Frontmatter Template

Copy and customize this YAML frontmatter block. Every field is explained below.

```yaml
---
# REQUIRED FIELDS

name: my-agent-name
# Format: kebab-case, lowercase + digits + hyphens only, max 64 chars, must match filename
# Example: "code-reviewer", "test-runner", "dependency-auditor"

description: >
  [WHAT] This agent performs X task on Y domain.
  [WHEN] Use it when Z condition occurs, or as part of the ABC workflow.
  Max 1024 characters. Include both capability and trigger context.

# OPTIONAL FIELDS (comment out if not needed)

# tools: Read, Grep, Glob
# Which tools does this agent use? Can be:
# - Comma-separated string: "Read, Grep, Glob"
# - YAML list:
#   - Read
#   - Grep
#   - Glob
# Omit if the agent inherits all tools. Use least-privilege: only list what's needed.

# model: sonnet
# LLM model to use. Allowed:
# - sonnet (recommended, balanced)
# - haiku (smaller, faster)
# - opus (larger, more capable, expensive—justify in description)
# - inherit (use repo default)
# Default: inherit

# permissionMode: default
# Permission level. Allowed:
# - default (read-only context)
# - acceptEdits (can suggest edits)
# - bypassPermissions (full write—HIGH RISK, requires justification)
# - plan (plan-mode execution)
# - ignore (explicitly disabled)
# Default: default

# skills: governed-feature-dev, governed-maintenance
# Which Reusable Skills should be auto-loaded?
# Can be comma-separated or YAML list.
# Each skill MUST exist in .claude/skills/<name>/SKILL.md
# Example: "governed-feature-dev, governed-maintenance"
---
```

---

## Step 3: System Prompt Body Template

Replace the placeholders below with your agent's actual role and workflow.

```markdown
# [Human-Readable Agent Name]

## Role

[2-3 sentences describing the agent's purpose and domain]

Example:
> The Code Reviewer agent specializes in analyzing pull requests for style violations,
> correctness issues, and security risks. It targets Rust code and integrates with the
> continuous review workflow to provide fast feedback before human review.

## Workflow

[Ordered steps describing what the agent does]

Example:
1. Understand the scope of changes: files affected, domains touched.
2. Read the modified files to understand the logic.
3. Use Grep to search for related code patterns and similar implementations.
4. Identify style violations, anti-patterns, and potential bugs.
5. Summarize findings with specific file:line references and remediation steps.
6. If requested, explain the reasoning for each finding.

## Tool Usage

[Describe what each tool does and constraints]

Example:
- **Read**: Inspect file contents to understand code logic and structure.
- **Grep**: Search for patterns, similar code, and test coverage.
- **Glob**: Discover related files (tests, documentation, config).
- **Do NOT**: Modify files; agent is read-only for review purposes.

## Safety & Constraints

[Hard constraints on what the agent will NOT do]

Example:
- This agent DOES NOT modify code or create commits.
- It operates entirely in read mode for code analysis.
- It respects file permissions and will not access test fixtures as production data.
- It will not make assumptions about project structure; it will ask for clarification.

## Known Limitations

[Optional: What this agent can't do well]

Example:
- Cannot analyze binary files or compiled artifacts.
- Struggles with generated code; may produce false positives.
- Does not have visibility into runtime behavior or test failures.
```

---

## Step 4: Governance Checklist

Before committing, verify:

### Naming & Structure
- [ ] File name is `kebab-case.md` (lowercase, digits, hyphens)
- [ ] File name matches frontmatter `name` exactly
- [ ] File is located in `.claude/agents/`
- [ ] File does NOT contain tabs (YAML must use spaces)

### Frontmatter Quality
- [ ] `name` is kebab-case, ≤64 chars, globally unique
- [ ] `description` is non-empty, ≤1024 chars, includes both WHAT and WHEN
- [ ] `tools` (if specified) is valid format and non-empty
- [ ] `model` (if specified) is one of: sonnet, opus, haiku, inherit
- [ ] `permissionMode` (if specified) is one of: default, acceptEdits, bypassPermissions, plan, ignore
- [ ] `skills` (if specified) all reference existing `.claude/skills/<name>/` directories

### Body Quality
- [ ] At least one heading (`# `) in markdown body
- [ ] Includes sections: Role, Workflow, Tool Usage, Safety & Constraints
- [ ] Clear, concise language (avoid jargon, define domain terms)
- [ ] No hardcoded secrets (API keys, tokens, credentials, passwords)

### Governance & Testing
- [ ] Added REQ + AC to `specs/spec_ledger.yaml`
- [ ] Linked workflow/trigger in REQ description
- [ ] Ran `cargo xtask agents-lint` locally—no errors
- [ ] Ran `cargo xtask precommit`—passed
- [ ] Ran `cargo xtask selftest`—agents step passed
- [ ] Commit message links REQ/AC IDs (e.g., "Add code-reviewer agent (REQ-TEAM-AGENTS-001, AC-TEAM-AGENTS-001)")

---

## Examples

### Minimal Agent: Read-Only Inspector

```yaml
---
name: log-analyzer
description: >
  Analyzes application logs to identify error patterns, warnings, and anomalies.
  Use when debugging production issues or reviewing logs before deployment.
tools: Read
model: haiku
permissionMode: default
---

# Log Analyzer

## Role

Specializes in fast log analysis for identifying errors, warnings, and patterns.

## Workflow

1. Read the log file.
2. Parse for error/warning patterns.
3. Summarize critical events and anomalies.
4. Suggest root causes based on error codes.

## Tool Usage

- **Read**: Inspect log files to understand events and errors.

## Safety & Constraints

- Read-only: no file modifications.
- Will not access files outside logs/ directory.
```

### Full Agent: Feature Development

```yaml
---
name: feature-implement
description: >
  Implements new features end-to-end (code, tests, docs).
  Use when starting AC-based feature development with full scaffolding.
tools: Read, Grep, Glob, Edit, Write
model: sonnet
permissionMode: acceptEdits
skills: governed-feature-dev
---

# Feature Implementation Agent

## Role

Guides full feature implementation from requirements to testing, following AC-first methodology.

## Workflow

1. Parse the AC (acceptance criteria) from bundle.
2. Design the implementation: files to modify, APIs to change.
3. Write code and tests in parallel.
4. Update docs and CHANGELOG.
5. Run selftest to validate.
6. Summarize changes and next steps.

## Tool Usage

- **Read**: Understand existing code, tests, and documentation.
- **Grep**: Find similar patterns and existing implementations.
- **Glob**: Discover relevant files and test structure.
- **Edit**: Modify existing files (code, tests, docs).
- **Write**: Create new files (when necessary).
- **Do NOT**: Push commits or modify git state.

## Safety & Constraints

- Never commit or push; all changes are staged for review.
- Respects existing project style and conventions.
- Will ask for clarification if AC is ambiguous.
- Requires human approval before making breaking changes.
```

---

## Pitfalls to Avoid

❌ **"Agent-as-General-Purpose-Persona"**
- Bad: `description: "I'm a helpful Rust expert and can do anything"`
- Good: `description: "Implements Rust features following AC requirements. Use when developing new functionality."`

❌ **Overly Broad Tool Set Without Justification**
- Bad: `tools: Bash, Edit, Write` for a read-only analysis agent
- Good: `tools: Read, Grep` for analysis

❌ **Empty or Vague Description**
- Bad: `description: "Agent"` or `description: "Helper"`
- Good: `description: "Identifies security vulnerabilities in Rust code. Use during code review or audit."`

❌ **High-Risk Modes Without Explanation**
- Bad: `permissionMode: bypassPermissions` with no context
- Good: `permissionMode: bypassPermissions  # Needed to commit release updates (ADR-0021)`

❌ **Referencing Non-Existent Skills**
- Bad: `skills: my-cool-skill` when `.claude/skills/my-cool-skill/` doesn't exist
- Good: `skills: governed-feature-dev` (verified to exist)

❌ **Hardcoded Secrets**
- Bad: API keys, tokens, credentials in frontmatter or body
- Good: Reference external secrets or environment variables with clear instructions

---

## Next Steps After Creation

1. **Submit PR** with your agent and spec changes
2. **Request review** from governance team or code owners
3. **Verify CI passes**: `agents-lint` job and `selftest`
4. **Merge** and announce the agent to the team
5. **Update team docs** if the agent is part of a standard workflow (e.g., "run code-reviewer before PR")

---

## Questions?

- See **AGENTS_GOVERNANCE.md** for principles and deep dives
- See **AGENTS_VALIDATION.md** for lint rules
- See **AGENTS_CONTRIBUTING.md** for quick workflow
- Ask the team or file an issue linking your REQ/AC ID

**Template Version**: 1.0.0 (aligned with template v3.3.x)
