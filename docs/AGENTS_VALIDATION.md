# Agents Validation Rules

**Status**: Phase 1 (implemented) / Phase 2 (planned) / Phase 3 (integration)
**Last Updated**: 2025-11-27
**Audience**: Developers, reviewers, automation engineers

---

## Overview

This document describes the validation rules that `agents-lint` enforces on Claude Code agents (`.claude/agents/*.md`). The rules are split into:

- **Errors** (kernel ACs): Hard failures that block agent use
- **Warnings** (template ACs): Quality guidance that improves agents but doesn't block

---

## Phase 1: Core Validation (IMPLEMENTED)

These rules are enforced by `cargo xtask agents-lint` today.

### 1. YAML Parsing & Frontmatter

**Error: Tabs in frontmatter**
- Rule: First 20 lines MUST use spaces, not tabs
- Reason: YAML spec requires spaces; tabs cause parse errors
- Fix: Replace tabs with spaces

**Error: Missing frontmatter**
- Rule: File MUST start with `---` (closing `---` required)
- Reason: Frontmatter contains required metadata
- Fix: Add frontmatter block at line 1

**Error: YAML parse error**
- Rule: Frontmatter MUST be valid YAML
- Example failures: Malformed keys, invalid syntax
- Fix: Validate YAML syntax (e.g., check for mismatched quotes, invalid nesting)

### 2. Name Field (AC-TPL-AGENTS-NAME-FORMAT)

**Error: Missing or not a string**
- Rule: `name:` field MUST be present and a string
- Example: ✅ `name: my-agent` ❌ `name: 123` (number)

**Error: Name format (kebab-case)**
- Rule: `name` MUST match regex `^[a-z0-9-]{1,64}$`
- Allowed: lowercase letters, digits, hyphens
- Forbidden: UPPERCASE, underscores, spaces, special chars
- Example: ✅ `code-reviewer-v2` ❌ `Code_Reviewer` ❌ `code reviewer`

**Error: Name length**
- Rule: `name` MUST be 1–64 characters
- Example: ✅ `my-agent` (8 chars) ❌ `a_very_long_name_that_exceeds_sixty_four_characters_and_should_fail` (>64)

**Error: Name must match filename**
- Rule: Frontmatter `name` MUST equal file name without `.md` extension
- Example:
  - File: `.claude/agents/code-reviewer.md`
  - Frontmatter: `name: code-reviewer` ✅
  - Frontmatter: `name: codereviewer` ❌

### 3. Description Field (AC-TPL-AGENTS-DESCRIPTION-QUALITY)

**Error: Missing or empty**
- Rule: `description:` field MUST be present and non-empty
- Example: ❌ (missing) or ❌ `description: ""` or ❌ `description: null`

**Error: Length limit**
- Rule: `description` MUST be ≤1024 characters
- Reason: Keeps descriptions concise and prevents bloat
- Fix: Summarize key points; move detailed workflow to body

**Warning: Missing WHEN context**
- Rule: Description should include both WHAT and WHEN
- Heuristic: Lint searches for keywords: "when", "use when", "trigger", "if", "context"
- Example:
  - ✅ "Analyzes code for bugs and style issues. Use when reviewing pull requests."
  - ⚠️ "Analyzes code for bugs" (missing WHEN context)
