---
id: GUIDE-TPL-AGENT-PILOT-001
title: Running a Governed Agent Pilot
doc_type: how_to
status: published
audience: developers, platform-engineers, ai-engineers
tags: [agent, pilot, governance, automation, ai]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-AI-DISCOVERY
  - REQ-TPL-AI-CONTEXT
  - REQ-TPL-AI-IDP-COMPAT
acs:
  - AC-TPL-AGENT-HINTS-SCHEMA
  - AC-TPL-BUNDLE-MANIFEST-LINKED
adrs: [ADR-0004]
last_updated: 2025-12-01
---

# Running a Governed Agent Pilot

This guide walks you through setting up and running a governed autonomous agent pilot in a fork of this template. The approach is designed to be **safe, bounded, and human-supervised**, using the template's governance contracts as guardrails.

**Time:** 2-4 hours for setup and first AC
**Prerequisites:** Fork of template, Claude Code or similar LLM agent, basic understanding of the template's governance model

---

## Introduction: What is a Governed Agent Pilot?

A governed agent pilot is a controlled experiment where an autonomous AI agent works within strict boundaries to implement acceptance criteria, with humans:

- **Defining work scope** (selecting ACs)
- **Monitoring progress** (via selftest and platform APIs)
- **Reviewing artifacts** (code, tests, decisions)
- **Approving changes** (via PR review and CI gates)

**What makes it "governed"?**

- The agent cannot break out of AC-defined contracts
- All changes must pass `cargo xtask selftest` (10-step governance gate)
- Platform APIs (`/platform/agent/hints`, `/platform/graph`) provide structured context
- BDD tests and AC-to-test mappings enforce traceability
- ADRs, issues, and friction logs capture ambiguity and decisions

**This is NOT:**
- Unsupervised autonomous development
- A replacement for human design decisions
- A way to bypass governance or testing

---

## Prerequisites

Before starting your pilot:

### 1. Fork the Template

```bash
# Clone and initialize your fork
git clone https://github.com/YOUR-ORG/your-fork.git
cd your-fork
nix develop
cargo xtask dev-up
```

Verify baseline health:

```bash
cargo xtask selftest
# Should be green or have only known soft AC failures
```

See [QUICKSTART.md](../QUICKSTART.md) for detailed setup instructions.

### 2. Choose Your Agent Platform

This guide uses **Claude Code** as the reference agent, but the patterns work with any LLM agent that can:

- Read/write files
- Execute bash commands
- Call HTTP APIs
- Follow multi-step workflows

