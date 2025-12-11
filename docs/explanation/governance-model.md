---
id: EXPLANATION-GOVERNANCE-MODEL
doc_type: explanation
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-BDD-HARNESS, REQ-PLT-DOCS-CONSISTENCY, REQ-TPL-GRAPH-INVARIANTS]
acs: [AC-TPL-SELFTEST-GATE, AC-TPL-PLATFORM-GRAPH]
adrs: [ADR-0003, ADR-0005, ADR-0020, ADR-0021, ADR-0024]
---
# Governance Model: How Rust-as-Spec Platform Cells Stay Honest

**Template Version:** v3.3.8
**Last Updated:** 2025-12-10

> **TL;DR:** This template uses specs, BDD tests, policies, and graph invariants to create a **self-governing platform cell** where code, docs, and contracts must agree — and the repo can prove it via `cargo xtask selftest`.

---

## 1. The Core Idea

In traditional repos:
- Specs drift from code
- Tests drift from requirements
- Docs drift from reality
- Nobody knows what's "true"

**In this template:**
- **Specs are code** — `spec_ledger.yaml` defines Stories → Requirements → ACs → Tests
- **Tests validate specs** — BDD scenarios tagged with AC IDs prove behavior
- **Docs reference specs** — Design docs link REQ/AC IDs for traceability
- **CI enforces everything** — `cargo xtask selftest` validates the entire graph

The result: **You can query the runtime for the same truth CI enforces.**

---

## 2. The Specs-as-Code Hierarchy

The governance model is built on a strict hierarchy that flows from user value down to automated verification:

```
Stories (US-*)
  → Requirements (REQ-*)
    → Acceptance Criteria (AC-*)
      → Tests (BDD + Unit + Integration)
        → Documentation (ADRs, Design Docs, How-Tos)
          → Commands (xtask)
```

### 2.1 Stories (User Value)

**What:** High-level user goals or platform capabilities

**Example:**
```yaml
- id: US-TPL-001
  title: "Service Core Capabilities"
  adr: ADR-0001  # Links to architectural decision
```

**Purpose:**
- Capture the "why" — what user value drives this work
- Group related requirements
- Provide context for agents and developers

### 2.2 Requirements (What Must Be True)

**What:** Concrete system properties or behaviors that satisfy a story

**Example:**
```yaml
- id: REQ-TPL-HEALTH
  title: "Health Check Endpoint"
  tags: [platform, structural]
  must_have_ac: true  # Kernel contract flag
  adr: ADR-0003
```

**Key Properties:**
- `tags` — Classify requirement type (platform, security, devex, docs, release)
- `must_have_ac` — If `true`, this REQ **must** have at least one AC (enforced by graph invariants)
- `adr` — Links to architectural decision record explaining rationale

**Purpose:**
- Define verifiable system properties
- Enable traceability from user value to tests
- Support filtering and search (by tag, status, etc.)

### 2.3 Acceptance Criteria (Concrete Behavior)

**What:** Precise, testable statements about system behavior

**Example:**
```yaml
acceptance_criteria:
  - id: AC-TPL-001
    text: "GET /health returns 200 with status 'ok' when service is healthy"
    tags: [kernel]  # Part of kernel contract
    must_have_ac: true  # Must pass for merge
    tests:
      - type: bdd
        tag: "@AC-TPL-005"
        file: "specs/features/template_core.feature"
```

**Key Properties:**
- `id` — Unique identifier (never reused)
- `text` — Human-readable behavior description
- `tags` — `[kernel]` marks template contracts; `[future]` marks planned work
- `must_have_ac` — If `true`, test must pass in `cargo xtask selftest`
- `tests` — Array linking to BDD scenarios, unit tests, or integration tests

**Purpose:**
- Define testable success criteria
- Enable AC-focused development (implement AC → test AC → validate AC)
- Support selective testing (`cargo xtask test-ac AC-TPL-001`)

### 2.4 Tests (Automated Verification)

**What:** Executable code that proves an AC is satisfied

**BDD Example:**
```gherkin
@AC-TPL-001
Scenario: Health endpoint returns OK when service is healthy
  Given the service is running
  When I GET /health
  Then the response status should be 200
  And the response body should contain "ok"
```

