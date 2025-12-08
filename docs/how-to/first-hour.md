---
id: HOWTO-FIRST-HOUR-001
title: "Your First Hour with the Template"
doc_type: how-to
status: published
audience: new-contributors, fork-maintainers
tags: [onboarding, quickstart, tutorial]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-ONBOARDING
  - REQ-PLT-DEVEX-CONTRACT
acs:
  - AC-PLT-ENV-CHECK
  - AC-PLT-009
adrs: [ADR-0002, ADR-0005]
last_updated: 2025-12-07
---
<!-- doclint:disable orphan-version -->

# Your First Hour with the Template

**Goal:** Get oriented with the Rust-as-Spec platform cell in ~30-60 minutes.

This is a hands-on tour of the template's key features. You'll check environment health, explore governance, run tests, and interact with the platform APIs.

---

## Prerequisites

- **Nix installed** (for reproducible builds and tooling)
- **Repository cloned** and you're in the project root
- **Terminal ready**

If you haven't entered the Nix environment yet:

```bash
nix develop
```

---

## Step 1: Check Environment Health

**Command:**
```bash
cargo xtask doctor
```

**What it does:**
Validates your development environment, checking:
- Rust toolchain version
- Required tools (git, conftest, Nix)
- Build configuration (sccache, ABI compatibility)
- Repository structure

**What to look for:**
```
✓ Rust toolchain... rustc 1.91.1
✓ glibc compatibility... glibc 2.39
✓ Cargo... cargo 1.91.1
✓ Nix... nix (Nix) 2.30.2
```

You may see warnings about optional tools (like `cargo-hakari`) or `libz.so.1` – these won't block your work. The final line should say:

```
⚠ Environment functional with warnings
```

Or ideally:

```
✅ Environment ready
```

**If it fails:** Check `docs/TROUBLESHOOTING.md` for common environment issues.

---

## Step 2: View Acceptance Criterion Coverage

**Command:**
```bash
cargo xtask ac-status
```

**What it does:**
Analyzes the governance graph to show:
- How many Acceptance Criteria (ACs) exist
- Which ACs are passing/failing
- BDD scenario coverage
- Unit test mappings to ACs

**What to look for:**
```
Parsing ledger: specs/spec_ledger.yaml
  Found 121 ACs
  Found 191 scenarios
  Found results for 91 ACs
  Captured results for 388 unit tests
[OK] All ACs passed
```

This command generates `docs/feature_status.md` – a living document showing which features are implemented and tested.

**Key insight:** This template treats specs as executable contracts. Every feature is tied to an AC, and every AC is tested via BDD scenarios and/or unit tests.

---

## Step 3: Validate the Governance Graph

**Command:**
```bash
cargo xtask graph-export --check
```

**What it does:**
Validates the governance graph structure:
- Stories → Requirements → Acceptance Criteria
- ACs → BDD scenarios → Tests → Documentation
- Checks for orphaned specs, broken links, and missing coverage

**What to look for:**
```
✓ Graph structure valid
✓ No orphaned requirements
✓ All ACs link to stories
✓ Documentation references correct
```

This ensures the spec ledger (`specs/spec_ledger.yaml`) is internally consistent. If this fails, it means there's a structural issue in how stories/requirements/ACs are connected.

---

## Step 4: Run BDD Tests

**Command:**
```bash
cargo xtask bdd
```

**What it does:**
Runs all Gherkin feature files (`specs/features/*.feature`) using the BDD test harness. These are black-box acceptance tests written in plain English.

**What to look for:**
```
✓ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

**Explore a feature file:**
Open `specs/features/myserv_todos.feature` to see how BDD scenarios are tagged with AC IDs:

```gherkin
@AC-MYSERV-001
Scenario: GET /todos returns a JSON array of todos
  Given the user has existing todos
  When I send a GET request to "/todos"
  Then the response status should be 200
  And the response should be a JSON array
```

These scenarios are executable specifications – they define what "done" means for each AC.

---

## Step 5: Start the Service and Hit Platform APIs

**Commands:**
```bash
# Start the HTTP service
cargo run -p app-http
```

The service starts on `http://localhost:8080`. In another terminal:

```bash
# Check governance health
curl http://localhost:8080/platform/status

# View the full governance graph
curl http://localhost:8080/platform/graph

# See available developer flows
curl http://localhost:8080/platform/devex/flows

# List all tasks
curl http://localhost:8080/platform/tasks

# Get agent work recommendations
curl http://localhost:8080/platform/agent/hints
```

**What to look for:**

`/platform/status` returns:
```json
{
  "status": "ok",
  "timestamp": "...",
  "governance": {
    "ledger_path": "specs/spec_ledger.yaml",
    "total_stories": 15,
    "total_requirements": 98,
    "total_acs": 121,
    "ac_pass_count": 121,
    "ac_fail_count": 0
  },
  "auth_mode": "disabled"
}
```

`/platform/graph` shows the entire spec → test → docs graph as JSON.

`/platform/devex/flows` lists all `cargo xtask` commands with descriptions.

**Key insight:** These introspection APIs are how agents (and humans) understand the repo's current state. They're authoritative and always up-to-date.

---

## Step 6: Explore the MYSERV /todos Example

The template includes a simple "todos" service slice to demonstrate the full stack.

**Check the feature file:**
```bash
cat specs/features/myserv_todos.feature
```

You'll see scenarios tagged with `@AC-MYSERV-001`, `@AC-MYSERV-002`, etc.

**Hit the endpoint:**
```bash
curl http://localhost:8080/todos
```

You should get:
```json
[]
```

(An empty array, since there are no todos yet.)

**Find the implementation:**
- **Spec:** `specs/spec_ledger.yaml` – search for `AC-MYSERV-001`
- **BDD:** `specs/features/myserv_todos.feature`
- **Code:** `crates/app-http/src/routes/todos.rs`
- **Tests:** `crates/app-http/tests/todos_tests.rs`

**Key insight:** Every feature follows the same pattern:
1. Story/REQ/AC defined in `spec_ledger.yaml`
2. BDD scenarios in `specs/features/`
3. Implementation in `crates/`
4. Unit tests linked to AC IDs via `#[test_case::tags("AC-MYSERV-001")]`

---

## What's Next?

You've completed the first-hour tour! Here's where to go deeper:

### Understand the Architecture
- **[docs/AGENT_GUIDE.md](../AGENT_GUIDE.md)** – How LLM agents work within this repo (discovery → plan → execute → validate)
- **[docs/explanation/TEMPLATE-CONTRACTS.md](../explanation/TEMPLATE-CONTRACTS.md)** – The kernel requirements, extension points, and governance contracts
- **[docs/explanation/rust-as-spec-overview.md](../explanation/rust-as-spec-overview.md)** – The conceptual model behind specs-as-code

### Explore Available Workflows
- **[.claude/skills/](.claude/skills/)** – Governed workflow recipes:
  - `bootstrap-dev-env` – One-command environment setup
  - `governed-feature-dev` – AC-first feature development
  - `governed-maintenance` – Fixing drift and health checks
  - `governed-release` – Release preparation and validation
  - `governed-governance-debug` – Debugging selftest failures

### Try a Workflow
Pick one:
- **Add a new AC:** `cargo xtask ac-new AC-MYSERV-005 "User can delete a todo" --story US-MYSERV-001 --requirement REQ-MYSERV-TODOS`
- **Run the full governance check:** `cargo xtask selftest`
- **Generate a work bundle:** `cargo xtask bundle implement_ac` (see `.llm/contextpack.yaml` for all available bundles)
- **Create an ADR:** `cargo xtask adr-new "Use PostgreSQL for persistence"`

### Get Help
- **Flows reference:** `cargo xtask help-flows`
- **Task list:** `cargo xtask tasks-list`
- **Troubleshooting:** `docs/TROUBLESHOOTING.md`
- **Friction log:** `cargo xtask friction-list` (see known DevEx issues)

---

## Summary

In this first hour, you:
1. ✅ Validated your environment with `doctor`
2. ✅ Checked AC coverage with `ac-status`
3. ✅ Validated the governance graph
4. ✅ Ran BDD tests
5. ✅ Started the service and explored platform APIs
6. ✅ Explored the MYSERV /todos example

You now understand:
- How specs, tests, and docs are connected
- How to validate governance with `cargo xtask`
- How to introspect the repo via `/platform/*` APIs
- The pattern for implementing features (AC → BDD → code → tests)

**Welcome to the Rust-as-Spec platform cell. The contracts keep you safe; the tools keep you productive.**