- Severity: Warning (doesn't block, improves UX)

### 4. Tools Field (AC-TPL-AGENTS-TOOLS-PERMISSION-SAFETY)

**Error: Invalid format**
- Rule: `tools` MUST be a YAML list or comma-separated string
- Valid:
  ```yaml
  tools: Read, Grep, Glob
  ```
  or
  ```yaml
  tools:
    - Read
    - Grep
    - Glob
  ```
- Invalid: ❌ `tools: {Read: true}` ❌ `tools: 123`

**Error: Empty if specified**
- Rule: If `tools:` is present, it MUST not be empty
- Example: ❌ `tools: ""` or ❌ `tools: []`
- Fix: Omit field if not needed; agents inherit all tools by default

**Warning: Broad tool combinations**
- Rule: Lint warns if agent has Bash + Edit + Write together
- Reason: Combination is powerful; should be justified by role
- Example:
  - ⚠️ "debugger with Bash, Edit, Write for a read-only analysis tool"
  - ✅ "deployment tool with Bash, Edit, Write for release workflow"

### 5. Model Field (AC-TPL-AGENTS-MODEL-POLICY)

**Error: Invalid model**
- Rule: `model` MUST be one of: `sonnet`, `opus`, `haiku`, `inherit`
- Example:
  - ✅ `model: sonnet`
  - ❌ `model: gpt-4`
  - ❌ `model: claude-3-opus`

**Warning: Expensive model (opus)**
- Rule: `opus` model triggers warning unless justified
- Reason: opus is expensive; encourage cost-conscious choices
- Example:
  - ⚠️ `model: opus` (triggers warning)
  - ✅ `model: opus` with context: "Complex reasoning required for security audits"

### 6. PermissionMode Field (AC-TPL-AGENTS-TOOLS-PERMISSION-SAFETY)

**Error: Invalid mode**
- Rule: `permissionMode` MUST be one of: `default`, `acceptEdits`, `bypassPermissions`, `plan`, `ignore`
- Example:
  - ✅ `permissionMode: default`
  - ❌ `permissionMode: full_access`
  - ❌ `permissionMode: sudo`

**Warning: High-risk mode (bypassPermissions)**
- Rule: `bypassPermissions` mode triggers warning
- Reason: Grants full write access; should be justified and reviewed
- Example:
  - ⚠️ `permissionMode: bypassPermissions` (triggers warning—expected for release agents)
  - ✅ "bypassPermissions (required for automated release tagging per ADR-0021)"

### 7. Skills Field (AC-TPL-AGENTS-SKILLS-REFERENCES)

**Error: Invalid format**
- Rule: `skills` MUST be a YAML list or comma-separated string
- Valid:
  ```yaml
  skills: governed-feature-dev, governed-maintenance
  ```
  or
  ```yaml
  skills:
    - governed-feature-dev
    - governed-maintenance
  ```
- Invalid: ❌ `skills: {name: governed-feature-dev}`

**Error: Skill does not exist**
- Rule: Each skill referenced in `skills` MUST have a corresponding directory in `.claude/skills/`
- Example:
  - Agent references: `skills: my-cool-skill`
  - Check: Does `.claude/skills/my-cool-skill/SKILL.md` exist?
  - ❌ Error if missing

### 8. Markdown Body (AC-TPL-AGENTS-LIFECYCLE-DOCS)

**Warning: No headings**
- Rule: Markdown body SHOULD contain at least one heading (`# `)
- Reason: Headings improve readability and structure
- Example:
  - ✅ Body starts with `# Agent Title`
  - ⚠️ Body is plain text without headings

---

## Phase 2: Security & Quality (PLANNED)

These rules are designed but not yet implemented.

### 2a. Secret Detection

**Error: Hardcoded secrets**
- Rule: Scan for patterns: `api_key`, `sk-`, `token`, `password`, `secret`, `credential`
- Reason: Prevent accidental leaks
- Severity: **ERROR** (blocks commit)
- Example:
  - ❌ `api_key=sk-abc123def456`
  - ❌ `password: "my-secret-pw"`

### 2b. Least-Privilege Analysis

**Warning: Read-only agent with write tools**
- Heuristic: If `description` contains "review", "analyze", "inspect", "audit", but `tools` includes `Edit` or `Write`
- Reason: Mismatch between intended use and capabilities
- Fix: Remove unnecessary write tools or clarify write purpose in description

**Warning: Dangerous tool combinations**
- Pattern: Bash (unscoped) + Write (to `/`) without clear justification
- Reason: Could modify critical files
- Fix: Use `Bash` scoped to specific directories, or justify in description

### 2c. Description Quality Deep-Dive

**Warning: Generic description**
- Pattern: Description is too generic ("Helper", "Agent", "Tool")
- Heuristic: Check for domain keywords and specific problem statements
- Fix: Be specific about problem space and trigger context

---

## Phase 3: Integration (PLANNED)

### Selftest Integration

- ✅ Step 3/10 in `cargo xtask selftest`
- ✅ Errors fail selftest; warnings are informational
- Skips if `.claude/agents/` directory doesn't exist

### CI Integration

- ✅ `.github/workflows/ci-agents.yml` (path-filtered job)
- ✅ Runs when `.claude/agents/**` or spec changes
- ✅ Blocks PR on errors
- ✅ Allows PR with warnings (but surfaces them for review)

### Precommit Integration

- ✅ Change-aware lint in `cargo xtask precommit`
- ✅ Only runs if `.claude/agents/**` changed
- ✅ Fast-path if no changes detected
- ✅ Blocks commit on errors

---

## Implementation Status

### ✅ Implemented (Phase 1)

| Rule | Category | Severity | Implemented | Tests |
|------|----------|----------|-------------|-------|
| Name kebab-case | Format | Error | ✅ | ✅ unit test |
| Name length | Format | Error | ✅ | ✅ unit test |
| Name matches file | Format | Error | ✅ | ✅ unit test |
| Description non-empty | Quality | Error | ✅ | ✅ unit test |
| Description ≤1024 chars | Quality | Error | ✅ | ✅ unit test |
| Description has WHEN | Quality | Warning | ✅ | ✅ unit test |
| Tools format | Format | Error | ✅ | ✅ |
| Tools not empty | Format | Error | ✅ | ✅ |
| Broad tool combo | Safety | Warning | ✅ | ✅ |
| Model is valid | Policy | Error | ✅ | ✅ unit test |
| Model is expensive | Policy | Warning | ✅ | ✅ |
| PermissionMode valid | Security | Error | ✅ | ✅ unit test |
| PermissionMode risky | Security | Warning | ✅ | ✅ |
| Skills reference exist | Traceability | Error | ✅ | ✅ unit test |
| YAML parse valid | Structure | Error | ✅ | ✅ |
| Frontmatter present | Structure | Error | ✅ | ✅ |
| Tabs in frontmatter | Structure | Error | ✅ | ✅ |
| Body has headings | Quality | Warning | ✅ | ✅ |

### ⏳ Planned (Phase 2)

| Rule | Category | Severity | Effort | Priority |
|------|----------|----------|--------|----------|
| Secret detection | Security | Error | Low | High |
| Read-only + write tools | Quality | Warning | Medium | High |
| Generic description | Quality | Warning | Medium | Medium |
| Dangerous tool combos | Security | Warning | Medium | Medium |

---

## Running agents-lint Locally

```bash
# Lint all agents
cargo run -p xtask -- agents-lint

# Within selftest
cargo run -p xtask -- selftest

# Within precommit (auto-runs if .claude/agents changed)
cargo run -p xtask -- precommit
```

## Interpreting Output

### Successful Run
```
[AGENT LINT] .claude/agents/code-reviewer.md ✓
[AGENT LINT] .claude/agents/test-runner.md ✓
```

### With Errors (Blocks Commit)
```
[AGENT LINT] .claude/agents/my-agent.md - ERRORS:
  ✗ frontmatter 'name' must match ^[a-z0-9-]{1,64}$ (got 'My-Agent').
  ✗ Skill 'nonexistent-skill' referenced in 'skills' does not exist in .claude/skills/

Agents governance check passed
error: agents governance check failed
```

### With Warnings (Allows Commit)
```
[AGENT LINT] .claude/agents/debugger.md - WARNINGS:
  ⚠ description could be more specific: try including 'when to use' or trigger context.
  ⚠ broad tool set detected (Bash + Edit + Write); ensure this is justified by agent role.
  ⚠ expensive model 'opus' specified; ensure this is justified by agent complexity.

[AGENT LINT] .claude/agents/debugger.md ✓
```

---

## FAQ

**Q: Why does agents-lint error on "name must match filename"?**
A: Keeps agents discoverable and prevents confusion. A single source of truth (filename) matches frontmatter.

**Q: Can I use UPPERCASE letters or underscores in agent names?**
A: No. Kebab-case (lowercase + hyphens only) is the standard. It matches Skills naming and CLI conventions.

**Q: What if I want to reference a Skill that doesn't exist yet?**
A: Create the Skill first (and get it governed), then reference it. This ensures both are auditable.

**Q: Is opus model always an error?**
A: No, it's a warning. If your agent needs complex reasoning, opus is justified. Document the reason in description or comment.

**Q: Can I have an agent with no tools field?**
A: Yes. If `tools` is omitted, the agent inherits all tools. Explicitly list `tools` for least-privilege.

---

## Related Documents

- **AGENTS_GOVERNANCE.md**: Governance principles and lifecycle
- **AGENTS_TEMPLATE.md**: Copy-paste template for creating new agents
- **AGENTS_CONTRIBUTING.md**: Quick developer workflow
- **Spec**: REQ-TPL-AGENTS-GOVERNANCE and AC-*

---

**Version**: 1.0.0 (aligned with template v3.3.x)