**Unit Test Example:**
```rust
/// AC-TPL-CONFIG-VALIDATION: Validates that the service rejects invalid
/// configuration at startup and exits with a clear error message.
#[test]
fn config_validation_rejects_invalid() {
    // Test implementation
}
```

**Test Types:**
- `bdd` — Cucumber/Gherkin scenarios (behavioral)
- `unit` — Rust `#[test]` functions (structural)
- `integration` — Multi-component tests
- `manual` — Human-verified (documented in spec only)

**Purpose:**
- Prove AC is satisfied
- Enable regression detection
- Support CI validation

### 2.5 Documentation (Context and Rationale)

**What:** Human-readable explanations linking back to specs

**Design Doc Example:**
```yaml
---
id: DESIGN-PLT-GRAPH-001
doc_type: design_doc
requirements: [REQ-TPL-GRAPH-INVARIANTS]
acs: [AC-TPL-GRAPH-REQ-HAS-AC, AC-TPL-GRAPH-AC-HAS-TEST]
---

# Graph Invariants Design

This document explains the structural integrity rules...
```

**ADR Example:**
```yaml
---
id: ADR-0005
title: "Selftest as the Single Quality Gate"
status: accepted
requirements: [REQ-PLT-SELFTEST]
---
```

**Purpose:**
- Explain "why" behind decisions
- Provide implementation context
- Support onboarding and knowledge transfer

### 2.6 Commands (Developer Actions)

**What:** `cargo xtask` commands that execute workflows

**Defined in `specs/devex_flows.yaml`:**
```yaml
commands:
  - id: selftest
    name: selftest
    description: "Full governance check (11 steps)"
    required: true  # Must exist in xtask
```

**Purpose:**
- Standardize developer workflows
- Enable agent automation
- Provide CLI interface to governance

**Enforcement:**
- `cargo xtask selftest` step 8 validates all `required: true` commands exist
- Graph invariants check that no commands are orphaned (unused in flows)

---

## 3. The Governance Graph

The governance graph is the **runtime representation** of the specs hierarchy. It's built at startup and exposed via APIs.

### 3.1 Graph Structure

**Nodes:**
- `story` — User stories (US-*)
- `requirement` — Requirements (REQ-*)
- `ac` — Acceptance criteria (AC-*)
- `test` — Test cases (from AC `tests` array)
- `doc` — Documentation (design docs, ADRs)
- `command` — xtask commands

**Edges:**
- `has_requirement` — Story → Requirement
- `has_ac` — Requirement → AC
- `tested_by` — AC → Test
- `documented_by` — Requirement → Doc
- `mentioned_in` — Command → Flow

**Example:**
```json
{
  "nodes": [
    {"id": "US-TPL-001", "type": "story", "label": "Service Core Capabilities"},
    {"id": "REQ-TPL-HEALTH", "type": "requirement", "label": "Health Check Endpoint"},
    {"id": "AC-TPL-001", "type": "ac", "label": "GET /health returns 200..."},
    {"id": "AC-TPL-001:test:0", "type": "test", "label": "@AC-TPL-005", "url": "file://specs/features/template_core.feature"}
  ],
  "edges": [
    {"source": "US-TPL-001", "target": "REQ-TPL-HEALTH", "type": "has_requirement"},
    {"source": "REQ-TPL-HEALTH", "target": "AC-TPL-001", "type": "has_ac"},
    {"source": "AC-TPL-001", "target": "AC-TPL-001:test:0", "type": "tested_by"}
  ]
}
```

### 3.2 Graph Invariants

**The graph enforces structural integrity rules:**

1. **REQ_HAS_AC** — Every requirement with `must_have_ac: true` has at least one AC
2. **AC_HAS_TEST** — Every AC with `tests` array has at least one test node
3. **COMMAND_REACHABLE** — Every required command is used in at least one flow
4. **NO_ORPHAN_DOCS** — Every design doc references at least one REQ/AC
5. **ADR_EXISTS** — Every ADR reference in specs exists as a file

**Enforcement:**
```bash
# Validate graph invariants
cargo xtask graph-export --check-invariants

# Included in selftest step 9
cargo xtask selftest
```

