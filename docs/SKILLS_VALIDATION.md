# Skills Validation Rules & Implementation Guide

**Audience**: Infrastructure engineers, tool maintainers, CI/CD operators
**Status**: Governs AC-TPL-SKILLS-NAME-FORMAT, AC-TPL-SKILLS-DESCRIPTION-QUALITY, AC-TPL-SKILLS-ALLOWED-TOOLS-SAFETY, AC-TPL-SKILLS-FLOW-MAPPING
**Related**: ADR-0020, SKILLS_GOVERNANCE.md

---

## Overview

Skills validation is enforced by:

1. **Local**: `cargo xtask skills-lint` (pre-commit, developer feedback)
2. **CI**: xtask in GitHub Actions (blocks PR)
3. **Selftest**: `cargo xtask selftest` step 4 (governance gate)

This document specifies the exact rules each validation enforces.

---

## Rule Categories

| Category | Tool | Enforcement | Severity |
|----------|------|------------|----------|
| **Syntax** | skills-lint | Local + CI | ERROR (blocks) |
| **Name Format** | skills-lint | Local + CI + selftest | ERROR (blocks) |
| **Description Quality** | skills-lint | Local + CI + selftest | WARNING (report) or ERROR |
| **Allowed-Tools Safety** | skills-lint | Local + CI + selftest | WARNING (report) |
| **Flow References** | skills-lint | Local + CI + selftest | WARNING (report) |
| **File Integrity** | skills-lint | Local + CI | ERROR (blocks) |

---

## 1. Syntax Validation

### What it checks

- YAML frontmatter is valid (parseable)
- Required fields present: `name`, `description`
- Optional field valid if present: `allowed-tools`
- Markdown body exists after `---` closing

### Rules

1. **Frontmatter must open with `---`** on line 1
2. **Frontmatter must close with `---`** before Markdown body
3. **No tab characters** in YAML (spaces only)
4. **`name` field**: Required, string, non-empty
5. **`description` field**: Required, string, non-empty
6. **`allowed-tools` field**: Optional, comma-separated string

### Examples

✅ **Valid**:

```yaml
---
name: my-skill
description: >
  This is a valid description.
allowed-tools: Read, Grep
---

# My Skill
...
```

❌ **Invalid (missing closing `---`)**:

```yaml
---
name: my-skill
description: "Description"

# My Skill  ← Should have --- before this
```

❌ **Invalid (tabs in YAML)**:

```yaml
---
name: my-skill
	description: "Description"  ← TAB character
---
```

### Error Message

```
ERROR: SKILL.md syntax invalid
  Line 1: Expected opening ---
  Line 15: Expected closing --- before Markdown body
  Validation: YAML parse error at line 12
```

---

## 2. Name Format Validation

### What it checks

- Kebab-case format (lowercase + hyphens only)
- Length: 1-64 characters
- Uniqueness within project
- No reserved names

### Rules

1. **Kebab-case**: Only `[a-z0-9]` and `-` (hyphens)
   - ✅ `my-skill`, `governed-feature-dev`, `skill-1`
   - ❌ `MySkill` (uppercase), `my_skill` (underscore), `my.skill` (dot)

2. **Length**: 1-64 characters (inclusive)
   - ✅ `a` (1 char), `my-really-long-skill-name-for-testing` (39 chars)
   - ❌ `my-incredibly-long-skill-name-that-exceeds-the-maximum-allowed-length` (71 chars)

3. **Uniqueness**: No other Skill in project with same `name`
   - ✅ First Skill named `foo-bar`
   - ❌ Second Skill also named `foo-bar`

4. **Directory name must match**:
   - `.claude/skills/my-skill/SKILL.md` has `name: my-skill` ✅
   - `.claude/skills/my-skill/SKILL.md` has `name: other-skill` ❌

### Anti-Pattern Detection

Warn if name **suggests single-command wrapping**:

- ❌ `skill-check` (suggests wraps `xtask check`)
- ❌ `skill-bdd` (suggests wraps `xtask bdd`)
- ❌ `skill-bundle` (suggests wraps `xtask bundle`)

**Pattern**: If name starts with `skill-` followed by single xtask command, warn:

```
WARNING: Name 'skill-check' suggests single-command Skill.
Anti-pattern: Skills should wrap workflows, not individual commands.
Use a workflow-based name instead (e.g., 'governed-feature-dev').
```

### Error Message

