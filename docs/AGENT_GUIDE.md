---
id: GUIDE-TPL-AGENT-001
title: Agent Guide - Operating the Rust-as-Spec Platform Cell
doc_type: guide
status: published
audience: agents, developers
tags: [agent, llm, automation, platform, governance]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: [ADR-0005]
last_updated: 2025-12-22
---
<!-- doclint:disable orphan-version -->
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

**Visualize Task Dependencies:**
```bash
# Get task dependency graph as JSON
curl http://localhost:8080/platform/tasks/graph

# Response includes:
# {
#   "nodes": [
#     {
#       "id": "TASK-001",
#       "title": "Task title",
#       "status": "InProgress",
#       "requirement": "REQ-001",
#       "owner": "agent",
#       "labels": ["platform"]
#     }
#   ],
#   "edges": [
#     {
#       "from": "TASK-002",    // Task that depends
#       "to": "TASK-001",      // Task being depended on
#       "edge_type": "depends_on"
#     }
#   ],
#   "blocking_relationships": [
#     {
#       "blocked_task": "TASK-002",
#       "blocking_tasks": ["TASK-001"],
#       "reason": "Task 'TASK-002' is blocked by 1 incomplete dependencies"
#     }
#   ]
# }

# Get Mermaid diagram (for visualization)
curl 'http://localhost:8080/platform/tasks/graph?format=mermaid'

# Response includes:
# {
#   "mermaid": "graph TD\n  TASK-001[Task title]:::inprogress\n  ..."
# }
```

**Use cases for task graph:**
- Identify which tasks are blocked by incomplete dependencies
- Visualize the task dependency tree using Mermaid.js
- Understand the critical path for completing a feature
- Find which tasks can be worked on in parallel
- Detect tasks that are blocking multiple other tasks (high priority)

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
      "title": "Implement CLI governance status dashboard",
      "status": "InProgress",
      "owner": "agent",
      "labels": ["platform", "devex", "observability"],
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

**Fields explained:**
- `task_id`: Unique identifier for the task
- `title`: Human-readable task title from tasks.yaml
- `status`: Current task status (Todo, InProgress, Review, Done)
- `owner`: Task owner/assignee (individual or team name)
- `labels`: Tags for filtering and categorization (e.g., platform, security, v3)
- `requirement_ids`: List of requirement IDs this task implements
- `ac_ids`: List of acceptance criteria IDs this task satisfies
- `reason`: Short explanation of why this task is ready for work
- `recommended_sequence`: Ordered list of commands to execute for this task

**Filtering and Prioritization:**

The hints endpoint supports query parameters for filtering and automatically sorts results:

```bash
# Filter by owner
curl "http://localhost:8080/platform/agent/hints?owner=alice"

# Filter by label
curl "http://localhost:8080/platform/agent/hints?label=security"

# Filter by requirement
curl "http://localhost:8080/platform/agent/hints?requirement=REQ-TPL-HEALTH"

# Combine filters
curl "http://localhost:8080/platform/agent/hints?owner=alice&label=security"
```

**Sorting behavior:**
1. **Primary**: Status (InProgress tasks appear before Todo tasks)
2. **Secondary**: Priority label (priority:high > priority:medium > priority:low > no priority)
3. **Tertiary**: Task ID (alphabetical)

This ensures you always see:
- Tasks already in progress first
- Within each status group, high-priority tasks first
- Predictable ordering by ID for stability

**Use cases:**
- Start a new session and discover what's already in progress or ready to start
- Filter for `Todo` and `InProgress` tasks automatically
- Find tasks assigned to a specific owner or team
- Identify high-priority work using labels
- Focus on tasks related to a specific requirement
- Use `labels` to identify task category (e.g., security, platform, docs)
- Check `owner` to understand who's responsible for the task
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

### Query Platform Schemas

The `/platform/schema` endpoint provides comprehensive JSON Schema definitions for all YAML configuration files in the platform, enabling validation, code generation, and machine-readable documentation.

**Get All Schemas:**
```bash
# Get comprehensive schema information
curl http://localhost:8080/platform/schema
```

