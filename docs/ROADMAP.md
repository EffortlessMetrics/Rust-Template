# Roadmap: Rust-as-Spec Platform Cell (v3.3.x)

This roadmap describes how we take the **v3.3.x kernel** from “implemented and selftesting” to a **fully reusable platform cell**:

- Kernel ACs green on **Tier-1** (Nix devshell on Linux/macOS/WSL2)
- Flows and APIs stable enough that humans and agents can work autonomously
- CI enforcing the same contracts that `/platform/*` and `/ui` expose

The emphasis is on **enabling**: we want a cell where the easiest path is also the governed one, for both people and LLMs.

---

## 1. Current position (v3.3.1)

Today, the cell already provides:

- **Governed kernel**
  - `cargo xtask selftest` passes on Tier-1 and exercises:
    - fmt, clippy, unit tests
    - BDD / Cucumber acceptance tests
    - AC ↔ tests ↔ ADR mapping
    - LLM context bundler checks
    - Policy tests (OPA/Conftest)
    - DevEx contract checks
    - Graph invariants and AC coverage sanity
  - All **kernel** ACs are `[PASS]` in `docs/feature_status.md`; remaining `[UNKNOWN]` ACs are explicitly marked as template/future.

- **Introspection**
  - `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/devex/flows`, `/platform/docs/index`, `/platform/schema`, `/platform/agent/hints` are implemented and wired to the same spec/runtime the kernel uses.
  - `/ui` renders status, graph, flows, and tasks with no separate DB.

- **DevEx flows**
  - `xtask doctor`, `check`, `test-changed`, `test-ac`, `selftest`, `release-*` and related commands are implemented and covered by ACs/BDD.
  - Selective testing (`test-changed` + plan-only) is present and documented.

- **Docs and spellcheck**
  - Spec ledger, flows, tasks, and docs are aligned.
  - `cspell` is pinned inside the Nix devshell; `cspell.json` is curated for the domain so Tier-1 spellcheck is clean.

From here, the roadmap focuses on **finishing the contract** and **smoothing the experience** for humans and agents.

---

## 2. Definition of “Fully Implemented”

We’ll call the template “fully implemented” when all of the following hold:

1. **Kernel is clean and enforced**
   - Tier-1 `cargo xtask selftest` green on `main`.
   - A Tier-1 selftest job in CI is **required** for merges.
   - All `kernel: true` / `must_have_ac: true` ACs are `[PASS]`, and graph invariants are enforced by code and referenced from the ledger.

2. **DevEx is predictable and low-friction**
   - `xtask check`, `test-changed`, `test-ac`, `selftest` behave exactly as described in docs.
   - Pre-commit hooks regenerate/auto-stage derived artifacts (like `feature_status`) and give fast, actionable feedback.
   - Tier-2 (native Windows) flow is documented and matches reality; Tier-1/WSL2 is the canonical gate.

3. **Agent interface is ready for real work**
   - `/platform/*` endpoints and `xtask` flows give agents everything required to:
     - discover tasks,
     - get bounded context,
     - apply changes,
     - run checks,
     - and surface questions/decisions as artifacts.
   - Skills (in `.claude/skills/*`) and `CLAUDE.md` / `AGENT_GUIDE.md` are aligned with actual behaviour.

4. **IaC and CI are part of the contract**
   - Docker compose, K8s, and Terraform examples are consistent with `config_schema.yaml` and `envs.yaml`.
   - CI workflows are referenced from REQs/ACs where appropriate.

5. **IDP/platform-cell story is explicit**
   - `service_metadata.yaml` + `/platform/*` expose enough to treat the repo as a **per-service IDP unit**.
   - Release bundles (`release_evidence/vX.Y.Z.md`) contain usable evidence (tasks, ACs, policies, selftest, friction log).
   - Template contracts and positioning docs explain “what a compliant cell is” on a single page.

---

## 3. Phase 1 – Lock Tier-1 and CI

**Goal:** Tier-1 selftest is not just healthy, it’s also the enforced gate on `main`.

### 3.1 Tier-1 baseline