```
ERROR: Name validation failed for 'mySkill'
  - Must be kebab-case (only a-z, 0-9, hyphen)
  - Got: 'mySkill' (uppercase present)

ERROR: Name validation failed for 'my-skill-that-exceeds-the-maximum-length-of-64-characters'
  - Must be ≤64 characters
  - Got: 71 characters

ERROR: Name not unique
  - 'my-skill' is already defined in .claude/skills/my-skill/SKILL.md
  - Cannot have duplicate names in project

WARNING: Anti-pattern detected for 'skill-check'
  - Name suggests wrapping single command, not workflow
  - Recommendation: Use workflow-based name (e.g., 'governance-debug')
```

---

## 3. Description Quality Validation

### What it checks

- Includes both WHAT (capability) and WHEN (triggers)
- Third-person voice (not first-person)
- Max 1024 characters
- Contains concrete trigger keywords

### Rules

#### 3.1 WHAT + WHEN Requirement

Description must answer both questions:

- **WHAT**: What capability does this Skill provide?
- **WHEN**: When should Claude use this Skill?

**Analysis**:

1. Tokenize description into sentences
2. First 1-3 sentences should define WHAT
3. Remaining sentences should define WHEN with triggers:
   - File types: `.toml`, `.yaml`, `.md`, `PDF`, `Excel`
   - User phrases: `implement`, `add AC`, `debug`, `release`
   - Task types/statuses: `status=Todo`, `type=feature`
   - Domains: `AC-first`, `security`, `release`

**Examples**:

✅ **Good (clear WHAT + WHEN)**:

```
AC-first feature development workflow for implementing Requirements and
Acceptance Criteria. Use when implementing tasks tagged 'feature', when
user says 'implement AC', or when working with specs/spec_ledger.yaml.
```

- WHAT: "AC-first feature development workflow for implementing Requirements and ACs"
- WHEN: "implementing tasks tagged 'feature'", "user says 'implement AC'", "working with specs/spec_ledger.yaml"

✅ **Good (another example)**:

```
Platform governance troubleshooting Skill. Use when selftest fails,
when user reports 'governance broken', or when policy violations detected.
```

- WHAT: "Platform governance troubleshooting Skill"
- WHEN: "selftest fails", "user reports governance broken", "policy violations detected"

❌ **Bad (missing WHEN)**:

```
Helps with development
```

- No clear WHAT or WHEN

❌ **Bad (missing specific triggers)**:

```
Workflow for stuff
```

- WHAT is vague ("stuff")
- WHEN missing entirely

#### 3.2 Third-Person Voice

Description must use third-person or imperative, NOT first-person:

- ✅ "This Skill implements..." or "Use when..."
- ❌ "I can implement..." or "I help with..."

**Detection**: Simple pattern matching:
- Flag if contains: `I can`, `I will`, `I help`, `I do`, `I provide`
- Suggest rewrite in third-person

#### 3.3 Character Limit

- Max 1024 characters (including spaces, newlines)
- **Rationale**: Fits in frontmatter + system prompt efficiently

**Detection**: Count characters, compare to limit

#### 3.4 Concrete Triggers

Description should mention **specific** triggers, not generic ones:

- ✅ `"when user says 'implement AC'"`
- ❌ `"when you need help with features"`

**Detection**: Look for presence of:
- File type keywords: `feature`, `API`, `docs`, `test`, `yaml`, `toml`, `rust`
- Action keywords: `implement`, `debug`, `fix`, `add`, `create`, `release`
- Status keywords: `Todo`, `InProgress`, `Done`, `failing`
- Domain keywords: `governance`, `policy`, `security`, `release`

If description contains <2 concrete keywords, warn:

```
WARNING: Description lacks concrete triggers
  Expected: file types, user phrases, status names, etc.
  Examples: "implement AC", "failing tests", ".toml files"
  Consider adding specific user scenarios or file types.
```

### Error Messages

```
ERROR: Description validation failed
  - Missing WHAT (capability): unclear what Skill does
  - Missing WHEN (triggers): no indication when to use
  Suggestion: "AC-first feature workflow (WHAT). Use when implementing ACs or tasks tagged 'feature' (WHEN)."

WARNING: Description uses first-person voice
  Found: "I can implement features"
  Rewrite in third-person: "Implements features according to..."

ERROR: Description exceeds max length
  - Max: 1024 characters
  - Got: 1247 characters
  - Recommendation: Move examples/details to separate .md file

WARNING: Description lacks concrete triggers
  - Add specific user phrases, file types, or task states
  - Examples: "when implementing ACs", ".yaml files", "status=Todo"
```

---

## 4. Allowed-Tools Safety Validation

### What it checks

