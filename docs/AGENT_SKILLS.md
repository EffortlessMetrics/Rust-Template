# Agent Skills Guide: Rust-as-Spec Platform

**Audience:** Developers and agents authoring Skills for this repository
**Scope:** Rust-as-Spec specific guidance (supplements [Anthropic's general Skills documentation](https://docs.anthropic.com/claude/docs/agent-skills))

---

## Purpose

This guide describes how to create and govern Agent Skills specifically for the Rust-as-Spec platform cell.

Unlike general-purpose Skills, Skills in this repository must align with:
- **DevEx flows** (`specs/devex_flows.yaml`) - Our defined workflows
- **xtask commands** - Platform tooling primitives
- **`/platform/*` APIs** - Governance introspection and task management
- **Task board** (`/ui/tasks`) - Work queue visualization

**Key principle:** Skills should wrap **governed workflows**, not individual commands. Think "how to do AC-first feature development" (a Skill), not "how to run xtask check" (too granular).

---

## Skill Categories in This Repo

We maintain **5 core Skills** that map to major workflows:

| Skill | Flow | Uses | When |
|-------|------|------|------|
| `bootstrap-dev-env` | Onboarding | `dev-up`, `status`, `/platform/status` | First-time setup, broken environment |
| `governed-feature-dev` | AC-first development | `ac-new`, `bundle`, `bdd`, `selftest`, task API | Implementing features/ACs |
| `governed-maintenance` | Platform upkeep | `policy-test`, `audit`, `check`, `selftest` | Fixing governance, dependencies, docs |
| `governed-release` | Version management | `release-prepare`, `release-verify`, tagging | Cutting releases |
| `governed-governance-debug` | Troubleshooting | `selftest` summary, graph, policies | Diagnosing selftest failures |

### Why Only 5 Skills?

**Anti-pattern:** One Skill per xtask command (e.g., "skill-check", "skill-bdd", "skill-bundle")
**Problem:** Creates 20+ near-identical Skills; Claude can't choose effectively

**Correct approach:** Skills = workflows = `devex_flows.yaml` entries
- Each Skill encapsulates a **complete governed process**
- xtask commands and APIs are **tools** the Skill uses, not Skills themselves

---

## Skill Authoring Pattern

### Structure

```
.claude/skills/
├── bootstrap-dev-env/
│   └── SKILL.md
├── governed-feature-dev/
│   ├── SKILL.md
│   ├── examples.md  (optional)
│   └── reference.md (optional)
└── governed-maintenance/
    └── SKILL.md
```

### SKILL.md Template

```yaml
---
name: skill-name-here
description: >
  Brief summary of what this Skill does and when to use it. Include specific
  triggers (e.g., "Use when implementing ACs", "Use when selftest fails").
  Max 1024 characters. Be specific about the workflow this Skill wraps.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash  # Optional: restrict tools
---

# Skill Name

## When to Use

Clear, specific criteria for when Claude should invoke this Skill:
- User says "implement feature X"
- Task status is Todo and type is "feature"
- etc.

## Prerequisites

- Platform must be running (check via `GET /platform/status`)
- Specs must be valid (no syntax errors in `spec_ledger.yaml`)
- etc.

## Workflow

Step-by-step process, referencing `devex_flows.yaml`:

1. **Discover work:**
   ```bash
   curl http://localhost:3000/platform/agent/hints | jq
   ```
   Pick a task from `tasks` array.

2. **Claim task:**
   ```bash
   curl -X POST http://localhost:3000/platform/tasks/{id}/status \
     -H "Content-Type: application/json" \
     -d '{"status": "InProgress"}'
   ```

3. **Follow AC-first flow** (from `devex_flows.yaml`):
   - Run `cargo xtask ac-new` if needed
   - Run `cargo xtask bundle implement_ac` to get context
   - Edit code and tests
   - Run `cargo xtask bdd`
   - Run `XTASK_LOW_RESOURCES=1 cargo xtask selftest`

4. **Mark done:**
   ```bash
   curl -X POST http://localhost:3000/platform/tasks/{id}/status \
     -H "Content-Type: application/json" \
     -d '{"status": "Done"}'
   ```

## Exit Criteria

How to know the workflow succeeded:
- Selftest passes (all 7 steps)
- Task status = Done
- No policy violations
- etc.

## Error Handling

Common failure modes and recovery:
- **Selftest fails:** Use `governed-governance-debug` Skill
- **BDD fails:** Check step definitions in `crates/acceptance/src/steps/`
- **Platform not running:** Run `cargo run -p app-http` first

## Examples

Concrete scenarios showing this Skill in action.

## References

- `specs/devex_flows.yaml` - Canonical flow definition
- `docs/AGENT_GUIDE.md` - Operational guide
- `docs/reference/xtask-commands.md` - Command reference
```

### Key Elements

1. **`description` field:** Must be **specific**
   - ❌ "Helps with features"
   - ✅ "AC-first feature workflow for implementing Requirements and Acceptance Criteria. Use when implementing tasks tagged `feature` or when user mentions 'implement AC'."

2. **`allowed-tools` field:** Optional but recommended
   - Restricts which tools Claude can use within this Skill
   - Useful for read-only Skills or security-sensitive workflows
   - Example: `allowed-tools: Read, Grep, Glob` for read-only governance debug

3. **Workflow steps:** Reference actual commands and APIs
   - Use exact `cargo xtask` commands
   - Use exact `/platform/*` API endpoints
   - Link to `devex_flows.yaml` entries

4. **Exit criteria:** Make success/failure explicit
   - "Selftest passes" not "code looks good"
   - "Task status updated to Done" not "work is finished"

---

## Recommended Skills (Detailed)

### 1. `bootstrap-dev-env`

**Purpose:** One-command environment setup and health check

**Flow:** `onboarding` (from `devex_flows.yaml`)

**Uses:**
- `cargo xtask dev-up`
- `cargo xtask status`
- `GET /platform/status`

**When:**
- First time in the repo
- After major environment changes (tool updates, config changes)
- Environment appears broken

**Key outputs:**
- Confirms dependencies installed (cargo, rustc, conftest)
- Validates platform is running
- Runs core checks and BDD
- Shows next steps

**Exit criteria:**
- `dev-up` exits 0
- `/platform/status` returns 200
- All URLs in output are reachable

### 2. `governed-feature-dev`

**Purpose:** AC-first feature development workflow

**Flow:** `ac_first` (from `devex_flows.yaml`)

**Uses:**
- `GET /platform/agent/hints` - Discover work
- `POST /platform/tasks/{id}/status` - Claim and complete tasks
- `cargo xtask ac-new` - Create new ACs
- `cargo xtask bundle implement_ac` - Get focused context
- `cargo xtask bdd` - Run acceptance tests
- `XTASK_LOW_RESOURCES=1 cargo xtask selftest` - Validate changes

**When:**
- Implementing new features
- Adding new ACs
- Working on tasks with status=Todo

**Workflow:**
1. Call `/platform/agent/hints` → get prioritized tasks
2. Pick task, call `POST /tasks/{id}/status` → InProgress
3. Create/update AC via `ac-new` if needed
4. Get context via `bundle implement_ac`
5. Implement code + tests
6. Run `bdd` to verify scenarios
7. Run `selftest` to ensure governance
8. Update task → Done

**Exit criteria:**
- Selftest passes (7/7 steps)
- Task status = Done
- No new policy violations

**Error recovery:**
- Selftest fails → Use `governed-governance-debug`
- BDD fails → Check step definitions, verify feature file syntax
- Can't find AC → Create with `ac-new`

### 3. `governed-maintenance`

**Purpose:** Platform upkeep, dependency updates, doc fixes

**Flow:** `maintenance` (from `devex_flows.yaml`)

**Uses:**
- `cargo xtask policy-test` - Validate policies
- `cargo xtask audit` - Check dependencies
- `cargo xtask check` - Core quality checks
- `cargo xtask selftest` - Full validation

**When:**
- Fixing policy violations
- Updating dependencies
- Fixing docs drift
- Resolving friction log entries

**Workflow:**
1. Identify issue (policy failure, outdated dep, doc drift)
2. Fix the issue (edit `.rego`, update `Cargo.toml`, fix docs)
3. Run `policy-test` if policy-related
4. Run `check` to validate code quality
5. Run `selftest` to ensure no regressions
6. Update friction log if relevant

**Exit criteria:**
- Selftest passes
- Specific issue resolved (policy passes, dep updated, doc fixed)
- Friction log entry marked resolved (if applicable)

### 4. `governed-release`

**Purpose:** Version management and release tagging

**Flow:** `release` (from `devex_flows.yaml`)

**Uses:**
- `cargo xtask release-prepare` - Bump versions, update changelogs
- `cargo xtask release-verify` - Validate release readiness
- `cargo xtask release-bundle X.Y.Z` - Generate evidence (v3.1+)
- Git tagging commands

**When:**
- Cutting a new version
- User explicitly requests "cut a release"
- Preparing for production deployment

**Workflow:**
1. Run `release-prepare` → bumps versions
2. Run `selftest` → ensure everything passes
3. Run `release-verify` → checks readiness
4. (v3.1+) Run `release-bundle X.Y.Z` → generate evidence file
5. Create git tag
6. Push tag to trigger CI

**Exit criteria:**
- `release-verify` passes
- Git tag created
- (v3.1+) Evidence file in `release_evidence/vX.Y.Z.md`

**Important:**
- **Never force-push to main/master**
- **Always ask user before tagging** (high-risk operation)

### 5. `governed-governance-debug`

**Purpose:** Diagnose and fix selftest failures

**Flow:** Ad-hoc troubleshooting

**Uses:**
- `cargo xtask selftest -v` - Verbose selftest output
- `GET /platform/graph` - Inspect governance graph
- `cargo xtask policy-test` - Isolate policy issues

**When:**
- Selftest fails
- User reports "governance broken"
- Policy violations detected

**Workflow:**
1. Run `selftest -v` to get detailed output
2. Identify which step failed (1-7)
3. Run isolated command for that step:
   - Step 1: `xtask check`
   - Step 2: `xtask bdd`
   - Step 3: `xtask ac-status`
   - Step 4: `xtask bundle implement_ac`
   - Step 5: `xtask policy-test`
   - Step 6: Check `devex_flows.yaml` for missing commands
   - Step 7: Check `/platform/graph` for invariant violations
4. Fix the specific issue
5. Re-run `selftest` to confirm

**Exit criteria:**
- Selftest passes all 7 steps
- Root cause identified and documented

---

## Skill Governance

**Skills are governed artifacts**, not ad-hoc scripts.

### Creating a New Skill

1. **Create a REQ/AC** in `spec_ledger.yaml`:
   ```yaml
   - id: REQ-TPL-SKILLS-EXAMPLE
     title: "Example workflow Skill"
     acceptance_criteria:
       - id: AC-TPL-SKILLS-EX-001
         text: "Skill SKILL.md exists at .claude/skills/example/"
   ```

2. **Create a Task** in `tasks.yaml`:
   ```yaml
   - id: TASK-TPL-SKILLS-EX-001
     title: "Implement example workflow Skill"
     requirement: REQ-TPL-SKILLS-EXAMPLE
     acs: [AC-TPL-SKILLS-EX-001]
     status: Todo
   ```

3. **Follow the workflow:**
   - Use `governed-feature-dev` Skill to implement the new Skill
   - Write SKILL.md following the template above
   - Add examples.md if needed
   - Run selftest to ensure no regressions

4. **Document in this file:**
   - Add entry to the "Recommended Skills" table
   - Add detailed section if it's a major workflow

### Modifying an Existing Skill

1. Update SKILL.md
2. Ensure `devex_flows.yaml` still matches (or update flow)
3. Run `selftest` to catch breaking changes
4. Log friction if the Skill needed changes due to kernel changes

### Removing a Skill

1. Mark the corresponding REQ as deprecated
2. Remove or archive SKILL.md
3. Update this document to remove references
4. Run `selftest` to ensure no orphaned references

---

## Common Anti-Patterns

### ❌ One Skill Per Command

**Wrong:**
- `skill-check` → wraps `xtask check`
- `skill-bdd` → wraps `xtask bdd`
- `skill-bundle` → wraps `xtask bundle`

**Problem:** 20+ Skills, Claude can't choose

**Right:** Skills wrap **workflows** that use multiple commands

### ❌ Vague Descriptions

**Wrong:**
```yaml
description: Helps with development
```

**Right:**
```yaml
description: >
  AC-first feature workflow for implementing Requirements and Acceptance Criteria.
  Use when implementing tasks tagged `feature`, when user says "implement AC",
  or when working with specs/spec_ledger.yaml.
```

### ❌ Ad-Hoc Skills (No Governance)

**Wrong:** Create `.claude/skills/quick-fix/SKILL.md` without REQ/AC/Task

**Right:** Follow the governance process (REQ → AC → Task → Skill)

### ❌ Skills That Don't Use Platform APIs

**Wrong:** Skill that manually parses `tasks.yaml` instead of calling `/platform/tasks`

**Right:** Use `/platform/*` APIs; they're the governed interface

---

## Example: `governed-feature-dev` Skill (Full)

See `.claude/skills/governed-feature-dev/SKILL.md` for the complete, working example.

**Key excerpts:**

```yaml
---
name: governed-feature-dev
description: >
  AC-first feature development workflow for the Rust-as-Spec platform cell.
  Use when implementing new features, adding ACs, or working on tasks with
  status=Todo. Follows the ac_first flow from devex_flows.yaml and uses
  xtask + /platform APIs for governance.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Governed Feature Development

## When to Use

- Implementing a new feature driven by Acceptance Criteria
- Adding or updating ACs in `specs/spec_ledger.yaml`
- Working on tasks in Todo or InProgress with type=feature

## Workflow

1. **Discover work:**
   ```bash
   curl http://localhost:3000/platform/agent/hints | jq '.tasks[] | select(.status == "Todo")'
   ```

2. **Claim task:**
   ```bash
   TASK_ID="TASK-TPL-XXX-001"
   curl -X POST "http://localhost:3000/platform/tasks/${TASK_ID}/status" \
     -H "Content-Type: application/json" \
     -d '{"status": "InProgress"}'
   ```

3. **AC-first implementation:**
   - Create AC if needed: `cargo xtask ac-new AC-ID "description" ...`
   - Get context: `cargo xtask bundle implement_ac`
   - Implement code and tests
   - Run BDD: `cargo xtask bdd`

4. **Governance validation:**
   ```bash
   XTASK_LOW_RESOURCES=1 cargo xtask selftest
   ```

5. **Mark done:**
   ```bash
   curl -X POST "http://localhost:3000/platform/tasks/${TASK_ID}/status" \
     -H "Content-Type: application/json" \
     -d '{"status": "Done"}'
   ```

## Exit Criteria

- ✅ Selftest passes (7/7 steps)
- ✅ Task status = Done
- ✅ BDD scenarios passing
- ✅ AC mapped to tests

## Error Recovery

- **Selftest fails:** Invoke `governed-governance-debug` Skill
- **BDD fails:** Check `crates/acceptance/src/steps/` for step definitions
- **Platform not running:** Start with `cargo run -p app-http`
```

---

## Testing Your Skill

### Manual Testing

1. **Write the Skill** following the template
2. **Ask Claude to use it:**
   - "Can you help me implement AC-TPL-XXX using the governed workflow?"
   - Claude should invoke your Skill automatically
3. **Verify outputs:**
   - Check that Claude follows the documented steps
   - Verify API calls are made correctly
   - Ensure exit criteria are checked

### Automated Testing (Future)

In v3.2+, we may add:
- BDD scenarios that verify Skill behavior
- Policy tests for SKILL.md structure
- Integration tests that simulate Skill invocation

---

## Further Reading

- **Anthropic's Skills Docs:** https://docs.anthropic.com/claude/docs/agent-skills
- **DevEx Flows:** `specs/devex_flows.yaml` - Canonical workflow definitions
- **Agent Guide:** `docs/AGENT_GUIDE.md` - Operational procedures
- **Platform APIs:** `http://localhost:3000/platform/status` - Runtime introspection
- **xtask Reference:** `docs/reference/xtask-commands.md` - Command documentation

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-11 | Only 5 core Skills | Prevents Skill explosion; maps to devex_flows.yaml |
| 2025-11 | Skills must have REQ/AC | Treats Skills as governed artifacts, not ad-hoc scripts |
| 2025-11 | Use `/platform/*` APIs | Enforces governed interface, not direct file parsing |

---

## Questions?

- **"Should I create a Skill for X?"** → Check: Does X map to a flow in `devex_flows.yaml`? If yes, consider it. If no, it's probably too granular.
- **"My Skill isn't being invoked"** → Check description specificity; add trigger keywords; test with explicit user prompt.
- **"Can I have Skills call other Skills?"** → No, but Skills can recommend that the user invoke other Skills in error recovery sections.

---

**Remember:** Skills are **governed workflows**, not wrappers around commands. If you're tempted to create `skill-xtask-foo`, stop and ask: "What workflow uses this command?"