- Inside Nix devshell on Linux/macOS/WSL2:

  ```bash
  nix develop
  cargo xtask precommit
  cargo xtask selftest
````

* Confirm both are clean on `main` with no skip flags.

**Exit criteria:**

* Tier-1 precommit and selftest pass on the current template version without manual intervention.

### 3.2 CI wiring

* Add or confirm a CI job (e.g. `tier1-selftest`) that runs:

  ```bash
  nix develop --command cargo xtask selftest
  ```

* Mark this job as **required** on `main` in branch protection.

**Exit criteria:**

* Any change that breaks selftest cannot merge to `main`.

---

## 4. Phase 2 – Smooth DevEx & Pre-Commit

**Goal:** Day-to-day work (including from agents) feels natural: `git commit` “just works” and governance shows up as helpful feedback.

### 4.1 Pre-commit ergonomics

Implement the following behaviour in `xtask precommit`:

* Regenerate `docs/feature_status.md` via `ac-status`.
* If it changed:

  * `git add docs/feature_status.md`
  * Log a concise note about the update.
* Run fmt, clippy, unit tests, and relevant acceptance tests.
* Run docs-check and spellcheck in a **soft mode** for local pre-commit:

  * log problems clearly,
  * do not block the commit unless a strict env flag is set (e.g. `XTASK_STRICT_PRECOMMIT=1`).

In `selftest`, keep docs-check and spellcheck **strict**.

**Exit criteria:**

* On a clean tree, a typical `git commit`:

  * regenerates+stages `feature_status` if needed,
  * provides docs/spellcheck feedback,
  * only fails on true breakage (tests, compilation, selftest failures).

### 4.2 Tier-2 (Windows) story

* Confirm the recommended Tier-2 flow:

  ```powershell
  $env:XTASK_LOW_RESOURCES = "1"
  cargo xtask check
  cargo xtask test-changed
  ```

* Ensure logs clearly indicate which steps are skipped.

* Use WSL2/Nix/CI for full `selftest`.

Update:

* `docs/reference/platform-support.md`
* `docs/MISSING_MANUAL.md`

with the canonical Windows/WSL2 guidance.

**Exit criteria:**

* One clearly documented and tested Windows happy path:

  * Native for fast local loops,
  * WSL2/CI for authoritative governance.

---

## 5. Phase 3 – Finish Agent Surfaces & Flows

**Goal:** Agents can execute meaningful work end-to-end using specs, flows, tasks, `xtask`, and `/platform/*`, without guesswork.

### 5.1 `/platform/agent/hints`

* Confirm and refine the contract for `AC-TPL-AGENT-HINTS`:

  * Inputs, response schema, semantics of `status`, recommended actions, ordering.
* Align handler and spec (ledger + docs).
* Ensure BDD for hints is stable and passing.
* Add a concrete example + usage notes to `docs/AGENT_GUIDE.md` and `CLAUDE.md`.

**Exit criteria:**

* Hints endpoint returns stable, documented JSON and has passing AC coverage.

### 5.2 Task flows (CLI + HTTP)

* Clarify and implement the semantics for:

  * CLI: `tasks-list`, `task-create`, `task-update`.
  * HTTP: `GET /platform/tasks`, `POST /platform/tasks/{id}/status`.

* Align with the relevant ACs: `AC-TPL-TASKS-CLI`, `AC-TPL-TASKS-CREATE-CLI`, `AC-TPL-TASKS-UPDATE-CLI`, `AC-TPL-TASKS-HTTP`.

* Update BDD expectations and docs to match the actual UX.

* Reflect this in `AGENT_GUIDE.md` as a canonical “agent task lifecycle”.

**Exit criteria:**

* Tasks can be discovered, created, updated, and observed consistently via CLI and HTTP, and ACs are `[PASS]`.

### 5.3 Flow idempotency

* Implement and test `AC-TPL-FLOW-IDEMPOTENT`:

  * Repeated runs of:

    ```bash
    cargo xtask selftest
    cargo xtask suggest-next
    ```

    do not generate duplicates or drift when there are no underlying changes.

* Add tests to assert stability over multiple runs and wire them into the spec ledger.

**Exit criteria:**

* Agents and humans can safely rerun flows without worrying about corrupting state or duplicating artifacts.

### 5.4 Questions as artifacts

* Implement `AC-TPL-QUESTIONS-LOGGED`:

  * Define a light “questions” mechanism (e.g. `questions/*.yaml`, friction log entries, or structured status).
  * Have flows record questions when they encounter ambiguity or missing data.
  * Surface a summary via `cargo xtask status` and/or `/platform/status`.

* Add BDD coverage and link it from the ledger.

**Exit criteria:**

* Ambiguity is recorded as structured questions, not lost or handled ad hoc; agents can move forward and leave a clear trail.

---

## 6. Phase 4 – IaC & CI as Contracted Surfaces

**Goal:** Compose, K8s, Terraform, and CI aren’t “supporting docs”; they’re part of the governed system.

### 6.1 docker-compose (local runtime)

* Ensure `AC-TPL-IAC-COMPOSE-ALIGN` is fully backed by tests in `local_docker`:

  * Ports and environment keys match `config_schema.yaml` and `envs.yaml`.
  * Local services (DB, tracing) follow the documented defaults.

**Exit criteria:**

* Local Docker is a trustworthy convenience example; its alignment AC is `[PASS]`.

### 6.2 K8s and Terraform examples

For K8s (`AC-TPL-IAC-K8S-ALIGN`):

* Add minimal manifests under `infra/k8s`.
* Write a test that parses manifests and checks ports/env names against the config schema.

For Terraform (`AC-TPL-IAC-TF-ALIGN`):

* Add a minimal module under `infra/tf`.
* Write a test to check variable names/defaults against the config schema.

Wire both tests into the ledger.

**Exit criteria:**

* K8s/TF examples exist, are small, and are verifiably consistent with the config schema.

### 6.3 CI workflows

* Reference the CI workflows (especially Tier-1 selftest) from:

  * `doc_index.yaml`
  * Relevant REQs/ACs (DevEx, release, governance).

**Exit criteria:**

* Pipelines are visible in the governance graph and treated as first-class infrastructure.

---

## 7. Phase 5 – Contract, Positioning, and Reuse

**Goal:** The cell is clearly defined, easy to adopt, and proven in reuse.

### 7.1 Template contract

* Finalize `TEMPLATE-CONTRACTS.md` (or `platform-kernel.md`) so it answers:

  * Which ACs define the kernel.
  * Which `xtask` commands are mandatory for a derived service.
  * Which `/platform/*` endpoints must exist and what they return.
  * Expectations for `service_metadata.yaml`, release evidence, and friction logs.

**Exit criteria:**

* A new team can understand “what makes a service a Rust-as-Spec cell” in one document, and selftest enforces everything listed there.

### 7.2 IDP/portal positioning

* Add `docs/explanation/idp-positioning.md`:

  * How this cell plugs under Backstage/Port/Humanitec/etc.
  * Which endpoints and artifacts a portal or orchestrator should consume (`/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/schema`, `release_evidence/...`).

**Exit criteria:**

* Platform/IDP owners understand how to integrate this cell into their existing toolchain.

### 7.3 Brownfield / second service

* Instantiate a second service from this cell or wrap an existing service.

* Use only:

  * documented Skills,
  * `xtask` commands,
  * `/platform/*` APIs
    to complete:

  ```text
  Bootstrap → Feature Dev → Maintenance → Release
  ```

* Capture a `FRICTION_LOG.md` in that repo and feed any systemic improvements back into the template.

**Exit criteria:**

* At least one reuse story demonstrates the template behaves as intended in practice, not just in theory.

---

## 8. Ultra-compressed path

If you need the roadmap in one page:

1. **Lock Tier-1 & CI**
   Make `cargo xtask selftest` in Nix devshell the required CI gate on `main`.

2. **Smooth pre-commit & DevEx**
   Pre-commit regenerates docs and helps developers/agents; docs/spellcheck are strict in selftest.

3. **Finish agent surfaces**
   `/platform/agent/hints`, tasks CLI/HTTP, bundles, and Skills give agents a complete, safe loop.

4. **Align IaC & pipelines**
   Compose/K8s/TF/CI are small, tested, and consistent with the config schema and REQs.

5. **Freeze contract & prove reuse**
   Document the kernel contract, explain IDP positioning, and validate with at least one additional service.

Once these are done, you have a **fully implemented, self-governing Rust-as-Spec platform cell** that humans and agents can work in confidently, and that a portal/IDP can treat as a first-class building block.

```
::contentReference[oaicite:0]{index=0}
