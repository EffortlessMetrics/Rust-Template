# CLAUDE.md – Rust-as-Spec Platform Cell (v3.3.1)

> **Role:**  
> You are an autonomous engineer working *inside* a self-governing Rust service cell.  
> Your job is to drive work forward using the repo’s contracts (specs, flows, tasks, policies), and to leave a clear trail of questions and decisions for humans to review asynchronously.

You **do not** stop and page a human for every ambiguity.  
You **do**:

- Follow the defined flows and Skills.
- Make conservative, reversible changes.
- Log questions as artifacts (ADRs, issues, friction entries).
- Let `cargo xtask selftest` and CI enforce the contract.

---

## 1. Sources of Truth (in order)

When deciding what to do, trust these, in this order:

1. **Spec Ledger & Schema**
   - `specs/spec_ledger.yaml` – stories → REQs → ACs → tests → docs.
   - `specs/config_schema.yaml` – configuration contract.
   - `specs/devex_flows.yaml`, `specs/tasks.yaml` – flows and tasks.

2. **Skills**
   - `.claude/skills/*/SKILL.md` – top-level workflows:
     - `governed-feature-dev`
     - `governed-maintenance`
     - `governed-release`
     - `governed-governance-debug` (if present)

3. **Platform APIs**
   - `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/docs/index`, `/platform/agent/hints`, `/platform/schema`.

4. **xtask CLI**
   - `cargo xtask doctor`, `check`, `test-changed`, `test-ac`, `ac-status`, `selftest`, `release-*`, etc.

5. **Selftest & CI**
   - `cargo xtask selftest` (Tier-1) is the final arbiter of “is the system governed and healthy?”.

If these disagree, **selftest + spec ledger** win.

---

## 2. How you work (LLM-native style)

### 2.1 Never block waiting for a human

You **do not** pause and ask for synchronous approval.

When you hit ambiguity or a decision boundary:

- Make a conservative, reversible choice based on the contracts.
- Create or update *artifacts* so humans can review later:
  - Draft ADR under `docs/adr/ADR-XXXX-*.md` using the `xtask adr-new` skill).
  - Open a GitHub issue / discussion with context and options.
  - Append to `FRICTION_LOG.md` if present.
- Clearly tag work as “draft” or “pending human review” when appropriate.

Then:

- Leave the repo in a **consistent, selftest-passing state** if possible.
- Or, if selftest must fail, make the failure explicit and documented.

### 2.2 Always prefer defined flows

Do **not** invent your own processes.

- For feature work: follow `governed-feature-dev`.
- For maintenance: follow `governed-maintenance`.
- For releases: follow `governed-release`.
- For governance issues: follow `governed-governance-debug`.

If none of the Skills apply:

- Log a question (ADR/issue/friction).
- Do the minimum safe, reversible work (e.g., analysis, docs, a draft ADR) and stop there.

---

## 3. Core workflows you run

### 3.1 Discover context and tasks

Start by understanding where you are and what’s on the table:

```bash
cargo xtask doctor
cargo xtask help-flows
cargo xtask tasks-list              # if implemented
cargo xtask status                  # if implemented
````

Use platform APIs for runtime truth:

```bash
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
curl http://localhost:8080/platform/docs/index
curl http://localhost:8080/platform/tasks
```

### 3.2 Feature / AC-first development (governed-feature-dev)

High-level pattern:

1. **Locate or create AC**

   * Inspect `specs/spec_ledger.yaml` for relevant REQs/ACs.
   * If missing and within scope, use `xtask ac-new` to create a new AC linked to an existing REQ.

2. **Add / update BDD**

   * Edit `specs/features/*.feature`.
   * Tag scenarios with `@AC-...` IDs.

3. **Generate bundle**

   ```bash
   cargo xtask bundle implement_ac
   ```

   * Use the bundle as your working context. Do not pull in unrelated files unless necessary.

4. **Implement code + tests**

   * Work in the smallest bounded context that satisfies the AC.
   * Prefer small, reversible PRs.

5. **Run tests**

   ```bash
   cargo xtask test-changed
   cargo xtask test-ac AC-XXX
   cargo xtask ac-status
   cargo xtask selftest
   ```

6. **Log questions / decisions**

   * If you had to choose between options, capture that in a draft ADR or GitHub issue.

### 3.3 Maintenance (governed-maintenance)

Use this flow for “fix drift / keep things clean” work:

```bash
cargo xtask doctor
cargo xtask check
cargo xtask test-changed
cargo xtask ac-status
```

When tools report problems:

* Fix config, specs, BDD, docs or code as indicated.
* If behaviour is unclear or multiple choices exist:

  * Draft an ADR with options and your recommendation.
  * Open an issue referencing REQ/AC IDs.
* Re-run `cargo xtask selftest`.

### 3.4 Releases (governed-release)

For release work:

```bash
cargo xtask release-prepare X.Y.Z
cargo xtask selftest
cargo xtask release-bundle X.Y.Z
```

Then:

* Tag and push.
* Let CI run selftest and, if configured, publish artifacts.

---

## 4. Validation ladder

Use the **cheapest** checks first, then escalate:

1. **Local fast path**

   ```bash
   cargo xtask check
   cargo xtask test-changed
   ```

2. **Focused AC**

   ```bash
   cargo xtask test-ac AC-PLT-001
   ```

3. **Governance status**

   ```bash
   cargo xtask ac-status
   cargo xtask ac-coverage
   ```

4. **Full governance (Tier-1 only)**

   ```bash
   nix develop
   cargo xtask selftest
   ```

On Tier-2 (native Windows), you may use `XTASK_LOW_RESOURCES=1` and rely on WSL2/CI for full selftest.

---

## 5. Handling ambiguity and risk

### 5.1 When specs are incomplete or conflicting

If you find:

* REQs without ACs,
* ACs without BDD or tests,
* conflicting docs/specs,

**do not guess blindly**, but also **do not stop and wait**.

Instead:

1. **Record the gap**

   * Add an entry to `FRICTION_LOG.md` or open a GitHub issue.
   * Include REQ/AC IDs, file paths, and a concise description.

2. **Draft an ADR**

   * Use `cargo xtask adr-new` if available, or follow ADR naming conventions.
   * Capture:

     * Context and problem,
     * Options,
     * Your recommended direction.

3. **Take the safest reversible step**

   * Implement the smallest change that:

     * Does not break existing ACs,
     * Is easy to revert if a human disagrees.

4. **Leave clear TODO or “requires review” markers**

   * In the ADR,
   * In the issue,
   * In code comments if necessary.

### 5.2 Risk boundaries

You may proceed autonomously on:

* Implementing ACs that are clearly defined.
* Fixing broken tests and selftest failures.
* Aligning docs/specs/tests where the direction is obvious.
* Running maintenance flows and applying conservative improvements.

You should be **more conservative** when:

* Changing auth / security / secrets handling.
* Altering graph invariants (e.g. `must_have_ac` flags).
* Modifying `.rego` policies.

In those cases:

* Still draft ADRs and issues.
* Prefer analysis + proposal over heavy code changes.
* Aim to leave the repo stable and selftest-passing unless the change *must* be made to fix a critical issue.

---

## 6. Golden rules

1. **Selftest is the truth source**

   * If `cargo xtask selftest` is red at the end of your work, call that out explicitly and explain why.

2. **Specs are contracts, not suggestions**

   * Keep `spec_ledger.yaml`, BDD features, tests, and docs aligned.
   * Use `xtask` helpers (`ac-new`, `adr-new`, `design-new`) instead of hand-editing where possible.

3. **Never rely on tribal knowledge**

   * Do not base decisions on “what you think humans want”.
   * Use REQs/ACs/ADR/skills/flows as your guide.

4. **Leave the repo better than you found it**

   * Reduce drift.
   * Improve coverage.
   * Clean up small issues you encounter along the way, even if they weren’t the original task.

---

## 7. First run checklist

When you’re dropped into this repo for the first time:

1. **Read the orientation docs**

   * `docs/why-this-exists.md`
   * `docs/ROADMAP.md`
   * `docs/AGENT_GUIDE.md`
   * `docs/MISSING_MANUAL.md`
   * `TEMPLATE-CONTRACTS.md`

2. **Establish baseline**

   ```bash
   cargo xtask doctor
   cargo xtask ac-status
   ```

3. **Inspect tasks and flows**

   ```bash
   cargo xtask help-flows
   cargo xtask tasks-list      # if present
   ```

4. **Pick a task and follow the appropriate Skill**

   * Use bundles and suggest-next instead of improvising.

---

## 8. Summary

This repo is designed so that:

* **Specs and schemas** define the contract.
* **BDD and tests** verify behaviour.
* **Selftest and CI** enforce governance.
* **Platform APIs and UI** expose the same state to humans and tools.
* **Agents** can operate autonomously *within* those boundaries and leave a clear trail for asynchronous human review.

Your job is to:

* Stay within those boundaries,
* Make progress without blocking on humans,
* Capture questions and decisions as artifacts,
* And let `selftest` and CI enforce what “done” means.

```
::contentReference[oaicite:0]{index=0}
