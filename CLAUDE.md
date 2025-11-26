# Agent Guidelines: Rust-as-Spec Platform Cell
**Template Version:** v3.3.1

**You are a team member working inside a self-governing platform cell.**

Your job is to execute work **within** the governance contracts, not around them.

---

## 1. Core Directive

**Do not invent workflows.**  
This repo already defines how work should be done.

Your authority comes from, in this order:

1. **Specs & Schema**  
   - `specs/spec_ledger.yaml` – stories → REQs → ACs → tests → docs  
   - `specs/config_schema.yaml` – configuration contract  
   - `specs/devex_flows.yaml` & `specs/tasks.yaml` – flows and tasks

2. **Skills** – `.claude/skills/*/SKILL.md`  
   Predefined workflows (feature dev, maintenance, release, governance debug).

3. **Platform APIs** – `/platform/*`  
   Ground-truth runtime state (status, graph, tasks, docs, hints, schema).

4. **xtask commands**  
   The only supported way to run checks, mutate specs, and orchestrate flows.

5. **Selftest** – `cargo xtask selftest`  
   Single source of truth for “is the system governed and healthy?”.

If any of these disagree with each other, **selftest wins**.

---

## 2. Skills you should use

Skills are your top-level workflows. Do not make up new ones.

### 2.1 Governed Feature Development
**Skill:** `governed-feature-dev`  
**Use when:** implementing or changing behaviour (new endpoints, flows, rules).  
**Contract:** AC → BDD → Code → Selftest  

Rough pattern:

1. Confirm or create AC in `spec_ledger.yaml` (via `xtask ac-new`).
2. Add / update BDD in `specs/features/*.feature`.
3. Implement code.
4. Run selective tests, then full selftest.

### 2.2 Governed Release
**Skill:** `governed-release`  
**Use when:** cutting versions, preparing releases, generating evidence.  
**Contract:** Prepare → Verify → Bundle → Tag  

Rough pattern:

1. `cargo xtask release-prepare X.Y.Z`
2. `cargo xtask selftest`
3. `cargo xtask release-bundle X.Y.Z`
4. Tag, push, let CI run `selftest`.

### 2.3 Governed Maintenance
**Skill:** `governed-maintenance`  
**Use when:** fixing drift, updating deps, addressing governance failures.  
**Contract:** Doctor → Audit / Fix → Docs → Graph → Selftest  

Rough pattern:

1. `cargo xtask doctor`
2. Fix reported issues (deps, config, docs).
3. `cargo xtask ac-status` + docs updates where needed.
4. `cargo xtask selftest`.

### 2.4 Governance Debug / Questions
**Skill:** `governed-governance-debug` (if present)  
**Use when:** selftest fails, graph invariants break, AC status looks off.  

Pattern:

1. Inspect `docs/feature_status.md`, `/platform/graph`, `/platform/status`.
2. Use `cargo xtask graph-export`, `ac-coverage`, and `docs-check`.
3. Fix mismatches, then rerun `selftest`.

---

## 3. Operational Commands (what you actually run)

### 3.1 Discovery

```bash
# What tasks exist?
cargo xtask tasks-list         # if implemented for this repo

# What should I do next for a given task?
cargo xtask suggest-next --task <TASK_ID>

# What flows exist and what commands are valid?
cargo xtask help-flows
````

When in doubt, **start from a task or a flow, not from raw code**.

### 3.2 Platform State

```bash
# Governance and runtime status
curl http://localhost:8080/platform/status

# Governance graph (stories/REQs/ACs/docs/commands)
curl http://localhost:8080/platform/graph

# Docs index
curl http://localhost:8080/platform/docs/index

# Tasks (HTTP view)
curl http://localhost:8080/platform/tasks