**Response structure:**
```json
{
  "schemas": [
    {
      "name": "spec_ledger",
      "version": "1.0",
      "description": "Story → Requirement → Acceptance Criterion traceability ledger",
      "source_file": "specs/spec_ledger.yaml",
      "json_schema": {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        ...
      }
    },
    {
      "name": "tasks",
      "version": "1.0",
      "description": "Work item tracking and task management",
      "source_file": "specs/tasks.yaml",
      "json_schema": { ... }
    },
    ...
  ],
  "endpoints": [
    {
      "path": "/platform/status",
      "method": "GET",
      "description": "Get platform governance and service status",
      "request_type": null,
      "response_type": "PlatformStatus"
    },
    ...
  ]
}
```

**Get Individual Schema by Name:**
```bash
# Get schema for tasks.yaml
curl http://localhost:8080/platform/schema/tasks

# Get schema for questions
curl http://localhost:8080/platform/schema/questions

# Get schema for config
curl http://localhost:8080/platform/schema/config
```

**Available schemas:**
- `spec_ledger` - Story/Requirement/AC traceability ledger (specs/spec_ledger.yaml)
- `tasks` - Work item tracking and task management (specs/tasks.yaml)
- `questions` - Structured ambiguity artifacts (specs/questions_schema.yaml)
- `devex_flows` - Developer experience workflows and commands (specs/devex_flows.yaml)
- `config` - Service configuration schema (specs/config_schema.yaml)
- `doc_index` - Documentation inventory (specs/doc_index.yaml)
- `service_metadata` - Service identity and metadata (specs/service_metadata.yaml)

**Use cases:**
- **Validation**: Validate YAML files against their JSON Schema before committing
- **Code Generation**: Generate types/structs from JSON Schema for external tooling
- **Documentation**: Auto-generate documentation from schema definitions
- **IDE Integration**: Use JSON Schema for autocomplete and validation in editors
- **Contract Verification**: Ensure YAML files conform to expected structure
- **Tooling Development**: Build external tools that interact with platform specs

**Validation workflow example:**
```bash
# 1. Get the schema
curl http://localhost:8080/platform/schema/tasks > /tmp/tasks.schema.json

# 2. Validate a YAML file using a JSON Schema validator
# (Example using ajv-cli - install with: npm install -g ajv-cli)
yq eval -o=json specs/tasks.yaml | \
  ajv validate -s /tmp/tasks.schema.json -d -

# 3. If valid, proceed with changes
# If invalid, fix the YAML to conform to the schema
```

**Integration with platform flows:**
- Schemas are auto-generated from the same runtime types used by xtask commands
- Changes to spec structure are reflected immediately in `/platform/schema`
- All platform endpoints are documented in the `endpoints` array
- JSON Schema format enables cross-language tooling and validation

### Check Open Questions

Questions are structured artifacts created when flows or agents encounter ambiguity that requires human decision-making. They prevent work from stalling while capturing the context and options for later resolution.

**CLI - View Questions:**
```bash
# View all questions with counts
cargo xtask status

# Output shows:
# Questions:
#   Open:        2
#   Answered:    1
#   Resolved:    1
#   Total:       4
#
#   ⚠️  Q-BUNDLE-001
#     Bundle flow found multiple ACs - unclear which to prioritize
#   ⚠️  Q-SUGGEST-002
#     Task has circular dependency in workflow
```

**API - Query Questions:**
```bash
# Get question counts and top open questions
curl http://localhost:8080/platform/status

# Response includes:
# {
#   "governance": {
#     "questions": {
#       "open": 2,
#       "answered": 1,
#       "resolved": 1,
#       "total": 4,
#       "top_open": [
#         {
#           "id": "Q-BUNDLE-001",
#           "summary": "Bundle flow found multiple ACs - unclear which to prioritize",
#           "flow": "bundle"
#         }
#       ]
#     }
#   }
# }
```

**Acting on Questions:**
1. Check for open questions in `cargo xtask status` or `/platform/status`
2. Review question files in `questions/` directory for full context
3. Questions include:
   - `options`: Available choices with risk assessment
   - `recommendation`: Agent's suggested option with rationale
   - `context`: Flow/phase where ambiguity occurred, files involved
