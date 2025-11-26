# Agent Guide: Operating the Rust-as-Spec Platform Cell

**Audience:** LLM agents (Claude, GPT-4, etc.) driving development workflows  
**Purpose:** Concrete instructions for discovering, planning, and executing work within this governed repository

---

## Core Principle

This repository is a **self-governing platform cell**. Your role as an agent is to:
1. **Discover** what needs to be done via structured APIs
2. **Plan** work using recommended flows and tasks
3. **Execute** via `cargo xtask` commands
4. **Validate** using `cargo xtask selftest` as ground truth

**Critical Rule:** Never bypass the governance contracts. If `selftest` fails, the work is not done.

---

## 0. First Run: Bootstrap the Environment

**Preferred entry point for humans and agents:**

```bash
# One-command bootstrap
cargo run -p xtask -- dev-up

# What it does:
# - Installs pre-commit hooks (if missing)
# - Checks Docker availability
# - Runs governance check (low-resource mode)
# - Displays next steps
```

After `dev-up` completes successfully, you're ready to start the service and interact with the platform APIs.

### Platform-Specific Notes

**Linux/macOS with Nix:**
```bash
nix develop
cargo run -p xtask -- dev-up
# Works as documented, matches CI exactly
```

**WSL2 with Nix:**
```bash
# Inside WSL2 Ubuntu shell
nix develop
cargo run -p xtask -- dev-up
# Recommended for Windows teams; matches CI exactly
```

**Native Windows (Known Caveat):**
```powershell
# Runs successfully, but may intermittently fail on `cargo rebuild` with:
# error: failed to remove xtask.exe: os error 5
#
# This is Windows file locking, not a test failure. See:
# docs/MISSING_MANUAL.md → "Platform Support" → "Tier 2: Native Windows"
#
# Mitigation:
# 1. Exclude target/ from antivirus real-time scanning
# 2. Or: Use WSL2 for canonical validation
cargo run -p xtask -- dev-up
```

---

## 1. Discovering Work

### Query Available Tasks

**CLI - List Tasks:**
```bash
# List all tasks with details (ID, status, requirement, ACs, owner, title)
cargo xtask tasks-list

# Example output:
# Tasks (from specs/tasks.yaml)
# ID                             Status       Requirement          ACs                            Owner        Title
# ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
# TASK-TPL-STATUS-CLI-001        InProgress   REQ-PLT-STATUS-CLI   AC-PLT-017                     agent        Implement CLI governance status dashboard
# TASK-TPL-FIX-AUDIT-001         Todo         REQ-PLT-SECURITY     AC-PLT-006, AC-PLT-007         team         Fix cargo-audit findings
```

**CLI - Create Task:**
```bash
# Create a new task (validates requirement and ACs exist in spec_ledger.yaml)
cargo xtask task-create \
  --id TASK-NEW-001 \
  --title "Implement new feature" \
  --req REQ-TPL-HEALTH \
  --ac AC-TPL-001 \
  --owner agent \
  --status Todo

# Task is written to specs/tasks.yaml with validated linkage
```

**CLI - Update Task:**
```bash
# Update task status (enforces valid transitions via business-core)
cargo xtask task-update \
  --id TASK-NEW-001 \
  --status InProgress \
  --owner agent \
  --title "Updated task title"

# Valid transitions: Todo → InProgress → Review → Done
# Invalid transitions (e.g., Done → Todo) are rejected with clear error
```

**HTTP API - GET Tasks:**
```bash
# List all tasks
curl http://localhost:8080/platform/tasks

# Filter by status
curl "http://localhost:8080/platform/tasks?status=InProgress"

# Filter by requirement
curl "http://localhost:8080/platform/tasks?req=REQ-TPL-HEALTH"
```

**Response structure:**
```json
{
  "tasks": [
    {
      "id": "TASK-TPL-STATUS-CLI-001",
      "title": "Implement CLI governance status dashboard",
      "requirement": "REQ-PLT-STATUS-CLI",
      "acs": ["AC-PLT-017"],
      "status": "InProgress",
      "owner": "agent",
      "labels": ["platform", "devex", "observability"],
      "docs": {
        "design": [],
        "plan": []
      }
    }
  ]
}
```