- Least-privilege principle (only necessary tools)
- No dangerous tool combinations
- No hardcoded secrets
- Proper scoping for dangerous tools (e.g., `Bash`)

### Rules

#### 4.1 Least Privilege

Each tool in `allowed-tools` must be necessary for the Skill.

**Default allowed tools**:
- `Read`: Reading files (always safe)
- `Grep`: Searching files (safe)
- `Glob`: Finding files (safe)
- `Edit`: Modifying files (should be justified)
- `Write`: Creating files (should be justified)
- `Bash`: Executing shell commands (dangerous, requires justification)
- `Task`: Creating sub-agents (use with caution)
- `WebSearch`: Network access (document why needed)

**Validation**:

1. Read-only Skill should NOT include: `Write`, `Edit`, `Bash`, `Task`
   ```yaml
   # ❌ Bad: read-only Skill with write tools
   allowed-tools: Read, Grep, Write
   # ✅ Good: minimal tools
   allowed-tools: Read, Grep, Glob
   ```

2. File-writing Skill should NOT include unscoped `Bash`
   ```yaml
   # ❌ Bad: gives full shell access
   allowed-tools: Write, Bash
   # ✅ Good: scoped to git operations
   allowed-tools: Write, Bash(git status:*), Bash(git diff:*)
   ```

3. Network-capable tools (`WebSearch`, `WebFetch`) need justification
   ```yaml
   allowed-tools: WebSearch  # ⚠ Warning: document why network access needed
   ```

#### 4.2 Bash Scoping

If `Bash` used, it should be **scoped to specific commands**:

Syntax: `Bash(command_pattern:*)`

Examples:
- ✅ `Bash(git status:*)` — only git status
- ✅ `Bash(cargo check:*)` — only cargo check
- ✅ `Bash(npm run:*::*)` — only npm run commands
- ❌ `Bash` — unscoped, any command allowed

**Detection**: If `Bash` present without scoping, warn:

```
WARNING: Unscoped Bash in allowed-tools
  Tool: Bash (unscoped)
  Recommendation: Scope to specific commands (e.g., Bash(git status:*))
  If unscoped Bash is necessary, document justification in SKILL.md comments.
```

#### 4.3 No Hardcoded Secrets

SKILL.md and supporting files must NOT contain:
- API keys
- OAuth tokens
- Passwords
- Private credentials
- Secrets manager credentials

**Detection**: Scan for patterns:
- `api_key`, `apiKey`, `API_KEY`
- `token`, `TOKEN`, `oauth`
- `password`, `PASSWORD`, `secret`, `SECRET`
- File paths like `.env`, `credentials.json`, `secrets.yaml`

**Error**:

```
ERROR: Hardcoded secret detected in SKILL.md
  Line 45: contains pattern 'api_key'
  Recommendation: Use environment variables or external secret store
```

### Error/Warning Messages

```
WARNING: Least-privilege violation
  Skill appears read-only (description: "reads and analyzes...")
  But allowed-tools includes: Write, Edit
  Recommendation: Remove Write/Edit if not needed, or update description to clarify.

WARNING: Unscoped Bash in allowed-tools
  Skill: my-skill
  Tool: Bash (unscoped)
  Risk: Can execute any shell command
  Recommendation: Scope to specific commands (e.g., Bash(git status:*))
  Or justify in SKILL.md comments if unscoped access necessary.

ERROR: Hardcoded secret in SKILL.md
  Line 48: text contains 'api_key = "sk-..."'
  Recommendation: Use environment variables or move to external secret store

ERROR: Dangerous tool combination
  Tools: Bash (unscoped), Write, WebSearch
  Risk: Unscoped shell + write + network = high risk
  Recommendation: Scope Bash, justify combination, document in SKILL.md
```

---

## 5. Flow Reference Validation

### What it checks

- Description or body references at least one devex_flows entry or xtask command
- Anti-pattern detection (single-command Skill)

### Rules

#### 5.1 Flow/Command Reference Requirement

Description or Workflow section must mention:

1. **Flow ID** from `specs/devex_flows.yaml`
   - Examples: `ac_first`, `maintenance`, `release`, `onboarding`, `governance_debug`
   - Format: mentions flow name in description or references it

2. **OR at least one xtask command** that Skill uses
   - Examples: `xtask ac-new`, `xtask bundle`, `xtask selftest`, `xtask skills-lint`

**Detection**:

1. Extract flow IDs from `specs/devex_flows.yaml`
2. Check if description/body mentions any flow ID
3. If not, check if mentions any xtask commands (`cargo xtask <cmd>`)
4. If neither, fail validation

#### 5.2 Anti-Pattern Detection

