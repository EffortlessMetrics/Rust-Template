# Roadmap: Rust-as-Spec Platform Cell

This roadmap is the shortest path to “fully implemented”: kernel ACs green on Tier-1, flows and APIs stable enough that other teams can drop this cell in and trust it. Current state: Kernel ACs are 100% passing on Tier-1 (Nix dev shell on Linux/macOS/WSL2). The only open item is a non-kernel local Docker convenience AC (`AC-TPL-LOCAL-DOCKER`), which is intentionally optional and not a gating requirement.

---

## Definition of “Fully Implemented”

1. **Kernel is clean**
   - `cargo xtask selftest` green on Tier-1 (Linux/macOS/WSL2).
   - All ACs marked `kernel: true` or `must_have_ac: true` are ✅ (no ❌/❓).
   - Graph invariants (REQ→AC, AC→tests, command reachability) enforced by code and referenced in the AC ledger.
2. **DevEx is predictable**
   - `xtask check`, `test-changed`, `test-ac`, `selftest` behave as documented.
   - Selective testing (change-aware BDD) stable, including plan-only mode.
   - Tier-2 (Windows) path documented and matches behaviour.
3. **Agent interface is complete**
   - `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints` match their ACs.
   - Agent skills + validation ladder are aligned with actual CLI behaviour.
4. **IDP / platform-cell story holds**
   - `service_metadata.yaml` + `/platform/*` expose enough to treat this as a reusable IDP cell.
   - Release bundles contain expected evidence (tasks, ACs, policies, selftest, friction log).
   - Docs (`why-this-exists`, `AGENT_GUIDE`, `SELECTIVE_TESTING`, platform support) tell one coherent story.

---

## Phase 0 — Establish Baseline

**Objective:** Know exactly what is red on Tier-1 today.

Commands:
```bash
cargo xtask selftest || true
cargo xtask ac-status > target/ac-status-baseline.txt
```

Capture all ❌ and core ❓ ACs (DevEx, graph, tasks, hints) into a short checklist (issue or tasks.md). Use this list to track the rest of the phases.

Exit: Checklist of named ACs to fix is recorded.

---

## Phase 1 — Close the Kernel (ACs + BDD + Graph)

**Goal:** Tier-1 selftest green; kernel ACs all ✅; no surprises in `docs/feature_status.md`.

### 1.1 DevEx ACs (doctor / check / dev-up / sbom-local)
- Run:
  ```bash
  CUCUMBER_TAG_EXPRESSION="@AC-PLT-001 or @AC-PLT-003 or @AC-PLT-008 or @AC-PLT-018" \
    cargo test -p acceptance --test acceptance -- --format=pretty
  ```
- For each failure, pick one layer to fix: AC text (`specs/spec_ledger.yaml`), feature (`specs/xtask_devex.feature`), or steps (`crates/acceptance/src/steps/xtask_devex.rs`); align implementation as needed.
- Re-run until clean, then:
  ```bash
  cargo xtask selftest
  cargo xtask ac-status
  ```
Exit: `AC-PLT-001/003/008/018` all ✅; dev-up output, check messaging, and SBOM path match docs.

### 1.2 Graph invariants in AC ledger
- For each `AC-TPL-GRAPH-*`, identify enforcing unit test(s) in `spec-runtime`.
- In `specs/spec_ledger.yaml`, add `tests:` entries of `type: unit` pointing to those tests.
- Extend AC status logic to handle `type: unit` (fail beats pass; no results = ❓).
- Decide `AC-TPL-GRAPH-SELFTEST` meaning:
  - Either add a thin BDD that breaks the graph and asserts `selftest` fails, or
  - Treat it as covered by unit tests.
- Re-run `cargo xtask selftest` and `cargo xtask ac-status`.
Exit: All `AC-TPL-GRAPH-*` ACs ✅ (or explicitly non-kernel); none left ❓.

---

## Phase 2 — Finish the Agent Interface

**Goal:** Agent-facing surfaces are trustworthy: “agents do the work, kernel keeps it safe.”

### 2.1 `/platform/agent/hints`
- Clarify contract in `specs/spec_ledger.yaml` (inputs, response fields, ordering/determinism).
- Align HTTP handler + runtime logic with that contract.
- Add/fix BDD for hints; run:
  ```bash
  CUCUMBER_TAG_EXPRESSION="@AC-TPL-AGENT-HINTS" \
    cargo test -p acceptance --test acceptance -- --format=pretty
  ```
- Update `docs/AGENT_GUIDE.md` with a JSON example and consumption guidance.
Exit: `AC-TPL-AGENT-HINTS` ✅; endpoint is documented and stable.

