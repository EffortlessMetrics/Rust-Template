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

## 1. Discovering Work

### Query Available Tasks

```bash
# List all tasks
cargo xtask tasks-list

# Get JSON output for programmatic use
cargo xtask tasks-list --json
```

**HTTP API:**
```bash
curl http://localhost:8080/platform/tasks
```

**Response structure:**
```json
{
  "tasks": [
    {
      "id": "implement_ac",
      "kind": "human",
      "category": "devex",
      "title": "Implement Acceptance Criterion",
      "summary": "...",
      "requirement": "REQ-TPL-AC-WORKFLOW",
      "recommended_flows": ["ac_first"]
    }
  ]
}
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

```bash
# 1. Create AC
cargo xtask ac-new AC-MYSERV-001 "Description" --requirement REQ-ID

# 2. Get context bundle (for LLM consumption)
cargo xtask bundle implement_ac
# Output: .llm/bundle/implement_ac.md

# 3. Implement (write code based on bundle context)

# 4. Run BDD tests
cargo xtask bdd

# 5. Validate governance
cargo xtask selftest
```

**Critical:** Always end with `cargo xtask selftest`. If it fails, the work is incomplete.

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

# Just BDD scenarios
cargo xtask bdd

# Just policies
cargo xtask policy-test

# Just graph
cargo xtask graph-export --check-invariants
```

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
- ✅ Add BDD scenarios for new behaviors
- ✅ Run `selftest` and fix violations

### What Requires Human Approval

- ⚠️ **Changing graph invariants** (e.g., removing `must_have_ac`)
- ⚠️ **Changing policy rules** (`.rego` files)
- ⚠️ **Changing DevEx flows** (workflow changes affect all developers)
- ⚠️ **Architectural decisions** (should be captured as ADR first)
- ⚠️ **Security-sensitive changes** (auth, crypto, secrets handling)

### What You Should Never Do

- ❌ **Bypass selftest** (force merge when it fails)
- ❌ **Edit specs without using `xtask` generators** (leads to ID collisions, schema errors)
- ❌ **Duplicate AC/REQ IDs** (breaks graph integrity)
- ❌ **Reference non-existent ADRs** (breaks traceability)
- ❌ **Hardcode secrets** (violates privacy policies)

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

# Execution
cargo xtask ac-new <id> "<desc>" --requirement <req>
cargo xtask bundle <task>
cargo xtask bdd
cargo xtask selftest

# Validation
cargo xtask check
cargo xtask policy-test
cargo xtask docs-check
cargo xtask graph-export --check-invariants

# Introspection
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
curl http://localhost:8080/platform/tasks
```

### Essential URLs

- Dashboard: `http://localhost:8080/ui`
- Graph: `http://localhost:8080/ui/graph`
- Flows: `http://localhost:8080/ui/flows`
- API Docs: `http://localhost:8080/platform/status`

---

## 12. Further Reading

- [CLAUDE.md](../CLAUDE.md) - High-level agent constitution
- [Technical Overview](explanation/rust-as-spec-overview.md) - Architecture deep-dive
- [ROADMAP.md](ROADMAP.md) - Strategic direction
- [DevEx Flows Reference](../specs/devex_flows.yaml) - All available workflows
- [Tasks Reference](../specs/tasks.yaml) - All available tasks