Detect and warn about single-command Skills:

**Pattern**: If Skill name + description suggests wrapping single command

- ❌ Name starts with `skill-` + xtask command name
- ❌ Description mentions only one xtask command
- ❌ No workflow sequence described

**Examples**:

```
name: skill-check
description: "Runs the check command"
  ↑ Suggests wraps single command

name: governed-feature-dev
description: "AC-first feature workflow using ac-new, bundle, bdd, selftest"
  ↑ Clearly wraps multi-step workflow
```

**Warning**:

```
WARNING: Anti-pattern detected
  Skill 'skill-check' appears to wrap single command 'xtask check'
  Problem: Creates too many Skills (one per command), unusable
  Solution: Group into workflow-level Skills (e.g., 'governed-feature-dev')
  Reference: See SKILLS_GOVERNANCE.md anti-patterns section
```

### Error/Warning Messages

```
ERROR: No flow/command reference
  Description must reference a devex_flows entry (ac_first, maintenance, etc.)
  OR mention at least one xtask command used by this Skill
  Example: "Uses ac_first flow (ac-new, bundle, bdd, selftest)"

WARNING: Anti-pattern detected
  Skill 'skill-bdd' suggests wrapping single command 'xtask bdd'
  Anti-pattern: One Skill per command → skill explosion
  Solution: Wrap entire workflow (ac_first flow) instead
  Suggestion: Rename to 'governed-feature-dev'
```

---

## 6. File Integrity Validation

### What it checks

- SKILL.md exists at correct path
- Directory matches name field
- Referenced supporting files exist
- Links are valid (relative paths)

### Rules

1. **File location**
   - Must be at `.claude/skills/<name>/SKILL.md`
   - `<name>` must match `name:` field in frontmatter

2. **Referenced files**
   - Any file mentioned in body (e.g., `examples.md`, `scripts/helper.sh`) must exist
   - Relative paths from SKILL.md directory

3. **Markdown links**
   - Links to other .md files must be valid (file exists)
   - HTTP/HTTPS links are warnings (external, might break)

### Examples

✅ **Valid**:

```
.claude/skills/my-skill/
├── SKILL.md           # Exists
├── examples.md        # Referenced in SKILL.md, exists
└── reference.md       # Referenced, exists
```

❌ **Invalid**:

```
.claude/skills/my-skill/
└── SKILL.md           # References examples.md (doesn't exist)
```

### Error Messages

```
ERROR: SKILL.md not found at expected location
  Expected: .claude/skills/my-skill/SKILL.md
  Got: .claude/skills/my-skill/skill.md (wrong name/case)

ERROR: Directory/name mismatch
  Directory: .claude/skills/my-skill/
  name field: other-skill
  These must match

ERROR: Referenced file not found
  SKILL.md mentions: examples.md
  Expected at: .claude/skills/my-skill/examples.md
  Not found

WARNING: External link in SKILL.md
  Link: https://example.com/docs
  Risk: External links can break
  Recommendation: Consider inlining or mirroring content
```

---

## 7. Integration: How Validation Runs

### 7.1 Development: `cargo xtask skills-lint`

```bash
$ cargo xtask skills-lint

Linting .claude/skills/...

Validating .claude/skills/governed-feature-dev/SKILL.md
  ✓ Syntax: Valid YAML frontmatter
  ✓ Name: 'governed-feature-dev' (kebab-case, 20 chars)
  ✓ Description: 187 chars, includes WHAT + WHEN
  ✓ Allowed-tools: Valid (Read, Grep, Glob, Edit, Write, Bash)
  ✓ Flow references: Mentions 'ac_first' flow
  ✓ File integrity: examples.md exists

Validating .claude/skills/mycorp-workflow/SKILL.md
  ✗ Name: 'myCorpWorkflow' must be kebab-case
  ✗ Description: 52 chars, missing WHEN triggers
  ⚠ Allowed-tools: Unscoped Bash without justification

Lint report:
  Passed: 5/7 Skills
  Errors: 2 (name, description)
  Warnings: 5 (allowed-tools, flow refs, etc.)

Result: FAILED (errors block)
```

### 7.2 Pre-commit Hook (Optional)

```bash
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: skills-lint
        name: Lint Agent Skills
        entry: cargo run -p xtask -- skills-lint
        language: system
        files: ^\.claude/skills/
        pass_filenames: false
```

Prevents commit if skills-lint fails.

### 7.3 CI: GitHub Actions

```yaml
# .github/workflows/lint.yml
- name: Lint Agent Skills
  run: cargo run -p xtask -- skills-lint
  # Fails if any errors detected
```