**HTTP API - Update Task Status:**
```bash
# Transition task to next status
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# Success: 204 No Content
# Invalid transition: 500 with error message containing "Invalid status transition"
```

**Simple Agent Loop Example:**
```bash
# 1. Get prioritized hints
curl http://localhost:8080/platform/agent/hints | jq '.hints[0]'
# → { "task_id": "TASK-001", "status": "Todo", "reason": "...", "recommended_sequence": [...] }

# 2. Get detailed steps for the task
curl "http://localhost:8080/platform/tasks/suggest-next?task=TASK-001"
# → { "task": {...}, "recommended_sequence": [{kind: "command", value: "cargo xtask bundle ..."}] }

# 3. Update status to InProgress
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# 4. Execute recommended commands
cargo xtask bundle TASK-001
cargo xtask test-ac AC-TPL-001

# 5. Mark as Review or Done
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "Review"}'
```

### Get Context-Aware Guidance

```bash
# Ask "what should I do next for task X?"
cargo xtask suggest-next --task implement_ac

# JSON output
cargo xtask suggest-next --task implement_ac --json
```

**HTTP API:**
```bash
curl "http://localhost:8080/platform/tasks/suggest-next?task=implement_ac"
```

**Response includes:**
- Sequence of steps (commands or edits)
- Status of each step (`Pending` or `Satisfied`)
- Summary text for each action

**Key Insight:** The `status` field tells you **what's already done**. Don't repeat satisfied steps.

### Get Prioritized Hints for Next Work

The `/platform/agent/hints` endpoint provides high-level guidance about what work is ready and prioritized:

```bash
# Get hints about what tasks are ready for work
curl http://localhost:8080/platform/agent/hints
```

**Response structure:**
```json
{
  "hints": [
    {
      "task_id": "TASK-TPL-STATUS-CLI-001",
      "status": "InProgress",
      "requirement_ids": ["REQ-PLT-STATUS-CLI"],
      "ac_ids": ["AC-PLT-017"],
      "reason": "Task 'Implement CLI governance status dashboard' is ready for work",
      "recommended_sequence": [
        {
          "kind": "command",
          "value": "cargo xtask bundle TASK-TPL-STATUS-CLI-001"
        },
        {
          "kind": "command",
          "value": "cargo xtask test-ac AC-PLT-017"
        }
      ]
    }
  ]
}
```

**Use cases:**
- Start a new session and discover what's already in progress or ready to start
- Filter for `Todo` and `InProgress` tasks automatically
- Get direct links to requirements and ACs for context
- Follow `recommended_sequence` for standard workflow steps

**Difference from `/platform/tasks/suggest-next`:**
- **Hints** = "What tasks are ready for me to work on?" (filters + prioritizes)
- **Suggest-next** = "For this specific task, what's the detailed step-by-step sequence?" (deep workflow)

---

## 2. Understanding the System State

### Check Governance Health

```bash
# Get platform status
curl http://localhost:8080/platform/status
```

**Key fields:**
- `governance.policies.status`: `"pass"` | `"fail"` | `"unknown"`
- `governance.ledger`: Story/Requirement/AC counts
- `governance.devex`: Command/flow counts
- `governance.docs`: Documentation inventory
- `governance.tasks`: Available work units

**Decision Logic:**
- If `policies.status == "fail"` → Run `cargo xtask policy-test` to see failures
- If `policies.status == "unknown"` → Policies haven't been run yet; run `cargo xtask policy-test`

### Inspect the Governance Graph

```bash
# Get full graph as JSON
curl http://localhost:8080/platform/graph

# View graph visually
open http://localhost:8080/ui/graph
```

**Use cases:**
- Find which requirements link to which ACs
- Find which docs cover which requirements
- Find which commands are used in which flows

### Query Documentation

```bash
# Get doc index
curl http://localhost:8080/platform/docs/index
```

**Use cases:**
- Find design docs for a requirement
- Find ADRs that explain architectural decisions
- Verify doc front-matter is valid

### Query DevEx Flows

```bash
# Get all flows
curl http://localhost:8080/platform/devex/flows
```