**Supported platforms:**
- [Claude Code](https://claude.ai) (recommended, follows CLAUDE.md instructions)
- [GitHub Copilot Workspace](https://copilot.github.com)
- [Cursor](https://cursor.sh)
- Custom agent using OpenAI API or similar

**Agent requirements:**
- Can read and follow CLAUDE.md instructions
- Can call `cargo xtask` commands
- Can query platform APIs (`curl http://localhost:8080/platform/*`)
- Can read bundled context (`cargo xtask bundle implement_ac`)

### 3. Understand the Governance Model

**Read these docs before starting:**

1. [CLAUDE.md](../../CLAUDE.md) - Agent instructions and workflows
2. [AGENT_GUIDE.md](../AGENT_GUIDE.md) - Platform API usage
3. [docs/explanation/TEMPLATE-CONTRACTS.md](../explanation/TEMPLATE-CONTRACTS.md) - Governance contracts
4. [docs/how-to/use-llm-bundles.md](./use-llm-bundles.md) - Context bundling

**Key concepts:**

- **AC-first development:** Everything flows from acceptance criteria in `specs/spec_ledger.yaml`
- **BDD as contract:** Gherkin scenarios tagged with `@AC-XXX` define expected behavior
- **Selftest as gate:** 10-step validation that checks specs, tests, docs, policies
- **Platform APIs as telemetry:** JSON endpoints providing governance state
- **Bundles as context:** Curated file sets for focused work

---

## Scope: One AC, One Agent, Humans-in-Loop

For your first pilot, limit scope to:

**One AC:**
- Choose a simple, well-defined acceptance criterion
- Prefer new features over refactoring
- Avoid cross-cutting concerns or architecture changes

**One agent:**
- Single agent instance working sequentially
- No parallel agents (avoid conflicts)
- Human reviews every PR before merge

**Humans-in-loop at:**
- **AC selection** (human decides what to build)
- **Design decisions** (agent drafts ADRs, human approves)
- **PR approval** (human reviews code, tests, docs)
- **Merge decision** (human verifies selftest green)

---

## Phase 1: Define the Pilot AC

Choose an AC that is:
- **Small:** Can be implemented in 2-4 hours
- **Well-specified:** Clear in `spec_ledger.yaml`
- **Testable:** Can write BDD scenario with clear assertions
- **Low-risk:** No security, auth, or data migration concerns

### Step 1.1: Review Available ACs

Start the platform service:

```bash
# In one terminal:
cargo run -p app-http
# Service starts on http://localhost:8080
```

Query for available work:

```bash
# Get prioritized work suggestions
curl http://localhost:8080/platform/agent/hints | jq .

# Example output:
{
  "hints": [
    {
      "task_id": "TASK-001",
      "task_name": "Implement echo endpoint",
      "status": "Todo",
      "priority": 1,
      "ac_ids": ["AC-TPL-005"],
      "requirement_ids": ["REQ-TPL-CORE"]
    }
  ]
}
```

**Or use CLI:**

```bash
cargo xtask tasks-list --status Todo
cargo xtask suggest-next
```

### Step 1.2: Verify AC is Well-Defined

Check the AC in `specs/spec_ledger.yaml`:

```bash
grep -A 5 "AC-TPL-005" specs/spec_ledger.yaml
```

**Good AC (ready for pilot):**
```yaml
- id: AC-TPL-005
  text: "GET /api/echo returns the input message as JSON"
  must_have_ac: true
  tests:
    - type: bdd
      tag: "@AC-TPL-005"
```

**Bad AC (not ready):**
```yaml
- id: AC-TPL-099
  text: "System should be performant"  # Too vague
  # Missing tests array
```

### Step 1.3: Create a Task for the AC (Optional)

If the AC doesn't have a task entry, add one to `specs/tasks.yaml`:

```yaml
- id: TASK-PILOT-001
  name: "Implement echo endpoint (pilot)"
  description: "First agent pilot: implement AC-TPL-005 end-to-end"
  status: Todo
  priority: 1
  story_id: US-TPL-001
  requirement_ids: [REQ-TPL-CORE]
  ac_ids: [AC-TPL-005]
  estimated_hours: 2
  dependencies: []
```

**Verify task visibility:**

```bash
cargo xtask tasks-list --task TASK-PILOT-001
curl http://localhost:8080/platform/tasks?id=TASK-PILOT-001
```

---

## Phase 2: Configure Agent Tools

Set up your agent with the minimum necessary tools.

### Step 2.1: Define Agent Permissions

**Recommended tool set for pilot:**

```yaml
# .claude/agents/pilot-agent.md
---
name: pilot-agent
description: Autonomous agent for governed AC implementation pilot. Implements a single AC end-to-end with human oversight.
permissionMode: restricted
model: inherit
tools:
  # File operations (least-privilege)
  - Read
  - Edit
  - Glob
  - Grep

  # Bash for xtask commands only
  - Bash

  # No Write tool (Edit is safer)
  # No unrestricted web access

skills:
  - governed-feature-dev

system: |
  You are working in a governed Rust-as-Spec platform cell.

  Your mission: Implement acceptance criteria following the governed-feature-dev skill workflow.

  Constraints:
  - You MUST use cargo xtask commands for all workflows
  - You MUST query /platform/agent/hints for work suggestions
  - You MUST use bundles for context (cargo xtask bundle implement_ac)
  - You MUST validate with selftest before declaring work complete
  - You MUST capture design decisions in ADRs
  - You MUST NOT invent AC IDs (they come from spec_ledger.yaml)

  Workflow:
  1. Query /platform/agent/hints for next AC
  2. Generate bundle: cargo xtask bundle implement_ac
  3. Read bundle context
  4. Implement code + BDD tests
  5. Validate: cargo xtask test-ac AC-XXX
  6. Full check: cargo xtask selftest
  7. If green: create PR
  8. If questions: draft ADR or create issue

  You work autonomously within these guardrails. Humans review your PRs.
---
```

**Validate agent definition:**

```bash
cargo xtask agents-lint
# Should pass with no violations
```

### Step 2.2: Grant Platform API Access

Ensure your agent can query the platform:

```bash
# Start the service (if not already running)
cargo run -p app-http &

# Test API access
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/agent/hints
curl http://localhost:8080/platform/graph
```

**Key endpoints for agents:**

| Endpoint | Purpose | Example |
|----------|---------|---------|
| `/platform/status` | Governance health, AC counts | Overall state |
| `/platform/agent/hints` | Prioritized work (Todo/InProgress) | What to work on next |
| `/platform/graph` | Full governance graph | AC→test→doc mappings |
| `/platform/tasks` | Task list with filters | `?status=Todo` |
| `/platform/coverage` | AC coverage summary | Test health |

See [AGENT_GUIDE.md](../AGENT_GUIDE.md) for full API reference.

### Step 2.3: Prepare Bundle Configuration

Verify bundle tasks are available:

```bash
cargo xtask bundle implement_ac > /tmp/test-bundle.md
wc -l /tmp/test-bundle.md
# Should show bundled context
```

**Bundle tasks available:**
- `implement_ac` - Single AC implementation (recommended for pilot)
- `implement_feature` - Broader feature context
- `debug_tests` - Test debugging

See `.llm/contextpack.yaml` for configuration.

---

## Phase 3: Agent Workflow (Query → Bundle → Edit → Validate → PR)

This is the core autonomous loop. The agent follows this pattern for each AC.

### Step 3.1: Agent Queries for Work

**Agent action:**

```bash
# Query for next work
curl http://localhost:8080/platform/agent/hints | jq '.hints[0]'
```

**Example response:**

```json
{
  "task_id": "TASK-PILOT-001",
  "task_name": "Implement echo endpoint (pilot)",
  "status": "Todo",
  "priority": 1,
  "ac_ids": ["AC-TPL-005"],
  "requirement_ids": ["REQ-TPL-CORE"]
}
```

**Agent reasoning:**
- "I see TASK-PILOT-001 with AC-TPL-005 is highest priority"
- "Status is Todo, so no one else is working on it"
- "I'll start implementing AC-TPL-005"

### Step 3.2: Agent Generates Bundle

**Agent action:**

```bash
# Generate focused context for AC-TPL-005
cargo xtask bundle implement_ac > /tmp/ac-context.md
```

**Agent reads bundle to understand:**
- Existing AC patterns in `spec_ledger.yaml`
- BDD scenario examples in `specs/features/*.feature`
- Code patterns in `crates/core` and `crates/app-http`
- Test patterns in `crates/acceptance`

**Bundle provides:**
- ✅ Concrete examples to pattern-match
- ✅ Coding style and conventions
- ✅ Error handling patterns
- ✅ Test structure

### Step 3.3: Agent Implements AC

**Agent actions:**

1. **Understand the AC:**
   - Read `specs/spec_ledger.yaml` entry for AC-TPL-005
   - "GET /api/echo returns the input message as JSON"

2. **Write BDD scenario:**
   ```gherkin
   # specs/features/template_core.feature
   @AC-TPL-005
   Scenario: Echo endpoint returns input message
     When I GET /api/echo?message=hello
     Then I receive 200 with JSON containing field "echo" with value "hello"
   ```

3. **Implement handler:**
   ```rust
   // crates/app-http/src/lib.rs
   #[tracing::instrument(name = "echo", skip(request_id))]
   async fn echo_handler(
       Extension(request_id): Extension<RequestId>,
       Query(params): Query<EchoQuery>
   ) -> Result<Json<EchoResponse>, AppError> {
       info!(ac = "AC-TPL-005", "Echo request");
       Ok(Json(EchoResponse {
           echo: params.message
       }))
   }
   ```

4. **Add step definitions:**
   ```rust
   // crates/acceptance/src/steps/template_core.rs
   #[when(expr = "I GET /api/echo?message={word}")]
   async fn get_echo(world: &mut TemplateWorld, message: String) {
       let response = world.client
           .get(&format!("{}/api/echo?message={}", world.base_url, message))
           .send()
           .await
           .expect("Failed to send request");
       world.last_response = Some(response);
   }
   ```

**Agent constraints:**
- ✅ Uses existing patterns from bundle
- ✅ Follows coding style (tracing, error handling)
- ✅ Tags scenario with `@AC-TPL-005`
- ✅ No new AC IDs invented

### Step 3.4: Agent Validates Changes

**Agent action (progressive validation):**

```bash
# 1. Quick syntax check
cargo xtask check

# 2. Run BDD for this AC
cargo xtask bdd --tags @AC-TPL-005

# 3. Test just this AC and related tests
cargo xtask test-ac AC-TPL-005

# 4. Full governance check
cargo xtask selftest
```

**Agent reasoning at each step:**

**After `check`:**
- ✅ Code compiles, no clippy warnings
- ✅ Formatting correct
- → Proceed to BDD

**After `bdd`:**
- ✅ Scenario `@AC-TPL-005` passes
- → Proceed to AC-focused tests

**After `test-ac`:**
- ✅ All tests mapped to AC-TPL-005 pass
- → Proceed to full selftest

**After `selftest`:**
- ✅ All 10 governance gates pass
- → Ready to create PR

**If any step fails:**
- Agent reviews error output
- Agent attempts fix using bundle context
- Agent re-runs validation
- If still blocked: agent creates issue or draft ADR with question

### Step 3.5: Agent Creates PR

**Agent action:**

```bash
# Create feature branch
git checkout -b pilot/ac-tpl-005-echo-endpoint

# Stage changes
git add specs/features/template_core.feature
git add crates/app-http/src/lib.rs
git add crates/acceptance/src/steps/template_core.rs

# Commit with AC reference
git commit -m "feat: implement echo endpoint (AC-TPL-005)

- Add GET /api/echo endpoint
- Accept message query parameter
- Return JSON with echo field
- Add BDD scenario @AC-TPL-005
- All tests passing

AC-TPL-005: GET /api/echo returns the input message as JSON"

# Push branch
git push -u origin pilot/ac-tpl-005-echo-endpoint

# Create PR using gh CLI
gh pr create \
  --title "feat: implement echo endpoint (AC-TPL-005)" \
  --body "$(cat <<'EOF'
## Summary

Implements AC-TPL-005: GET /api/echo returns the input message as JSON

Changes:
- Added `/api/echo` HTTP endpoint handler
- Accepts `message` query parameter
- Returns JSON: `{"echo": "<message>"}`
- Added BDD scenario `@AC-TPL-005` in template_core.feature
- Added Cucumber step definitions

## Validation

✅ `cargo xtask check` - passed
✅ `cargo xtask bdd` - scenario @AC-TPL-005 passed
✅ `cargo xtask test-ac AC-TPL-005` - passed
✅ `cargo xtask selftest` - all gates green

## Test Plan

- [x] Unit tests pass
- [x] BDD scenario @AC-TPL-005 passes
- [x] Selftest passes
- [x] No clippy warnings
- [x] Code formatted
- [ ] Human review of code quality
- [ ] Human review of test coverage
- [ ] Human approval to merge

## Traceability

- **AC:** AC-TPL-005
- **Requirement:** REQ-TPL-CORE
- **Story:** US-TPL-001
- **Task:** TASK-PILOT-001
EOF
)"
```

**PR checklist (for human reviewer):**

- [ ] AC-TPL-005 is valid and approved for implementation
- [ ] BDD scenario correctly tests the AC
- [ ] Code follows existing patterns
- [ ] No security concerns (input validation, etc.)
- [ ] Tests are comprehensive
- [ ] Documentation updated (if needed)
- [ ] Selftest green in CI

---

## Phase 4: Monitoring and Guardrails

While the agent works, monitor its progress and enforce boundaries.

### Step 4.1: Monitor Agent Progress

**Check governance state:**

```bash
# Overall health
curl http://localhost:8080/platform/status | jq .

# AC coverage
curl http://localhost:8080/platform/coverage | jq .

# Current task status
curl http://localhost:8080/platform/tasks?status=InProgress | jq .
```

**Watch logs:**

```bash
# If agent is using xtask commands, watch output
cargo xtask selftest --verbose

# Check git commits
git log --oneline
```

**Expected pattern:**
- Agent creates feature branch
- Agent makes incremental commits
- Agent runs validation after each change
- Agent creates PR when selftest green

### Step 4.2: Selftest as Hard Gate

**Selftest must pass before merge.** This is non-negotiable.

```bash
cargo xtask selftest
```

**10 governance gates:**

1. ✅ Code compiles (`cargo build --all`)
2. ✅ Tests pass (`cargo test --all`)
3. ✅ Clippy clean (`cargo clippy --all`)
4. ✅ Formatting correct (`cargo fmt --check`)
5. ✅ BDD scenarios pass (`cargo xtask bdd`)
6. ✅ Policies pass (`cargo xtask policy-test`)
7. ✅ Docs valid (`cargo xtask docs-check`)
8. ✅ AC status consistent (`cargo xtask ac-status`)
9. ✅ Skills/Agents valid (`cargo xtask skills-lint && agents-lint`)
10. ✅ No secrets in logs (`cargo xtask check-secrets`)

**If selftest fails:**
- Agent MUST fix before creating PR
- Agent MUST NOT merge with failing selftest
- Agent SHOULD capture blocker in ADR or issue if cannot resolve

### Step 4.3: Human Review Checkpoints

**When to intervene:**

| Scenario | Action |
|----------|--------|
| Agent creates invalid AC ID | Reject PR, remind agent to use `spec_ledger.yaml` |
| Agent changes architecture | Pause pilot, discuss design first |
| Agent skips tests | Reject PR, require BDD scenario |
| Agent bypasses selftest | Block merge, reinforce guardrails |
| Agent creates ADR for decision | Review reasoning, approve/modify |
| Agent gets stuck (no progress 30min) | Check logs, provide hint, or reassign work |

**Review cadence (recommended):**
- **Every PR:** Full code review before merge
- **Daily:** Check `/platform/status` for drift
- **Weekly:** Review friction log for DevEx issues

### Step 4.4: Kill Switch

If agent behavior is unsafe or unproductive:

```bash
# Stop the agent process
# (method depends on agent platform)

# Review changes
git status
git diff

# Revert if needed
git reset --hard origin/main

# Capture issue
gh issue create --title "Agent pilot issue: [describe]" --body "..."
```

---

## Phase 5: Capture Outcomes

Document pilot learnings for iteration.

### Step 5.1: Friction Log

Capture DevEx issues encountered during the pilot:

```bash
# View current friction
cargo xtask friction-list --status open

# Add new friction entry
vim friction/FRICTION-PILOT-001.yaml
```

**Example friction entry:**

```yaml
id: FRICTION-PILOT-001
title: "Agent unclear on BDD step reuse strategy"
description: |
  During pilot, agent created duplicate step definitions instead of
  reusing existing steps. Bundle context included examples but agent
  didn't recognize the pattern.

  Improvement: Add explicit comment in step definitions about reuse.
category: devex
severity: minor
status: open
created: 2025-12-01
affected_workflows:
  - governed-feature-dev
  - AC implementation
```

### Step 5.2: ADR for Design Decisions

If agent encountered ambiguity and drafted an ADR:

```bash
# Review draft ADR
ls docs/adr/

# Refine and accept
vim docs/adr/ADR-PILOT-ECHO-PARAMS.md
cargo xtask adr-check
```

**Example ADR from pilot:**

```markdown
---
id: ADR-PILOT-001
title: Echo Endpoint Query Parameter Validation
status: accepted
date: 2025-12-01
decision_makers: [pilot-agent, human-reviewer]
---

# Context

AC-TPL-005 requires "GET /api/echo returns the input message as JSON".
Spec doesn't specify what happens if `message` query param is missing.

# Options

1. Return 400 error if missing
2. Return empty string
3. Return error in JSON response

# Decision

Return 400 Bad Request if `message` parameter is missing.

**Rationale:**
- Matches existing validation patterns in template
- Clear error feedback to client
- Prevents silent failures

# Consequences

- Clients must always provide `message` parameter
- BDD scenario updated to test missing param case
- Consistent with REQ-TPL-ERROR-HANDLING
```

### Step 5.3: Pilot Report

After first AC is complete, document outcomes:

**Template: Pilot Report**

```markdown
# Agent Pilot Report: AC-TPL-005

**Date:** 2025-12-01
**Agent:** pilot-agent (Claude Code)
**AC:** AC-TPL-005 - GET /api/echo returns the input message as JSON
**Duration:** 2.5 hours

## Summary

✅ Successfully implemented AC-TPL-005 end-to-end
✅ Selftest passed on first attempt
✅ PR approved and merged

## Metrics

- **Time to PR:** 1.5 hours
- **Validation attempts:** 1 (selftest passed immediately)
- **Human interventions:** 0
- **ADRs created:** 1 (query param validation)
- **Friction entries:** 1 (step definition reuse)

## What Worked

- Platform APIs (`/agent/hints`) provided clear work direction
- Bundles (`implement_ac`) gave sufficient context
- Selftest caught all issues before PR creation
- Agent correctly followed governed-feature-dev skill workflow

## What Didn't Work

- Agent initially created duplicate step definitions (fixed after hint)
- Bundle size slightly too large (300KB → could trim to 200KB)

## Improvements for Next Pilot

1. Add explicit step reuse guidance in CLAUDE.md
2. Trim bundle to exclude test utilities (not needed for implementation)
3. Add example ADR in bundle context for faster decision capture

## Recommendation

✅ Ready to expand pilot to 3-5 ACs
✅ Agent can work autonomously with daily human check-ins
⚠️ Keep architecture decisions human-owned
```

---

## Success Criteria

Your pilot is successful if:

**Technical:**
- ✅ AC implemented correctly (BDD scenario passes)
- ✅ Selftest green (all 10 gates pass)
- ✅ No regressions (existing tests still pass)
- ✅ PR approved and merged

**Process:**
- ✅ Agent followed governed-feature-dev workflow
- ✅ Agent queried `/platform/agent/hints` for work
- ✅ Agent used bundles for context
- ✅ Agent captured decisions in ADRs or issues
- ✅ Human reviewed and approved PR

**Learning:**
- ✅ Friction log entries captured DevEx issues
- ✅ Pilot report documents outcomes
- ✅ Recommendations for next iteration

**Guardrails:**
- ✅ Agent did not invent AC IDs
- ✅ Agent did not bypass selftest
- ✅ Agent did not make architecture changes without human approval

---

## Troubleshooting

### Agent Gets Stuck on Validation

**Symptom:** Agent runs selftest repeatedly, never gets green.

**Diagnosis:**

```bash
# Run selftest manually to see failure
cargo xtask selftest --verbose
```

**Common causes:**

1. **BDD scenario syntax error**
   - Fix: Run `cargo xtask bdd` to see Gherkin errors
   - Agent should read error output and fix scenario

2. **AC not tagged in scenario**
   - Fix: Ensure scenario has `@AC-XXX` tag
   - Policy `features.rego` enforces this

3. **Test coverage gap**
   - Fix: Add missing test or update `spec_ledger.yaml` tests array
   - Run `cargo xtask ac-status` to see gaps

4. **Docs drift**
   - Fix: Run `cargo xtask docs-check` to see mismatch
   - Update frontmatter or doc_index.yaml

**Agent recovery:**
- Agent reads error output from failed command
- Agent uses bundle context to find correct pattern
- Agent applies fix and re-runs validation
- If still stuck after 3 attempts: agent creates issue for human review

### Agent Creates Invalid AC ID

**Symptom:** Agent edits `spec_ledger.yaml` and invents new AC ID.

**Prevention:**

1. Agent definition includes constraint:
   ```yaml
   system: |
     You MUST NOT invent AC IDs. They come from spec_ledger.yaml.
   ```

2. Pre-commit hook runs `cargo xtask policy-test`:
   ```bash
   # Blocks commits with policy violations
   ```

3. CI enforces selftest:
   ```yaml
   # .github/workflows/tier1-selftest.yml
   ```

**If it happens:**
- Reject PR
- Remove invented AC ID
- Remind agent of constraint
- Add friction log entry to improve agent instructions

### Agent Bypasses Selftest

**Symptom:** Agent creates PR without running selftest.

**Detection:**

```bash
# PR description should include:
# ✅ cargo xtask selftest - all gates green

# If missing, PR is incomplete
```

**Enforcement:**

1. CI runs selftest on every PR (required check)
2. Branch protection requires CI to pass
3. Human reviewer checks PR description for validation evidence

**Remediation:**
- Request agent to run selftest
- If agent refuses: block PR and escalate

### Bundle Context Insufficient

**Symptom:** Agent doesn't have enough context to implement AC.

**Diagnosis:**

```bash
# Check bundle size
cargo xtask bundle implement_ac > /tmp/bundle.md
wc -c /tmp/bundle.md
# If < 50KB, might be too small
```

**Fix:**

Edit `.llm/contextpack.yaml` to include more files:

```yaml
tasks:
  implement_ac:
    max_bytes: 300000  # Increase limit
    include:
      - specs/spec_ledger.yaml
      - specs/features/**/*.feature
      - crates/core/**/*.rs
      - crates/app-http/**/*.rs
      - crates/model/**/*.rs  # Add model layer
      - docs/how-to/add-http-endpoint.md  # Add how-to guide
```

**Or create custom bundle for pilot:**

```yaml
tasks:
  pilot_ac:
    max_bytes: 250000
    include:
      # Exactly what pilot agent needs
      - specs/spec_ledger.yaml
      - specs/features/template_core.feature
      - crates/app-http/src/lib.rs
      - crates/acceptance/src/steps/template_core.rs
```

### Selftest Passes Locally but Fails in CI

**Symptom:** Agent sees green selftest locally, but CI is red.

**Common causes:**

1. **Platform difference** (Linux vs macOS vs Windows)
   - CI runs on Linux (Tier-1)
   - Local might be macOS (Tier-2)
   - Fix: Run `nix develop` to match CI environment

2. **Missing dependency**
   - CI has clean environment
   - Local has cached dependencies
   - Fix: Run `cargo clean && cargo xtask selftest`

3. **Timing/concurrency issue**
   - CI might be slower
   - Fix: Add retries or timeouts in tests

**Diagnosis:**

```bash
# Run in CI-equivalent environment
nix develop
cargo clean
cargo xtask selftest
```

---

## Next Steps After First Pilot

Once your first AC is successfully implemented:

### Expand Scope

1. **More ACs** - Pilot 3-5 similar ACs in same domain
2. **Different domains** - Try AC in different area (e.g., validation vs. endpoints)
3. **Refactoring** - Test agent on maintenance tasks

### Automate More

1. **Auto-assign work** - Agent queries `/agent/hints` daily
2. **Scheduled runs** - Agent works during off-hours
3. **Batch PRs** - Agent groups related ACs into single PR

### Improve Context

1. **Refine bundles** - Trim unnecessary files, add missing patterns
2. **Better prompts** - Improve agent system prompt based on learnings
3. **Custom bundles** - Create domain-specific bundles

### Measure Impact

Track:
- **Time to implement AC** (with agent vs. without)
- **Selftest success rate** (first-time green vs. iterations)
- **PR review time** (how long for human approval)
- **Regression rate** (does agent break existing tests)

---

## Related Documentation

- [CLAUDE.md](../../CLAUDE.md) - Agent instructions and workflows
- [AGENT_GUIDE.md](../AGENT_GUIDE.md) - Platform API reference
- [docs/how-to/use-llm-bundles.md](./use-llm-bundles.md) - Context bundling guide
- [docs/explanation/TEMPLATE-CONTRACTS.md](../explanation/TEMPLATE-CONTRACTS.md) - Governance contracts
- [ADR-0004](../adr/0004-platform-introspection-apis.md) - Platform API design
- [docs/design/agent-interface.md](../design/agent-interface.md) - Agent-native interface design
- [docs/design/skills-guide.md](../design/skills-guide.md) - Skills governance
- [docs/design/agents-governance.md](../design/agents-governance.md) - Agents governance

---

## Summary Checklist

Before starting pilot:
- [ ] Fork template and run `cargo xtask dev-up`
- [ ] Verify baseline selftest green
- [ ] Choose simple, well-defined AC
- [ ] Define agent with restricted permissions
- [ ] Start platform service (`cargo run -p app-http`)
- [ ] Test bundle generation (`cargo xtask bundle implement_ac`)

During pilot:
- [ ] Agent queries `/platform/agent/hints` for work
- [ ] Agent generates bundle for context
- [ ] Agent implements AC following governed-feature-dev skill
- [ ] Agent validates with `cargo xtask selftest`
- [ ] Agent creates PR with validation evidence
- [ ] Human reviews and approves PR

After pilot:
- [ ] Capture friction in friction log
- [ ] Review and accept ADRs created by agent
- [ ] Write pilot report with metrics and learnings
- [ ] Plan next iteration (expand or refine)

**Success = AC implemented, selftest green, PR merged, learnings captured.**