# UI dashboards
open http://localhost:8080/ui
```

Use `/platform/*` as your runtime truth instead of scraping files.

### 3.3 Bounded Context for an AC or Task

```bash
# Get an LLM-friendly context pack for a task/flow
cargo xtask bundle implement_ac
# or
cargo xtask bundle <TASK_ID>
```

Bundles are written under `.llm/bundle/` and are capped in size.
Treat bundle content as **the AC-level context**; don’t reach beyond it unless necessary.

### 3.4 Selective Testing (fast loop)

```bash
# Test only what changed vs origin/main
cargo xtask test-changed

# Plan-only (see what would run)
XTASK_TEST_CHANGED_PLAN_ONLY=1 cargo xtask test-changed

# Test a specific AC by ID
cargo xtask test-ac AC-PLT-001

# Use a different base for change detection
cargo xtask test-changed --base main
```

Tag expressions are normalized: input `AC-PLT-001` is treated as `@AC-PLT-001`.

**Default ladder:**

1. After edits: `cargo xtask test-changed`
2. Focusing on one AC: `cargo xtask test-ac <AC_ID>`
3. Before merge (Tier-1): `nix develop && cargo xtask selftest`

### 3.5 Validation & Governance

```bash
# Quick dev sanity (fmt, clippy, unit tests)
cargo xtask check

# Spellcheck & docs checks (depending on repo config)
cargo xtask spellcheck
cargo xtask docs-check

# AC coverage and status
cargo xtask ac-status
cargo xtask ac-coverage

# Full governance gate (Tier-1 only)
cargo xtask selftest
```

On **Tier-2/Windows**: prefer `check`, `test-changed`, `test-ac`.
On **Tier-1 (Nix+Linux/macOS/WSL2)**: use `selftest` as the pre-merge gate.

See `docs/SELECTIVE_TESTING.md` for the complete selective testing guide.

---

## 4. Golden Rules

### 4.1 Selftest is the arbiter

**If `cargo xtask selftest` fails, the work is not done.**

Selftest validates:

1. Core checks (fmt, clippy, unit tests)
2. BDD (behaviour matches AC text)
3. AC mapping (every kernel AC has tests; AC status is accurate)
4. LLM bundler (context generation is bounded and stable)
5. Policy tests (OPA/Rego)
6. DevEx contract (required xtask commands and flows exist)
7. Graph invariants (REQ/AC/command connectivity, no orphans)
8. AC coverage sanity (kernel ACs must be passing)

### 4.2 Never do this

* ❌ **Bypass selftest** or ignore a red result.
* ❌ **Claim work is complete** without ACs green for the relevant REQs (`ac-status` / `ac-coverage`).
* ❌ **Force-merge failing work** (CI selftest is a hard gate).
* ❌ **Hand-edit specs or schemas** unless explicitly instructed. Prefer:

  * `cargo xtask ac-new`, `adr-new`, `design-new`, etc.
* ❌ **Invent new workflows**. Use Skills + flows + tasks. If nothing fits, escalate via a question or friction log.

---

## 5. Decision Boundaries

### 5.1 You may do autonomously

* ✅ Implement ACs with clearly defined specs and scenarios.
* ✅ Fix failing tests and selftest failures.
* ✅ Update docs to match code and specs.
* ✅ Run DevEx and maintenance commands:

  * `doctor`, `check`, `test-changed`, `test-ac`, `ac-status`, `ac-coverage`.
* ✅ Propose and implement non-breaking improvements within existing flows.

### 5.2 You must involve a human

* ⚠️ High-risk work identified in `REQ-TPL-AGENT-INTERFACE` or similar REQs.
* ⚠️ Architecture decisions (new patterns, significant refactors) – requires an ADR.
* ⚠️ Security changes (auth, crypto, secrets, policies).
* ⚠️ Changes to graph invariants (e.g., removing `must_have_ac` from core REQs).
* ⚠️ Modifying policy files (`policy/*.rego`) beyond adding obvious new cases.

### 5.3 You must never

* ❌ Bypass governance gates (selftest, CI, policy tests).
* ❌ Introduce breaking changes to external contracts (APIs, schemas) without an ADR.
* ❌ Add new ad-hoc commands or flows without updating `devex_flows.yaml` and tests.
* ❌ Hide drift by editing generated artifacts instead of fixing root specs/tests.

If you’re unsure whether something is “autonomous” or “needs review,” treat it as **needs review** and surface a question (see below).

---

## 6. Handling ambiguity (questions as artifacts)

When you encounter ambiguity (missing ACs, unclear REQs, conflicting docs):

1. **Do not guess.**
2. Log a **structured question**:

   * Add or append to the friction log (`FRICTION_LOG.md`) if present, or
   * Use the repo’s question mechanism (if implemented), or
   * Open a draft issue with the relevant AC/REQ/ADR references.
3. Reference:

   * The task/AC/REQ IDs.
   * What you need to proceed.
   * What you recommend (if any).

Then continue with what you *can* do (e.g., unrelated ACs, doc cleanups) instead of blocking the whole flow.

---

## 7. First-time orientation

When you first see this repository:

1. **Read the strategic docs:**

   * `docs/ROADMAP.md` – where the template is and where it’s going.
   * `docs/AGENT_GUIDE.md` – extended version of this guide.
   * `docs/MISSING_MANUAL.md` – operational realities and caveats.
   * `TEMPLATE-CONTRACTS.md` – what a “compliant cell” must provide.

2. **Check environment and platform health:**

   ```bash
   cargo xtask doctor
   curl http://localhost:8080/platform/status || true
   ```

3. **Discover work:**

   ```bash
   cargo xtask tasks-list               # if available
   cargo xtask help-flows
   cargo xtask status                   # if implemented
   ```

4. **Before touching code:**

   * Confirm relevant REQ/AC in `specs/spec_ledger.yaml`.
   * Confirm or create tasks in `specs/tasks.yaml`.
   * Use `suggest-next` and bundles instead of guessing.

---

## 8. Example: implementing a new endpoint

**User asks:** “Add an endpoint to list users.”

**Your workflow (sketch):**

```bash
# 1. Confirm requirements / ACs
rg "list users" specs/spec_ledger.yaml || true

# 2. If missing, ask for details and then:
cargo xtask ac-new AC-MYSERV-USERS-LIST \
  "GET /users returns list of users" \
  --requirement REQ-MYSERV-USERS

# 3. Add a BDD scenario in specs/features/users.feature
#    tagged with @AC-MYSERV-USERS-LIST

# 4. Generate a context bundle
cargo xtask bundle implement_ac

# 5. Implement the handler and tests
cargo xtask test-ac AC-MYSERV-USERS-LIST
cargo xtask test-changed

# 6. Validate full governance
cargo xtask ac-status
cargo xtask selftest

# 7. Report status back (including AC ID, tests, and selftest result)
```

If anything in this flow conflicts with specs, flows, or selftest, **fix the conflict before reporting completion**.

---

## 9. References

* Operational guide: `docs/AGENT_GUIDE.md`
* Selective testing: `docs/SELECTIVE_TESTING.md`
* Technical overview: `docs/explanation/rust-as-spec-overview.md`
* Platform APIs: `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/docs/index`
* Skills: `.claude/skills/*/SKILL.md`

---

## 10. Remember

This is not a loose project. It is a **self-governing cell**:

* Specs are executable contracts, not advisory docs.
* Tests validate governance as much as functionality.
* Drift is treated as a failure, not as “something to clean up later.”

Work with the contracts (specs, flows, tasks, policies), not against them.
Your leverage comes from using the platform and tools as designed, then letting `selftest` and CI enforce the rest.

```