4. Resolve questions by updating the `status` field to `"answered"` or `"resolved"` after human input
5. Document resolution in the `resolution` section with `chosen_option` and `notes`

**Question States:**
- `open`: Needs human decision
- `answered`: Human provided input but implementation pending
- `resolved`: Fully resolved and implemented
- `obsolete`: No longer relevant (e.g., requirement changed)

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

`cargo xtask selftest` is the **supreme arbiter** of correctness. It runs 8 steps:

1. **Core checks** (fmt, clippy, tests) - Code quality
2. **BDD** - Behavior matches specs
3. **AC mapping** - Traceability (ACs have tests, ADRs exist)
4. **LLM bundler** - Context generation works
5. **Policy tests** - Compliance (OPA/Rego)
6. **DevEx contract** - Required commands exist
7. **Graph invariants** - Structural integrity (no orphans, missing ACs)
8. **AC coverage** - All kernel and non-kernel ACs are passing

**What failure means:**
- Step 1-2: Code doesn't work
- Step 3: Metadata is broken
- Step 4: Agent infrastructure is broken
- Step 5-8: **Governance is violated** (cannot merge)

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

### Selftest and AC Status (for Agents)

Agents should treat `cargo xtask selftest` and the AC status as the **truth** about the workspace.

**Key commands:**

```bash
cargo xtask selftest          # 8 governance gates
cargo xtask ac-status         # Human-readable AC table
cargo xtask ac-status --json  # Machine-readable AC table
```

**How to interpret selftest:**

* If `selftest` is **green**:

  * All kernel ACs (`must_have_ac: true`) are passing.
  * Template ACs are either passing or explicitly marked as template/meta.
  * Agents can assume the platform APIs (`/platform/*`) and DevEx commands behave as documented.

* If `selftest` is **red**:

  * The failure will be associated with one of the 8 gates:

    * Core checks, BDD, AC/ADR mapping, Bundler, Policy tests,
      DevEx contract, Graph invariants, AC coverage.
  * Agents should:

    1. Call `GET /platform/status` and inspect the `governance` section.
    2. Call `GET /platform/graph` to see which ACs/REQs/tests are failing.
    3. Prefer **fixing** the underlying issue (or filing a question/friction artifact)
       rather than working around it.

**How to interpret AC status JSON:**

The JSON from:

```bash
cargo xtask ac-status --json
```

has, for each AC:

* `status`: `"pass" | "fail" | "unknown"`
* `tags`: classification (`kernel`, `template`, `philosophy`, `governance`, `harness`, etc.)
* `must_have_ac`: `true` for kernel ACs
* `tests_total` / `tests_executed`: how much coverage exists

Recommended agent behaviour:

* Treat `status == "pass" && must_have_ac == true` as "safe to rely on".
* Treat `status == "fail"` as "do not trust this behaviour" and either:

  * avoid using it, or
  * surface a question via `question-new` / `/platform/questions`.
* Treat `status == "unknown"` as:

  * OK if tags include `harness` or `example` (meta/CI-only),
  * otherwise: suspect, worth surfacing as a question.

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

## 4.5. When to Use: Friction Log vs ADR vs Issue vs Question

Understanding when to use each artifact type is critical for effective governance.

### Friction Log

**Use the friction log for:**
- Process or tooling problems
- Developer experience pain points
- Workflow inefficiencies
- CI/CD issues
- Flaky tests or intermittent failures
- Poor error messages or unclear diagnostics
- Missing or unclear documentation

**Examples:**
- "Port discovery requires manual lsof lookup"
- "Selftest takes 2+ hours on Windows"
- "Pre-commit hook intermittently fails with file locking"
- "Bundle command slow on large repos"
- "Error message doesn't explain which file is invalid"