### 7.4 Selftest: Step 4

```bash
$ cargo xtask selftest

[Step 4] Skills Governance Validation
  Running: skills-lint
  [✓] All Skills pass governance rules

[Step 5] Policy Tests
...
```

---

## 8. Implementation Checklist (for Tool Maintainers)

To implement `skills-lint` command in xtask:

### Phase 1: Syntax + Name (MVP)

- [ ] Parse YAML frontmatter from SKILL.md
- [ ] Validate `name` field (kebab-case, length, uniqueness)
- [ ] Report errors blocking commits
- [ ] Test with existing 5 Skills

### Phase 2: Description + Tools

- [ ] Validate description (WHAT + WHEN detection)
- [ ] Warn on vague descriptions
- [ ] Validate allowed-tools (least-privilege)
- [ ] Detect hardcoded secrets pattern

### Phase 3: Flow References + Anti-Patterns

- [ ] Parse devex_flows.yaml
- [ ] Detect if Skill references a flow
- [ ] Anti-pattern detection (single-command Skills)
- [ ] Suggest alternatives

### Phase 4: File Integrity

- [ ] Verify SKILL.md exists at correct path
- [ ] Check referenced files exist
- [ ] Validate relative links
- [ ] Report missing files

### Phase 5: Integration

- [ ] Add to `selftest` step 4
- [ ] Wire into pre-commit hooks
- [ ] Add to CI workflows
- [ ] Document in CLAUDE.md

---

## 9. Test Cases (for validation testing)

### Syntax Tests

```yaml
# test_skill_syntax_valid.yaml
name: my-skill
description: "Valid description"

# test_skill_syntax_missing_name.yaml
# (missing name field → ERROR)

# test_skill_syntax_tabs.yaml
---
name: my-skill
	description: "Has tab" → ERROR
```

### Name Tests

```
test_name_kebab_case_valid: my-skill ✓
test_name_kebab_case_invalid: MySkill ✗
test_name_kebab_case_invalid: my_skill ✗
test_name_length_64: [a-z0-9-]{64} ✓
test_name_length_65: [a-z0-9-]{65} ✗
test_name_unique: first-skill, first-skill ✗ (duplicate)
test_name_antipattern_skill_check: warns
test_name_dir_match: .claude/skills/my-skill + name: my-skill ✓
```

### Description Tests

```
test_desc_has_what_when: "Workflow (WHAT). Use when implementing (WHEN)." ✓
test_desc_missing_what: "Use when doing things" ✗
test_desc_missing_when: "This is a workflow" ✗
test_desc_first_person: "I can implement" ✗ (flags voice)
test_desc_length_1024: 1024 chars ✓
test_desc_length_1025: 1025 chars ✗
test_desc_triggers: mentions "AC", "feature", "status" ✓
test_desc_no_triggers: generic words only ⚠ (warn)
```

### Allowed-Tools Tests

```
test_tools_read_only: Read, Grep, Glob ✓
test_tools_read_with_write: Read, Grep, Write ✓ (explicit)
test_tools_read_with_bash: Read, Bash ✗ (unscoped Bash in read-only)
test_tools_bash_scoped: Bash(git status:*) ✓
test_tools_secret: "api_key = sk-..." ✗ (secret detected)
test_tools_kitchen_sink: All tools ⚠ (warn, justify)
```

### Flow Tests

```
test_flow_reference_valid: description mentions "ac_first" ✓
test_flow_reference_valid: description mentions "xtask selftest" ✓
test_flow_reference_missing: no flow/command mentioned ✗
test_antipattern_skill_check: name suggests single-command ⚠
test_antipattern_workflow_skill: name suggests workflow ✓
```

### File Integrity Tests

```
test_file_exists: .claude/skills/my-skill/SKILL.md ✓
test_file_wrong_path: .claude/skills/my-skill/skill.md ✗
test_file_referenced_exists: SKILL.md → examples.md ✓
test_file_referenced_missing: SKILL.md → examples.md (missing) ✗
test_link_relative_valid: [examples](examples.md) ✓
test_link_relative_invalid: [examples](examples.md) (file missing) ✗
```

---

## References

- **Governance**: [SKILLS_GOVERNANCE.md](SKILLS_GOVERNANCE.md)
- **Template**: [SKILLS_TEMPLATE.md](SKILLS_TEMPLATE.md)
- **ADR**: [ADR-0020](adr/0020-claude-code-skills-governance.md)
- **Spec**: `specs/spec_ledger.yaml` (REQ-TPL-SKILLS-GOVERNANCE)