### 2.2 Task CLI/HTTP semantics
- Target ACs: `AC-TPL-TASKS-CLI`, `AC-TPL-TASKS-CREATE-CLI`, `AC-TPL-TASKS-UPDATE-CLI`, `AC-TPL-TASKS-HTTP`.
- Decide exact UX (CLI flags/exit codes/output; HTTP status/JSON shape). Adjust `xtask` and `/platform/tasks` to match; add unit tests as needed.
- Update BDDs to assert on stable substrings/fields; run:
  ```bash
  CUCUMBER_TAG_EXPRESSION="@AC-TPL-TASKS-CLI or @AC-TPL-TASKS-CREATE-CLI or @AC-TPL-TASKS-UPDATE-CLI or @AC-TPL-TASKS-HTTP" \
    cargo test -p acceptance --test acceptance -- --format=pretty
  ```
- Refresh `AGENT_GUIDE.md` examples (mark done via CLI/HTTP).
Exit: All task ACs ✅; agent docs reflect real UX.

---

## Phase 3 — Cement Selective Testing & Cross-Platform Story

**Goal:** `check` / `test-changed` / `test-ac` + Tier-2 behaviour are boringly predictable.

### 3.1 Selective testing as contract
- Treat the selective-testing scenario as canonical; ensure git worktree setup/teardown is robust and tag expression stable.
- Add a scenario where only specs change (no `.feature`) to prove `test-changed` finds ACs via the ledger; plan-only mode should emit correct tags.
- Ensure `docs/SELECTIVE_TESTING.md` + `AGENT_GUIDE.md` match behaviour (validation ladder: `test-changed` → `test-ac` → `selftest`).
Exit: Selective-testing BDDs ✅; docs read like reference, not aspiration.

### 3.2 Tier-2 (Windows) path
- On native Windows:
  ```powershell
  $env:XTASK_LOW_RESOURCES = "1"
  cargo xtask check
  cargo xtask test-changed
  ```
- Confirm no panics/permission crashes; logs state skipped steps; plan-only works with `XTASK_TEST_CHANGED_PLAN_ONLY=1`.
- Update `docs/reference/platform-support.md` and `MISSING_MANUAL.md` with the single recommended Windows flow (local low-resource; full selftest via WSL2/Nix/CI).
Exit: Tier-2 docs match reality; one clear Windows happy path.

---

## Phase 4 — Freeze the Template Contract

**Goal:** The cell is a reusable IDP unit with a clear, stable contract.

- Add/update a kernel contract doc (e.g., `docs/platform-kernel.md` or `TEMPLATE_CONTRACT.md`):
  - Kernel ACs that must remain in derived services.
  - Required `xtask` commands and intents.
  - Required `/platform/*` endpoints and behaviours.
  - Expectations for `service_metadata.yaml`, release bundles, `FRICTION_LOG.md`.
- Update `service_metadata.yaml` with the new baseline `template_version` (e.g., `3.3.0`) and links to kernel docs.
- Align `docs/why-this-exists.md`, `AGENT_GUIDE.md`, `CLAUDE.md` so they tell the same LLM-native, kernel-governed story.

Exit: “What does it mean to be a compliant Rust-as-Spec cell?” is answerable on one page and matches enforcement in `selftest`.

---

## Phase 5 — Prove Reuse (Optional but Powerful)

**Goal:** Validate the template by using it twice.

- Instantiate a second service from this cell; adjust metadata, ledger, and minimal domain logic.
- Run Bootstrap → Feature Dev → Maintenance → Release using only `xtask`, `/platform/*`, and documented Skills.
- Capture friction in that service’s `FRICTION_LOG.md` and backport improvements.

Exit: At least one surprise removed via reuse; the cell feels like an IDP product, not a one-off.

---

## Ultra-Compressed Path

1. Green the kernel: fix DevEx BDDs; wire graph ACs to unit tests; get selftest + kernel ACs fully green on Tier-1.
2. Finish agent surfaces: `/platform/agent/hints` + tasks CLI/HTTP match ACs and `AGENT_GUIDE`.
3. Lock flows: `check` / `test-changed` / `test-ac` + Tier-2 behaviour documented and tested.
4. Freeze contract: document kernel ACs + required surfaces; bump `template_version`; treat this repo as the canonical Rust-as-Spec cell.
5. (Optional) Reuse once to shake out final seams.

When 1–4 are done, you can say: this is a fully implemented, self-governing platform cell you can drop into any Rust shop and safely let agents work inside.