**Commands:**
```bash
# List friction entries
cargo xtask friction-list
cargo xtask friction-list --status open
cargo xtask friction-list --severity high

# Create new friction entry
cargo xtask friction-new --category devex --severity medium --summary "Description of the issue"

# Resolve a friction entry
cargo xtask friction-resolve --id FRICTION-TOOL-001 --resolved-by agent \
  --fix-description "Fixed by updating the config" --pr "#123"

# Create GitHub issue from friction entry
cargo xtask friction-gh-create FRICTION-TOOL-001
cargo xtask friction-gh-create FRICTION-TOOL-001 --dry-run  # Preview without creating
cargo xtask friction-gh-create FRICTION-TOOL-001 --open     # Open in browser after creation
cargo xtask friction-gh-create FRICTION-TOOL-001 --labels "team:platform,sprint:q1"

# Link existing GitHub issue to friction entry
cargo xtask friction-gh-link FRICTION-TOOL-001 123  # Links to issue #123
cargo xtask friction-gh-link FRICTION-TOOL-001 "#123"  # Also accepts # prefix
```

**API:**
```bash
# Get friction summary from status
curl http://localhost:8080/platform/status | jq '.governance.friction'

# Get all friction entries
curl http://localhost:8080/platform/friction

# Get specific friction entry
curl http://localhost:8080/platform/friction/FRICTION-AGENT-001
```

**Response structure (GET /platform/friction):**
```json
{
  "entries": [
    {
      "id": "FRICTION-AGENT-001",
      "date": "2025-11-20",
      "category": "api",
      "severity": "high",
      "summary": "UI/API inconsistency...",
      "description": "...",
      "status": "resolved",
      "context": { ... },
      "resolution": { ... }
    }
  ],
  "total": 2
}
```

### ADR (Architecture Decision Record)

**Use ADRs for:**
- Architectural decisions
- Technology choices
- Design patterns
- System boundaries
- Trade-offs between technical alternatives

**Examples:**
- "Why we use Axum instead of Actix"
- "How we structure the governance graph"
- "Why BDD tags must match AC IDs"
- "Decision to use YAML for specs instead of JSON"

**Command:**
```bash
cargo xtask adr-new "Title of architectural decision"
```

### GitHub Issue

**Use GitHub issues for:**
- Feature requests
- Bug reports (functional bugs, not process friction)
- Work items requiring tracking
- Public discussion needed
- External stakeholder visibility

**Examples:**
- "Add support for GraphQL API"
- "Task filtering returns wrong results"
- "Implement multi-tenancy"
- "Agent should support multiple simultaneous tasks"

### Question

**Use questions for:**
- Ambiguity in specs or requirements
- Unclear choices during flow execution
- Missing information needed to proceed
- Multiple valid interpretations of a requirement

**Examples:**
- "Bundle flow finds multiple ACs - which to prioritize?"
- "Task has circular dependency in workflow"
- "AC description ambiguous - multiple interpretations"

**Commands:**
```bash
# Show questions summary
cargo xtask status  # Shows open questions

# List all questions
cargo xtask questions-list
cargo xtask questions-list --status open
cargo xtask questions-list --json

# Create a new question
cargo xtask question-new --category BUNDLE --summary "Multiple ACs found" \
  --flow bundle --phase ac_selection \
  --description "Found 3 ACs for the same requirement, unclear priority"

# Create a question with task/ref linkage
cargo xtask question-new --category TPL --summary "Unclear behavior" \
  --flow implement_ac --phase implementation \
  --description "Spec is ambiguous about error handling" \
  --task-id TASK-001 --refs REQ-TPL-001 --refs AC-TPL-001

# Resolve a question
cargo xtask question-resolve --id Q-BUNDLE-001 --resolved-by agent \
  --chosen-option "Option A" --notes "Chose based on risk assessment"

# Mark question as obsolete
cargo xtask question-resolve --id Q-BUNDLE-002 --resolved-by human \
  --status obsolete --notes "Requirement was removed"
```

**API:**
```bash
# Get question summary from status
curl http://localhost:8080/platform/status | jq '.governance.questions'

# Get all questions
curl http://localhost:8080/platform/questions

# Filter questions by status
curl "http://localhost:8080/platform/questions?status=open"

# Get specific question
curl http://localhost:8080/platform/questions/Q-EXAMPLE-001
```