**Use cases:**
- Understand recommended workflow for a task
- Find which commands are part of which flow
- Discover step-by-step guidance

---

## 3. Executing Workflows

### Standard AC-First Workflow

When implementing a new feature:

#### Before Starting Work

```bash
# 1. Check what ACs exist for your feature area
cargo xtask ac-coverage | grep FEATURE-NAME
# Example: cargo xtask ac-coverage | grep PLT-GRAPH

# 2. Inspect specific AC status
cargo xtask ac-coverage | grep AC-PLT-XXX
# Look for: ✅ (has tests) or ❌ (no tests)

# 3. Get context for a specific AC
cargo xtask bundle implement_ac --ac AC-PLT-XXX
# Output: .llm/bundle/implement_ac.md (max 250KB)
```

**Expected output from ac-coverage:**
```
Feature: Platform Graph Visualization
  ✅ AC-PLT-GRAPH-001: Export graph as JSON
  ❌ AC-PLT-GRAPH-002: Export graph as DOT format
  ✅ AC-PLT-GRAPH-003: Validate graph invariants
```

**Interpretation:**
- ✅ = AC has BDD tests wired
- ❌ = AC has no tests (work needed)

#### During Implementation

```bash
# 1. Create AC (if new)
cargo xtask ac-new AC-MYSERV-001 "Description" --requirement REQ-ID

# 2. Generate BDD scenario stubs
cargo xtask ac-suggest-scenarios AC-MYSERV-001
# Outputs suggested Gherkin scenarios based on AC description

# 3. Add scenario to appropriate .feature file
# Edit specs/features/your_feature.feature

# 4. Implement step definitions
# Edit crates/acceptance/src/steps/your_module.rs

# 5. Run tests for specific AC (RECOMMENDED: Use selective testing)
cargo xtask test-ac AC-MYSERV-001
# Fast, focused testing of your AC only

# Alternative: Direct cucumber invocation
cargo test -p acceptance --test acceptance -- --include-tag AC-MYSERV-001

# 6. Implement code based on bundle context
cargo xtask bundle implement_ac
# Read .llm/bundle/implement_ac.md for context

# 7. Validate your changes (RECOMMENDED: Selective testing)
cargo xtask test-changed
# Runs only tests affected by your edits

# Alternative: Full BDD suite (slower, especially on Windows)
cargo xtask bdd
```

#### Before Committing

**Recommended workflow (fast):**
```bash
# 1. Test only what changed
cargo xtask test-changed
# Runs affected tests only (seconds to minutes)

# 2. Validate kernel ACs are green
cargo xtask ac-coverage
# All kernel ACs MUST show ✅
```

**Full validation (use Tier-1 environment):**
```bash
# In WSL2 or Linux with Nix (matches CI)
nix develop
cargo xtask selftest
# All 7 steps must pass (10-20 minutes on Tier-1)
```

**Critical:**
- **Default ladder:** `cargo xtask test-changed` after edits -> `cargo xtask test-ac <ID>` when you touched a single AC -> `nix develop && cargo xtask selftest` before merge (Tier-1)
- **Pre-commit:** Use `cargo xtask test-changed` for fast feedback
- **Pre-merge:** Run `cargo xtask selftest` in Tier-1 (Nix+Linux/WSL2)
- **You MUST run `cargo xtask ac-coverage`** before claiming work is complete
- If kernel ACs show ❌, work is NOT done
- **Never skip selftest** - but run it in the right environment (Tier-1)

**Performance Note:**
- On **native Windows** (Tier-2): `selftest` may take 2+ hours due to file locking. Use `test-changed` for iteration, `selftest` in WSL2 for final validation.
- On **Nix+Linux/WSL2** (Tier-1): `selftest` takes 10-20 minutes. This is your canonical validation environment.

See `docs/SELECTIVE_TESTING.md` for complete guide.

### When to Use Which Task

| Task | When to Use | Risk Level |
|------|-------------|------------|
| `implement_ac` | Adding new behavior | Medium |
| `fix_audit` | Addressing `cargo audit` findings | High (security) |
| `design_decision` | Documenting architecture choice | Low |
| `prepare_release` | Cutting a new version | High (production) |
| `refactor_core` | Changing business logic | High |