**Why invariants matter:**
- Prevent broken references (e.g., AC-TPL-999 doesn't exist)
- Catch spec drift (e.g., requirement has no ACs)
- Ensure documentation traceability

### 3.3 Querying the Graph

**CLI:**
```bash
# Export graph as JSON
cargo xtask graph-export --format json > graph.json

# Export as Mermaid diagram
cargo xtask graph-export --format mermaid > graph.mmd
```

**HTTP API:**
```bash
# Get full graph
curl http://localhost:8080/platform/graph

# Visualize in browser
open http://localhost:8080/ui/graph
```

**Use Cases:**
- Find which ACs satisfy a requirement
- Find which tests validate an AC
- Find which docs explain a requirement
- Identify gaps (requirements without tests)

---

## 4. The Validation Ladder

The validation ladder is a **progressive validation strategy** — start fast, escalate as needed.

### 4.1 Level 1: Local Fast Checks

**Purpose:** Quick feedback during development (seconds)

```bash
# Format + clippy + unit tests
cargo xtask check

# Only changed code
cargo xtask test-changed
```

**What it validates:**
- Code compiles
- Clippy rules pass
- Unit tests pass
- Formatting is correct

**When to use:**
- After every code change
- Before committing

### 4.2 Level 2: AC-Focused Testing

**Purpose:** Validate specific AC changes (seconds to minutes)

```bash
# Test single AC
cargo xtask test-ac AC-TPL-001

# List tests for AC
cargo xtask ac-tests AC-TPL-001
```

**What it validates:**
- BDD scenarios for this AC pass
- Unit tests tagged with this AC pass

**When to use:**
- While implementing a feature
- To verify AC is satisfied

### 4.3 Level 3: AC Status Check

**Purpose:** Validate AC coverage (seconds)

```bash
# Check coverage
cargo xtask ac-status

# Summary view
cargo xtask ac-coverage
```

**What it validates:**
- All kernel ACs (`must_have_ac: true`) have tests
- Tests are passing or failing
- Coverage percentage

**When to use:**
- Before claiming work complete
- To find ACs without tests

### 4.4 Level 4: Full Selftest (Governance Gate)

**Purpose:** Full governance validation (10-20 minutes on Tier-1)

```bash
# Enter hermetic environment
nix develop

# Run all 11 steps
cargo xtask selftest
```

**What it validates:**

1. **Core checks** — fmt, clippy, unit tests
2. **Skills governance** — Skills structure and policies
3. **Agents governance** — Agent definitions and policies
4. **BDD acceptance tests** — All scenarios pass
5. **AC/ADR mapping** — Traceability complete
6. **LLM bundler** — Context generation works
7. **Policy tests** — Conftest/OPA policies pass
8. **DevEx contract** — Required commands exist
9. **Graph invariants** — No broken references
10. **AC coverage** — All kernel ACs passing (configurable)
11. **Test coverage** — Advisory check (non-blocking)

**When to use:**
- Before creating PR
- Before merge (CI enforces)
- When changing specs or governance

**Environment:**
- **Tier-1** (Nix + Linux/macOS/WSL2) — Canonical, hermetic, matches CI
- **Tier-2** (Native Windows) — Informational only, may have file locking issues

**Why Tier-1 matters:**
- Same environment as CI (reproducible)
- No platform-specific issues (hermetic)
- Fast (~10-20 min vs 2+ hours on Windows)

See [ADR-0017: Tier-1 Selftest as Required Gate on Main Branch](../adr/0017-tier1-selftest-gate.md)

### 4.5 Kernel AC Coverage Enforcement (ADR-0024)

**Step 10** of selftest enforces that kernel ACs (`must_have_ac: true`) have test coverage. This gate uses the **AcEvidence model** defined in ADR-0024 to determine AC status from spec mappings and BDD coverage.

**Status Classification Rules:**
1. **FAIL**: Any BDD scenario failed for this AC
2. **PASS**: BDD scenario passed OR unit tests are mapped in spec_ledger.yaml
3. **UNKNOWN**: No test evidence

The enforcement level is configurable via environment variables:

**Environment Variables:**

| Variable | Default | Effect |
|----------|---------|--------|
| `KERNEL_UNKNOWN_BUDGET` | unlimited | Max unknown kernel ACs allowed before selftest fails |
| `XTASK_STRICT_AC_COVERAGE=1` | off | Equivalent to `KERNEL_UNKNOWN_BUDGET=0` (no unknowns allowed) |

**Enforcement Levels:**

1. **Default (no env vars)** — Unknown kernel ACs are advisory (warning only)
   ```bash
   cargo xtask selftest
   # ⚠ 58 kernel ACs have unknown coverage (advisory)
   # Selftest passes
   ```

2. **Budget mode** — Fail if unknown count exceeds budget
   ```bash
   KERNEL_UNKNOWN_BUDGET=50 cargo xtask selftest
   # ❌ Kernel AC coverage gate failed (budget: 50, actual: 58)
   # Selftest fails
   ```

3. **Strict mode** — Zero tolerance for unknowns
   ```bash
   XTASK_STRICT_AC_COVERAGE=1 cargo xtask selftest
   # ❌ Kernel AC coverage gate failed (strict mode)
   # Selftest fails
   ```

**Ratcheting Strategy:**

To progressively improve kernel AC coverage:

1. Start with current count as budget:
   ```bash
   KERNEL_UNKNOWN_BUDGET=58 cargo xtask selftest
   ```

2. As you add tests, lower the budget:
   ```bash
   KERNEL_UNKNOWN_BUDGET=50 cargo xtask selftest
   ```

3. Eventually reach strict mode:
   ```bash
   XTASK_STRICT_AC_COVERAGE=1 cargo xtask selftest
   ```

**CI Configuration:**

In `.github/workflows/tier1-selftest.yml`:
```yaml
# Enable strict mode on main branch
env:
  XTASK_STRICT_AC_COVERAGE: ${{ github.ref == 'refs/heads/main' && '1' || '0' }}
```

**Viewing the Backlog:**

```bash
# List unknown kernel ACs
cargo xtask ac-coverage --todo --must-have

# Generate scenarios for an unknown AC
cargo xtask ac-suggest-scenarios AC-TPL-001
```

---

## 5. Skills and Agents: Governed Workflows

Skills and Agents are **first-class governance artifacts** — they're not just documentation, they're validated by CI.

### 5.1 Skills (Workflow Recipes)

**What:** Structured workflows for common development tasks

**Location:** `.claude/skills/SKILL_NAME/SKILL.md`

**Governance Rules:**
- **Name:** kebab-case, unique, max 64 chars
- **Description:** Includes WHAT (capability) + WHEN (triggers), max 1024 chars
- **Allowed-Tools:** Least-privilege tool list (e.g., `[Read, Grep, Glob]` for read-only)
- **Linked Flows:** References flows from `devex_flows.yaml`
- **No Secrets:** Never hardcode API keys, tokens, or credentials

**Example:**
```yaml
---
name: governed-feature-dev
description: >
  AC-first feature development workflow. Use when implementing features,
  adding ACs, or working on tasks with status=Todo.
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
linked-flows:
  - ac_first
  - validation_ladder
---

# Governed Feature Development Skill

[Workflow steps...]
```

**Validation:**
```bash
# Lint Skills
cargo xtask skills-lint

# Format Skills
cargo xtask skills-fmt
```

**Enforcement:**
- `cargo xtask selftest` step 2
- Pre-commit hook (auto-format)

**Why govern Skills?**
- Prevents "skill explosion" (one skill per command)
- Ensures security (validates tools, detects secrets)
- Maintains consistency (enforces naming, descriptions)

See [SKILLS_GOVERNANCE.md](../SKILLS_GOVERNANCE.md) for details.

### 5.2 Agents (Specialized AI Personas)

**What:** Long-lived, specialized agents with system prompts and tool bindings

**Location:** `.claude/agents/AGENT_NAME.md`

**Governance Rules:**
- **Name:** kebab-case, unique, max 64 chars
- **Description:** Includes WHAT + WHEN, max 1024 chars
- **Tools:** Explicit least-privilege list
- **PermissionMode:** `restricted` (default) or `permissive` (with justification)
- **Model:** `inherit` or explicit model name
- **Skills:** Optional list of skill names to include
- **System Prompt:** Optional, must not contain secrets
- **No Secrets:** Never hardcode API keys, tokens, or credentials

**Example:**
```yaml
---
name: feature-dev-agent
description: >
  Implements features using AC-first workflow. Use when implementing
  new ACs or extending platform capabilities.
tools: [Read, Write, Edit, Bash, Glob, Grep]
permissionMode: restricted
model: inherit
skills:
  - governed-feature-dev
  - governed-maintenance
---

# Feature Development Agent

[System prompt...]
```

**Validation:**
```bash
# Lint Agents
cargo xtask agents-lint

# Format Agents
cargo xtask agents-fmt
```

**Enforcement:**
- `cargo xtask selftest` step 3
- Pre-commit hook (auto-format)

**Why govern Agents?**
- Security (validates tools, permissions, detects secrets)
- Consistency (enforces naming, descriptions, model policy)
- Traceability (agents reference skills and workflows)

See [AGENTS_GOVERNANCE.md](../AGENTS_GOVERNANCE.md) for details.

---

## 6. ADRs, Friction Logs, and Tasks

Beyond specs and tests, the governance model includes artifacts for decisions, feedback, and work tracking.

### 6.1 ADRs (Architecture Decision Records)

**What:** Permanent record of significant technical decisions

**Location:** `docs/adr/NNNN-title.md`

**When to use:**
- Architectural choices (hexagonal architecture, Nix environment)
- Technology decisions (Axum vs Actix, YAML vs JSON)
- Design patterns (error handling, validation strategy)
- Trade-offs between alternatives

**Structure:**
```yaml
---
id: ADR-0005
title: "Selftest as the Single Quality Gate"
status: accepted
date: 2025-11-15
requirements: [REQ-PLT-SELFTEST]
---

## Context
[Why this decision was needed]

## Decision
[What was decided]

## Consequences
[Impact on the system]
```

**Commands:**
```bash
# Create new ADR
cargo xtask adr-new "Title of decision"

# Check ADR references valid
cargo xtask adr-check

# Part of selftest step 5
```

**Purpose:**
- Capture "why" for future maintainers
- Provide context for onboarding
- Enable governance traceability (specs → ADRs)

### 6.2 Friction Logs (DevEx Issues)

**What:** Structured tracking of developer experience pain points

**Location:** `friction/FRICTION-*.yaml` + `FRICTION_LOG.md`

**When to use:**
- Process or tooling problems (slow CI, flaky tests)
- Workflow inefficiencies (port discovery requires manual lsof)
- Poor error messages or unclear diagnostics
- Missing or unclear documentation

**Structure:**
```yaml
id: FRICTION-AGENT-001
date: 2025-11-20
category: api  # api, devex, docs, process
severity: high  # low, medium, high
status: resolved  # open, in_progress, resolved
summary: "UI/API inconsistency..."
description: |
  Detailed explanation...
context:
  trigger: "Agent workflow"
  frequency: "Every bundle generation"
resolution:
  approach: "Added /platform/schema/{name} endpoint"
  validation: "cargo xtask selftest passes"
```

**Commands:**
```bash
# List friction entries
cargo xtask friction-list
cargo xtask friction-list --status open

# Create friction entry
cargo xtask friction-new --category devex --severity medium --summary "Issue"
```

**API:**
```bash
# Get friction summary
curl http://localhost:8080/platform/status | jq '.governance.friction'

# Get all entries
curl http://localhost:8080/platform/friction
```

**Purpose:**
- Surface DevEx issues systematically
- Track resolution progress
- Feed into platform improvements

### 6.3 Tasks (Work Tracking)

**What:** Structured work items linking to REQs and ACs

**Location:** `specs/tasks.yaml`

**Structure:**
```yaml
- id: TASK-TPL-STATUS-CLI-001
  title: "Implement CLI governance status dashboard"
  requirement: REQ-PLT-STATUS-CLI
  acs: [AC-PLT-017]
  status: InProgress  # Todo, InProgress, Review, Done
  owner: agent
  labels: [platform, devex, observability]
```

**Commands:**
```bash
# List tasks
cargo xtask tasks-list

# Create task
cargo xtask task-create --id TASK-001 --title "..." --req REQ-001 --ac AC-001

# Update status
cargo xtask task-update --id TASK-001 --status InProgress
```

**API:**
```bash
# Get tasks
curl http://localhost:8080/platform/tasks

# Filter tasks
curl "http://localhost:8080/platform/tasks?status=Todo"

# Update status
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# Get task dependencies
curl http://localhost:8080/platform/tasks/graph
```

**Purpose:**
- Track work progress
- Provide agent guidance (`/platform/agent/hints`)
- Enable task-focused development (`cargo xtask bundle <task>`)

**State Machine:**
```
Todo → InProgress → Review → Done
```

**Enforcement:**
- `business-core` crate validates transitions
- HTTP API enforces state machine
- CLI rejects invalid transitions

### 6.4 When to Use Which Artifact

| Artifact | Use Case | Visibility | Lifecycle |
|----------|----------|------------|-----------|
| **ADR** | Architectural decisions | Internal | Permanent |
| **Friction Log** | DevEx pain points | Internal | Open → Resolved |
| **Task** | Work tracking | Internal + API | Todo → Done |
| **Question** | Spec ambiguity | Internal + API | Open → Answered |
| **Issue** | Feature requests, bugs | Public (GitHub) | Open → Closed |

**Decision Flow:**
1. Is it about the development process? → **Friction Log**
2. Is it an architectural decision? → **ADR**
3. Is it unclear what the spec means? → **Question**
4. Is it a work item? → **Task**
5. Is it a feature request or bug? → **GitHub Issue**

---

## 7. Putting It All Together

The governance model creates a **closed loop** where specs, tests, docs, and runtime state are always in sync:

### 7.1 The Governance Loop

```
1. Specs define behavior (spec_ledger.yaml)
     ↓
2. Tests validate specs (BDD + unit tests)
     ↓
3. Code implements behavior (Rust)
     ↓
4. Runtime exposes state (/platform/* APIs)
     ↓
5. Selftest validates everything (CI gate)
     ↓
6. Feedback flows back (friction logs, ADRs, tasks)
     ↓
[Loop back to step 1]
```

### 7.2 Developer Workflow Example

**Scenario:** Implement a new feature

```bash
# 1. Check what needs work
curl http://localhost:8080/platform/agent/hints
# → { "task_id": "TASK-001", "requirement_ids": ["REQ-001"], "ac_ids": ["AC-001"] }

# 2. Create AC (if new)
cargo xtask ac-new AC-MYSERV-001 "Users can list todos" \
  --story US-MYSERV-001 \
  --requirement REQ-MYSERV-TODOS

# 3. Generate BDD scenario stubs
cargo xtask ac-suggest-scenarios AC-MYSERV-001
# Edit specs/features/todos.feature and tag with @AC-MYSERV-001

# 4. Generate context bundle
cargo xtask bundle implement_ac
# Read bundle/<task>/context.md

# 5. Implement code + tests
# ... edit crates/business-core/src/todos.rs ...
# ... implement BDD steps in crates/acceptance/src/steps/todos.rs ...

# 6. Validate incrementally
cargo xtask test-changed          # Fast: only what changed
cargo xtask test-ac AC-MYSERV-001 # AC-specific
cargo xtask ac-status              # Check coverage

# 7. Full validation before PR
nix develop
cargo xtask selftest  # All 11 steps must pass

# 8. Update task status
curl -X POST http://localhost:8080/platform/tasks/TASK-001/status \
  -H "Content-Type: application/json" \
  -d '{"status": "Review"}'
```

### 7.3 Agent Workflow Example

**Scenario:** AI agent implements a feature autonomously

```bash
# 1. Query for work
GET /platform/agent/hints
# → Prioritized list of Todo/InProgress tasks with REQ/AC IDs

# 2. Get detailed guidance
GET /platform/tasks/suggest-next?task=implement_ac
# → Step-by-step sequence with commands

# 3. Execute workflow
cargo xtask bundle implement_ac
cargo xtask ac-new AC-MYSERV-001 "..." --requirement REQ-MYSERV-TODOS
cargo xtask ac-suggest-scenarios AC-MYSERV-001
# [Agent edits files based on bundle context]
cargo xtask test-ac AC-MYSERV-001
cargo xtask ac-status

# 4. Validate
cargo xtask selftest
# → If green: mark task Review
# → If red: log friction or question

# 5. Report status
POST /platform/tasks/TASK-001/status {"status": "Review"}
```

### 7.4 Fork Maintainer Workflow Example

**Scenario:** Adopt template in new domain (e.g., Knowledge Hub)

```bash
# 1. Fork template
git clone https://github.com/EffortlessMetrics/Rust-Template.git khub
cd khub

# 2. Validate kernel baseline
nix develop
cargo xtask kernel-smoke  # All kernel ACs should pass

# 3. Register fork
cargo xtask fork-register \
  --name "Knowledge Hub" \
  --domain ml-documentation \
  --kernel-version v3.3.8

# 4. Add domain-specific specs
# Edit specs/spec_ledger.yaml:
#   - Add US-KHUB-* stories
#   - Add REQ-KHUB-* requirements
#   - Add AC-KHUB-* acceptance criteria

# 5. Implement domain features
cargo xtask ac-new AC-KHUB-001 "Users can search ML docs" \
  --story US-KHUB-001 \
  --requirement REQ-KHUB-SEARCH

# 6. Validate fork
cargo xtask selftest
# → Kernel ACs (AC-TPL-*, AC-PLT-*) must pass
# → Domain ACs (AC-KHUB-*) can fail during development

# 7. Track divergence
cargo xtask kernel-status
# → Shows which kernel contracts are satisfied
# → Surfaces any template updates available
```

---

## 8. Key Concepts Summary

### 8.1 Specs-as-Code Hierarchy

| Level | ID Pattern | Purpose | Must Have |
|-------|-----------|---------|-----------|
| Story | US-* | User value | Requirements |
| Requirement | REQ-* | System property | ACs (if `must_have_ac: true`) |
| AC | AC-* | Testable behavior | Tests array |
| Test | @AC-* (BDD), test name (unit) | Proof | Passing status |
| Doc | DESIGN-*, ADR-* | Context | REQ/AC refs |
| Command | (defined in devex_flows.yaml) | Developer action | Implementation |

### 8.2 Governance Graph Invariants

1. **REQ_HAS_AC** — Requirements with `must_have_ac: true` have ≥1 AC
2. **AC_HAS_TEST** — ACs with `tests` array have ≥1 test node
3. **COMMAND_REACHABLE** — Required commands used in ≥1 flow
4. **NO_ORPHAN_DOCS** — Design docs reference ≥1 REQ/AC
5. **ADR_EXISTS** — ADR references resolve to files

### 8.3 Validation Ladder

1. **Fast checks** — `cargo xtask check` (seconds)
2. **AC-focused** — `cargo xtask test-ac AC-*` (seconds to minutes)
3. **AC status** — `cargo xtask ac-status` (seconds)
4. **Full selftest** — `cargo xtask selftest` (10-20 min Tier-1, 2+ hours Tier-2)

### 8.4 Skills and Agents Governance

- **Skills** — Workflow recipes (`.claude/skills/*`)
- **Agents** — AI personas (`.claude/agents/*`)
- **Validation** — `skills-lint`, `agents-lint` (in selftest steps 2-3)
- **Security** — No secrets, least-privilege tools, explicit permissions

### 8.5 Governance Artifacts

- **ADRs** — Architectural decisions (permanent)
- **Friction Logs** — DevEx pain points (tracked to resolution)
- **Tasks** — Work items (state machine: Todo → InProgress → Review → Done)
- **Questions** — Spec ambiguities (open → answered → resolved)

---

## 9. Reference Material

### Related Documentation

**Governance Deep Dives:**
- [Template Contracts](TEMPLATE-CONTRACTS.md) — Kernel contracts and customization surface
- [Rust-as-Spec Overview](rust-as-spec-overview.md) — Conceptual model and four-phase pipeline
- [Controls as Code](controls-as-code.md) — Compliance and policy testing
- [Skills Governance](../SKILLS_GOVERNANCE.md) — Skills validation rules
- [Agents Governance](../AGENTS_GOVERNANCE.md) — Agents validation rules

**Operational Guides:**
- [CLAUDE.md](../../CLAUDE.md) — Agent operational instructions
- [Agent Guide](../AGENT_GUIDE.md) — API and workflow reference for agents
- [Selective Testing Guide](../SELECTIVE_TESTING.md) — Validation ladder usage

**Specs and Schemas:**
- `specs/spec_ledger.yaml` — Canonical specs
- `specs/devex_flows.yaml` — Workflow definitions
- `specs/tasks.yaml` — Work items
- `specs/config_schema.yaml` — Configuration contract

**Architecture Decision Records:**
- [ADR-0003: Spec and BDD as Source of Truth](../adr/0003-spec-bdd-source-of-truth.md)
- [ADR-0005: Selftest as the Single Quality Gate](../adr/0005-xtask-selftest-single-gate.md)
- [ADR-0017: Tier-1 Selftest as Required Gate on Main Branch](../adr/0017-tier1-selftest-gate.md)
- [ADR-0020: Claude Code Skills Governance](../adr/0020-claude-code-skills-governance.md)
- [ADR-0021: Claude Code Agents Governance](../adr/0021-claude-code-agents-governance.md)

### Platform APIs

**Governance & Discovery:**
- `GET /platform/status` — Governance health, policy status, metadata
- `GET /platform/graph` — Full governance graph (JSON)
- `GET /platform/schema` — JSON schemas for all specs
- `GET /platform/coverage` — AC coverage summary
- `GET /platform/docs/index` — Documentation inventory

**Work & Tasks:**
- `GET /platform/tasks` — Task list with filtering
- `POST /platform/tasks/{id}/status` — Update task status
- `GET /platform/tasks/graph` — Task dependencies
- `GET /platform/tasks/suggest-next?task=X` — Workflow guidance
- `GET /platform/agent/hints` — Prioritized work suggestions

**Metadata & Issues:**
- `GET /platform/friction` — Friction log entries
- `GET /platform/questions` — Question artifacts
- `GET /platform/forks` — Fork registry

### CLI Commands

**Bootstrap:**
- `cargo xtask dev-up` — One-command setup
- `cargo xtask doctor` — Environment check
- `cargo xtask kernel-smoke` — Kernel validation

**Validation:**
- `cargo xtask check` — Fast checks
- `cargo xtask test-changed` — Change-aware testing
- `cargo xtask test-ac <AC-ID>` — AC-specific tests
- `cargo xtask ac-status` — AC coverage report
- `cargo xtask selftest` — Full governance gate

**Governance:**
- `cargo xtask skills-lint` — Validate Skills
- `cargo xtask agents-lint` — Validate Agents
- `cargo xtask policy-test` — Run OPA policies
- `cargo xtask graph-export --check-invariants` — Validate graph

**Development:**
- `cargo xtask ac-new <ID> "<text>" --requirement <REQ>` — Create AC
- `cargo xtask ac-suggest-scenarios <AC-ID>` — Generate BDD stubs
- `cargo xtask bundle <task>` — Generate context bundle
- `cargo xtask task-create` — Create task
- `cargo xtask task-update --id <ID> --status <status>` — Update task

**Artifacts:**
- `cargo xtask adr-new "<title>"` — Create ADR
- `cargo xtask friction-new` — Log friction
- `cargo xtask questions-list` — List questions

---

## 10. Summary: What Makes This Governance Work

The governance model succeeds because it:

1. **Makes specs executable** — Not just documentation, but runtime contracts
2. **Validates everything automatically** — `cargo xtask selftest` as single gate
3. **Exposes runtime state** — `/platform/*` APIs show the same truth CI enforces
4. **Enables progressive validation** — Fast checks to full selftest
5. **Governs workflows** — Skills and Agents are first-class artifacts
6. **Provides feedback loops** — Friction logs, ADRs, tasks, questions
7. **Enforces invariants** — Graph structure rules prevent drift
8. **Supports autonomous agents** — Clear APIs, hints, and validation
9. **Scales with forks** — Kernel contracts + domain extensions
10. **Proves correctness** — Selftest green = system is governed

**Bottom line:** If `cargo xtask selftest` passes, your repo is honest. If it fails, the governance model tells you exactly what's broken and how to fix it.