**Response structure (GET /platform/questions):**
```json
{
  "questions": [
    {
      "id": "Q-EXAMPLE-001",
      "summary": "Bundle flow found multiple ACs for the same requirement - unclear which to prioritize",
      "status": "open",
      "flow": "bundle",
      "phase": "ac_selection",
      "created_at": "2025-11-26T00:00:00Z"
    }
  ],
  "total": 1
}
```

**Response structure (GET /platform/questions/{id}):**
```json
{
  "id": "Q-EXAMPLE-001",
  "task_id": "implement_ac",
  "req_ids": ["REQ-TPL-SUGGEST-NEXT"],
  "ac_ids": ["AC-TPL-SUGGEST-NEXT-CLI"],
  "summary": "Bundle flow found multiple ACs...",
  "context": {
    "flow": "bundle",
    "phase": "ac_selection",
    "description": "...",
    "files_involved": ["specs/spec_ledger.yaml", "specs/tasks.yaml"]
  },
  "options": [
    {
      "label": "Implement AC-001 first (foundational)",
      "description": "...",
      "risk": "low",
      "reversible": true
    }
  ],
  "recommendation": {
    "option_label": "Implement AC-001 first (foundational)",
    "rationale": "...",
    "confidence": "medium"
  },
  "created_by": "flow",
  "created_at": "2025-11-26T00:00:00Z",
  "status": "open"
}
```

### Unified Issue Search

The `issues-search` command provides unified search across friction entries, questions, and tasks. This is useful when you need to find related issues across different artifact types.

**Use unified search for:**
- Finding all issues related to a specific REQ or AC
- Searching by keyword across all governance artifacts
- Discovering related friction entries and questions for a task
- Quick lookup by ID without knowing the artifact type

**Examples:**
```bash
# Search across all artifact types by keyword
cargo xtask issues-search "bundle"

# Search only friction entries
cargo xtask issues-search "port discovery" --type friction

# Search only questions
cargo xtask issues-search "ambiguous" --type question

# Search only tasks
cargo xtask issues-search "platform" --type task

# Filter by status
cargo xtask issues-search "config" --status open

# Filter by REQ/AC reference
cargo xtask issues-search "" --refs REQ-TPL-001

# Combine filters
cargo xtask issues-search "api" --type friction --status open --refs AC-PLT-001

# JSON output for programmatic use
cargo xtask issues-search "bundle" --json

# Limit results
cargo xtask issues-search "test" --limit 10
```

**Search behavior:**
- Results are ranked by relevance (ID match > summary > description > category/labels)
- Exact ID matches receive a bonus score
- Empty query with `--refs` filter finds all issues linked to that REQ/AC
- Default limit is 50 results

**Response structure (--json):**
```json
{
  "query": "bundle",
  "total_results": 3,
  "results": [
    {
      "issue_type": "friction",
      "id": "FRICTION-BUNDLE-001",
      "summary": "Bundle command slow on large repos",
      "status": "open",
      "refs": ["REQ-TPL-BUNDLE"],
      "date": "2025-01-01",
      "relevance_score": 15.0
    }
  ]
}
```

### Fork Registry

**Use fork registry for:**
- Tracking known forks of this template
- Recording which kernel versions forks are based on
- Connecting fork maintainers for collaboration
- Identifying patterns for kernel backports

**Examples:**
- "Knowledge Hub fork for ML documentation platform"
- "SDK template fork for Rust client libraries"
- "Compliance fork for regulated industries"

**Command:**
```bash
cargo xtask fork-list  # List all registered forks
```

**API:**
```bash
# Get all forks
curl http://localhost:8080/platform/forks

# Get specific fork
curl http://localhost:8080/platform/forks/FORK-KHUB-001
```

**Response structure (GET /platform/forks):**
```json
{
  "forks": [
    {
      "id": "FORK-KHUB-001",
      "name": "Knowledge Hub",
      "domain": "ml-documentation",
      "status": "active",
      "kernel_version": "v3.3.9-kernel"
    }
  ],
  "total": 1
}
```