**Risk levels:**
- **Low**: Can proceed autonomously
- **Medium**: Should summarize plan and ask for confirmation
- **High**: **Must** get explicit human approval before executing

---

## 4. Validation and Verification

### The Selftest Contract

`cargo xtask selftest` is the **supreme arbiter** of correctness. It runs 7 steps:

1. **Core checks** (fmt, clippy, tests) - Code quality
2. **BDD** - Behavior matches specs
3. **AC mapping** - Traceability (ACs have tests, ADRs exist)
4. **LLM bundler** - Context generation works
5. **Policy tests** - Compliance (OPA/Rego)
6. **DevEx contract** - Required commands exist
7. **Graph invariants** - Structural integrity (no orphans, missing ACs)

**What failure means:**
- Step 1-2: Code doesn't work
- Step 3: Metadata is broken
- Step 4: Agent infrastructure is broken
- Step 5-7: **Governance is violated** (cannot merge)

**Agent Decision Tree:**
```
Run cargo xtask selftest
  ├─ All pass → ✅ Work is valid, can proceed
  ├─ Core/BDD fail → Fix code/tests
  ├─ AC mapping fail → Fix spec linkage
  ├─ Bundle fail → Fix .llm/contextpack.yaml
  ├─ Policy fail → Run cargo xtask policy-test for details
  ├─ DevEx fail → Update devex_flows.yaml
  └─ Graph fail → Fix spec_ledger.yaml (add missing ACs, etc.)
```

### Incremental Validation

For faster feedback during development:

```bash
# Quick code checks
cargo xtask check

# AC coverage status (shows which ACs have tests)
cargo xtask ac-coverage
# Example output:
#   Kernel ACs: 45/50 (90%)
#   ✅ AC-PLT-001: Platform status endpoint
#   ❌ AC-PLT-002: Platform graph endpoint (NO TESTS)

# Just BDD scenarios
cargo xtask bdd

# BDD tests for specific AC
cargo test -p acceptance --test acceptance -- --include-tag AC-PLT-001

# Just policies
cargo xtask policy-test

# Just graph
cargo xtask graph-export --check-invariants
```

### AC Coverage Validation Workflow

**Before claiming work is complete:**

```bash
# Step 1: Check coverage
cargo xtask ac-coverage

# Step 2: Interpret results
# ✅ = AC has tests wired (good)
# ❌ = AC has no tests (must fix)

# Step 3: For any ❌ ACs, generate scenarios
cargo xtask ac-suggest-scenarios AC-PLT-XXX

# Step 4: Wire up tests
# - Add scenario to specs/features/*.feature
# - Implement step definitions in crates/acceptance/src/steps/

# Step 5: Verify
cargo test -p acceptance --test acceptance -- --include-tag AC-PLT-XXX

# Step 6: Re-check coverage
cargo xtask ac-coverage | grep AC-PLT-XXX
# Should now show ✅
```

**Golden Rule for Kernel ACs:**
- All kernel ACs (marked with `kernel: true` in spec_ledger.yaml) **MUST** show ✅ in `ac-coverage`
- If any kernel AC shows ❌, the work is not complete
- Use `ac-suggest-scenarios` to generate BDD stubs rather than guessing scenario structure

---

## 5. Common Patterns

### Pattern: Implementing a New Endpoint

```bash
# 1. Discover task
cargo xtask tasks-list | grep endpoint

# 2. Get guidance
cargo xtask suggest-next --task add_endpoint

# 3. Follow sequence:
#    - Create AC
#    - Add BDD scenario
#    - Bundle context
#    - Implement handler
#    - Run tests
#    - Selftest
```

### Pattern: Fixing a Security Vulnerability

```bash
# 1. Run audit
cargo xtask audit
# Identifies CVE-XXXX in dependency Y

# 2. Get guidance
cargo xtask suggest-next --task fix_audit

# 3. Document decision
cargo xtask adr-new "Upgrade dependency Y to fix CVE-XXXX"

# 4. Make change
cargo update -p dependency-y

# 5. Verify
cargo xtask audit  # Should pass
cargo xtask selftest  # Ensures nothing broke
```

### Pattern: Adding a Design Doc

```bash
# 1. Create doc
cargo xtask design-new REQ-TPL-DATABASE "Database Schema Design"

# 2. Edit generated file
#    docs/design/database-schema.md

# 3. Validate
cargo xtask docs-check
cargo xtask selftest
```

---

## 6. Reading the Specs (Bounded Context)

When you need deep context, **don't read the entire codebase**. Use bounded contexts:

### For a Specific Task

```bash
cargo xtask bundle <task_id>
# Output: .llm/bundle/<task_id>.md
# Contains: Only files relevant to that task (max 250KB)
```

**What's included:**
- `spec_ledger.yaml` (requirements/ACs)
- `devex_flows.yaml` (workflows)
- `tasks.yaml` (task definition)
- Relevant design docs
- Relevant code files (based on `.llm/contextpack.yaml` include patterns)

**What's excluded:**
- Test files (unless specifically included)
- Generated code
- Dependencies (`target/`, `node_modules/`)

### For Understanding Architecture

Don't parse code ASTs. Instead:

1. **Read ADRs:**
   ```bash
   ls docs/adr/
   # Each ADR answers: What decision? Why? Consequences?
   ```

2. **Read design docs:**
   ```bash
   curl http://localhost:8080/platform/docs/index | jq '.docs[] | select(.type=="design_doc")'
   ```

3. **Inspect graph:**
   ```bash
   curl http://localhost:8080/platform/graph | jq
   ```

---

## 7. Error Handling

### When Selftest Fails

**Step-by-Step Diagnosis:**

1. **Identify which step failed:**
   ```
   [5/7] Running policy tests...
     ✗ Policy tests failed
   ```

2. **Run that step in isolation:**
   ```bash
   cargo xtask policy-test
   ```

3. **Read the error output:**
   ```
   Ledger Policy:
     ✗ ledger_valid.json (expected pass, got fail)
     
   FAIL - AC-TPL-001 has no tests
   ```

4. **Fix the root cause:**
   - In this case: Add `tests: [{ type: bdd, tag: "@AC-TPL-001" }]` to AC in `spec_ledger.yaml`

5. **Re-run selftest:**
   ```bash
   cargo xtask selftest
   ```

### When Policy Test Fails

Policies are **compliance gates**. Failures mean the spec/code violates a governance rule.

**Common failures:**

| Policy | Violation | Fix |
|--------|-----------|-----|
| `ledger` | AC has no tests | Add `tests` array to AC |
| `template_core` | Requirement lacks AC | Add AC or remove `must_have_ac: true` |
| `features` | Unknown AC referenced | Fix typo in feature file tag |
| `privacy` | Literal secret in code | Move to env var/secret |
| `k8s` | Container runs as root | Add `securityContext.runAsNonRoot: true` |

### When Graph Invariants Fail

Graph failures mean **structural integrity is broken**.

**Common violations:**

| Code | Meaning | Fix |
|------|---------|-----|
| `REQ_HAS_NO_AC` | Requirement with `must_have_ac: true` has no ACs | Add AC or set `must_have_ac: false` |
| `COMMAND_UNREACHABLE` | Required command not in any flow | Add command to a flow in `devex_flows.yaml` |

---

## 8. Boundaries and Constraints

### What You Should Do Autonomously

- ✅ Implement ACs with clear specs
- ✅ Fix failing tests
- ✅ Update docs to match code
- ✅ Add BDD scenarios for new behaviors (use `ac-suggest-scenarios` for scaffolding)
- ✅ Run `ac-coverage` to verify all kernel ACs have tests
- ✅ Run `selftest` and fix violations

### What Requires Human Approval

- ⚠️ **Changing graph invariants** (e.g., removing `must_have_ac`)
- ⚠️ **Changing policy rules** (`.rego` files)
- ⚠️ **Changing DevEx flows** (workflow changes affect all developers)
- ⚠️ **Architectural decisions** (should be captured as ADR first)
- ⚠️ **Security-sensitive changes** (auth, crypto, secrets handling)

### What You Should Never Do