**Response structure (GET /platform/forks/{name}):**
```json
{
  "id": "FORK-KHUB-001",
  "name": "Knowledge Hub",
  "domain": "ml-documentation",
  "kernel_version": "v3.3.9-kernel",
  "status": "active",
  "url": "https://github.com/org/knowledge-hub",
  "maintainer": {
    "name": "ML Team",
    "contact": "ml-team@example.com"
  },
  "forked_at": "2025-11-01",
  "last_synced": "2025-11-20",
  "features": [
    "GraphQL API integration",
    "Extended platform endpoints for ML workflows"
  ],
  "pain_points": [
    "FRICTION-BUNDLE-001"
  ],
  "notes": "Production ML documentation platform...",
  "related_items": {
    "issues": ["#123"],
    "adrs": ["ADR-012"],
    "friction": ["FRICTION-BUNDLE-001"]
  }
}
```

### Quick Reference Table

| Artifact Type | Purpose | Visibility | Lifecycle |
|--------------|---------|------------|-----------|
| **Friction Log** | Process/tooling pain | Internal (repo) | Open → Resolved |
| **ADR** | Architectural decisions | Internal (repo) | Permanent record |
| **GitHub Issue** | Feature work, bugs | Public (GitHub) | Open → Closed |
| **Question** | Spec ambiguity | Internal (repo) | Open → Answered → Resolved |
| **Fork Registry** | Template fork tracking | Internal (repo) | Active → Archived |

### Decision Flow

1. **Is it about the development process or tools?** → Use **Friction Log**
2. **Is it an architectural or design decision?** → Use **ADR**
3. **Is it unclear what the spec means or requires?** → Use **Question**
4. **Is it a feature request or functional bug?** → Use **GitHub Issue**

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
cargo xtask version                  # Show kernel version

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

# Governance Artifacts
cargo xtask friction-new --category X --severity Y --summary "..."
cargo xtask friction-list
cargo xtask friction-resolve --id FRICTION-ID --resolved-by agent --fix-description "..."
cargo xtask friction-gh-create FRICTION-ID          # Create GitHub issue from friction
cargo xtask friction-gh-link FRICTION-ID 123        # Link to existing GitHub issue
cargo xtask question-new --category X --summary "..." --flow F --phase P --description "..."
cargo xtask questions-list
cargo xtask question-resolve --id Q-ID --resolved-by agent --chosen-option "A" --notes "..."
cargo xtask issues-search "query"                   # Unified search across friction/questions/tasks
cargo xtask issues-search "query" --type friction --status open --refs REQ-ID
cargo xtask fork-register --name "Name" --domain "domain" --kernel-version "v3.3.9-kernel" ...
cargo xtask fork-list

# Introspection (see docs/reference/platform-api-endpoints.md for full reference)
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
curl http://localhost:8080/platform/openapi                  # OpenAPI spec (YAML)
curl http://localhost:8080/platform/schema                   # All JSON schemas
curl http://localhost:8080/platform/schema/tasks             # Specific schema by name
curl http://localhost:8080/platform/devex/flows              # DevEx commands and flows
curl http://localhost:8080/platform/docs/index               # Documentation inventory
curl http://localhost:8080/platform/coverage                 # AC coverage summary
curl http://localhost:8080/platform/tasks
curl http://localhost:8080/platform/tasks/suggest-next?task=TASK-ID  # Next steps for task
curl http://localhost:8080/platform/tasks/graph              # Task dependency graph (JSON)
curl 'http://localhost:8080/platform/tasks/graph?format=mermaid'  # Mermaid diagram
curl http://localhost:8080/platform/agent/hints              # Prioritized work for agents
curl http://localhost:8080/platform/friction                 # Friction log entries
curl http://localhost:8080/platform/questions                # Question artifacts
curl "http://localhost:8080/platform/questions?status=open"  # Filter by status
curl http://localhost:8080/platform/forks                    # Fork registry
curl http://localhost:8080/platform/issues                   # Unified issues (friction+questions+tasks)
curl http://localhost:8080/platform/idp/snapshot             # IDP snapshot for dashboards
curl http://localhost:8080/platform/ui/contract              # UI contract specification
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