- ❌ **Bypass selftest** (force merge when it fails)
- ❌ **Claim work is complete without running `ac-coverage`** (kernel ACs must be green)
- ❌ **Edit specs without using `xtask` generators** (leads to ID collisions, schema errors)
- ❌ **Duplicate AC/REQ IDs** (breaks graph integrity)
- ❌ **Reference non-existent ADRs** (breaks traceability)
- ❌ **Hardcode secrets** (violates privacy policies)
- ❌ **Hand-write BDD scenarios without checking `ac-suggest-scenarios`** (leads to inconsistent structure)

---

## 9. Debugging the Agent Interface Itself

If the platform APIs return unexpected data:

### Verify Specs Load

```bash
# Try loading each spec manually
cargo run -p spec-runtime --example load_ledger
cargo run -p spec-runtime --example load_devex
cargo run -p spec-runtime --example load_tasks
```

If parsing fails, the YAML has schema errors.

### Verify API Server is Running

```bash
# Start server
cargo run -p app-http

# Health check
curl http://localhost:8080/health

# Platform status
curl http://localhost:8080/platform/status
```

If APIs return 404, routes may not be wired correctly in `app-http/src/main.rs`.

### Verify Graph Builds

```bash
cargo xtask graph-export --format json > /tmp/graph.json
jq '.nodes | length' /tmp/graph.json
jq '.edges | length' /tmp/graph.json
```

If node/edge counts are zero, graph construction failed.

---

## 10. Summary: Agent Operating Model

**Your workflow as an agent:**

1. **Poll for work:** `GET /platform/tasks` or `cargo xtask tasks-list`
2. **Get guidance:** `GET /platform/tasks/suggest-next?task=X`
3. **Execute steps:** Run `cargo xtask` commands as specified
4. **Validate:** Run `cargo xtask selftest` after each atomic unit of work
5. **Report status:** Summarize what passed/failed, ask human for guidance on failures you can't auto-resolve

**Your contract:**
- You are **bounded** by the specs and policies
- You **must** pass selftest before claiming work is done
- You **should** ask for approval on high-risk tasks
- You **can** proceed autonomously on low-risk, spec-driven work

---

## 11. Quick Reference

### Essential Commands

```bash
# Discovery
cargo xtask tasks-list
cargo xtask suggest-next --task <id>
cargo xtask help-flows
cargo xtask ac-coverage              # Check AC test coverage
cargo xtask ac-coverage | grep FEATURE-NAME

# Execution
cargo xtask ac-new <id> "<desc>" --requirement <req>
cargo xtask ac-suggest-scenarios <ac-id>    # Generate BDD scenario stubs
cargo xtask bundle <task>
cargo xtask bundle implement_ac --ac <ac-id>
cargo xtask bdd
cargo test -p acceptance --test acceptance -- --include-tag <ac-id>
cargo xtask selftest

# Validation
cargo xtask check
cargo xtask ac-coverage              # MUST show ✅ for kernel ACs
cargo xtask policy-test
cargo xtask docs-check
cargo xtask graph-export --check-invariants

# Introspection
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
curl http://localhost:8080/platform/tasks
```

### Environment Variables

- `XTASK_LOW_RESOURCES=1`: Set this if running on a constrained environment (low RAM/CPU) to serialize builds and disable heavy caching.


### Essential URLs

- Dashboard: `http://localhost:8080/ui`
- Graph: `http://localhost:8080/ui/graph`
- Flows: `http://localhost:8080/ui/flows`
- API Docs: `http://localhost:8080/platform/status`

---

## 12. Further Reading

- [CLAUDE.md](../CLAUDE.md) - High-level agent constitution
- [Agent Skills Guide](AGENT_SKILLS.md) - **How to author Skills for this repo** (maps flows → Skills)
- [Technical Overview](explanation/rust-as-spec-overview.md) - Architecture deep-dive
- [ROADMAP.md](ROADMAP.md) - Strategic direction
- [DevEx Flows Reference](../specs/devex_flows.yaml) - All available workflows
- [Tasks Reference](../specs/tasks.yaml) - All available tasks
- [xtask Commands Reference](reference/xtask-commands.md) - Complete command documentation

