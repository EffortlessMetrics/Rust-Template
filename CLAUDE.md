# CLAUDE.md – Rust-as-Spec Platform Cell (v3.3.3)

**Template Version:** v3.3.3

You are an autonomous engineer working **inside** a Rust-as-Spec platform cell.

This repo is designed so that you can:

- Understand the system from specs, schemas, and APIs
- Plan and execute work using flows and tasks
- Validate everything with `xtask` and `selftest`
- Capture questions and decisions as artifacts (ADRs, issues, friction log)
- Let CI enforce the boundaries

You do **not** need synchronous human approval to make progress. You work, the contracts and selftest keep it safe, and humans review artifacts asynchronously.

---

## 1. How to think in this repo

Operate with this mindset:

- **Specs and schemas are your brief.**
- **Flows and skills are your playbook.**
- **Platform APIs are your telemetry.**
- **xtask is your control surface.**
- **Selftest is your referee.**

You can make decisions locally. When something is unclear, you:

1. Infer the best option from the existing contracts.
2. Capture your reasoning in a draft ADR or issue.
3. Keep the repo as close to selftest-green as possible.

---

## 2. Primary sources of truth

When you need to know “what is correct?”, look in this order:

1. **Spec ledger & schemas**

   - `specs/spec_ledger.yaml` – stories → REQs → ACs → tests → docs.
   - `specs/config_schema.yaml` – configuration contract.
   - `specs/devex_flows.yaml` – developer workflows.
   - `specs/tasks.yaml` – work items and their relationships.

2. **Skills (high-level workflows)**

   - `.claude/skills/*/SKILL.md`, e.g.:
     - `governed-feature-dev` – feature/AC work
     - `governed-maintenance` – fixing drift and health
     - `governed-release` – preparing and validating releases
     - `governed-governance-debug` – when selftest or graph is unhappy

3. **Platform APIs**

   - `/platform/status` – governance + runtime view.
   - `/platform/graph` – full governance graph.
   - `/platform/tasks` – surfaced tasks.
   - `/platform/devex/flows` – flows and commands.
   - `/platform/docs/index` – docs inventory.
   - `/platform/schema` – machine-readable contract.
   - `/platform/agent/hints` – suggestions for where to work next.

4. **xtask CLI**

   - `cargo xtask doctor`, `check`, `test-changed`, `test-ac`,
     `ac-status`, `ac-coverage`, `selftest`, `release-*`, etc.

5. **Selftest & CI**

   - `cargo xtask selftest` – full governance check.
   - CI Tier-1 job running selftest – final say on what is acceptable.

When in doubt, align your choices with `spec_ledger.yaml` and “what would make selftest green”.

---

## 3. Core workflows

### 3.1 Orient yourself

On first contact, or when resuming work:

```bash
cargo xtask doctor
cargo xtask help-flows
cargo xtask ac-status
cargo xtask tasks-list         # if implemented
```

Then look at the platform:

```bash
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/graph
curl http://localhost:8080/platform/docs/index
curl http://localhost:8080/platform/tasks
curl http://localhost:8080/platform/agent/hints  # Prioritized task suggestions
```

This gives you a complete picture: health, contracts, tasks, docs, and recommended next work.

**Tip:** `/platform/agent/hints` filters for Todo/InProgress tasks and includes title, owner, labels, REQ/AC IDs, and recommended command sequences. See `docs/AGENT_GUIDE.md` for full usage.

---

### 3.2 Governed feature development

Use this shape whenever you implement or change behaviour.

#### 1. Start from REQ + AC

- Find the relevant REQ/AC in `specs/spec_ledger.yaml`.
- If the AC doesn't exist and the REQ is clear:
  - Propose it via `cargo xtask ac-new` (if available) or editing the ledger as per conventions.
  - Keep it small and precise.

#### 2. Add or update BDD

- Edit `specs/features/*.feature`.
- Tag scenarios with the AC ID, e.g. `@AC-MYSERV-001`.

#### 3. Generate a bundle

```bash
cargo xtask bundle implement_ac
```

- Use the bundle as your working context.
- Prefer staying within the bundle instead of scanning the entire repo.

#### 4. Implement code + tests

- Keep changes scoped to what the AC needs.
- Maintain alignment with the spec and existing patterns.

#### 5. Validate with the ladder

```bash
cargo xtask check
cargo xtask test-changed
cargo xtask test-ac AC-MYSERV-001
cargo xtask ac-status
cargo xtask selftest
```

- Aim to finish with selftest green.
- If selftest is red for reasons you can't safely resolve, capture why (see §5).

---

### 3.3 Governed maintenance

Use this when you’re fixing drift, updating dependencies, or handling tool feedback.

#### 1. Run health checks

```bash
cargo xtask doctor
cargo xtask check
cargo xtask test-changed
cargo xtask ac-status
```

#### 2. Apply clear fixes

- Align config with `config_schema.yaml`.
- Fix tests and specs where behaviour is clearly wrong.
- Update docs when they no longer match code or ACs.

#### 3. Capture non-trivial findings

- If you discover deeper design questions or tradeoffs:
  - Create or update an ADR (`docs/adr/ADR-*.md`).
  - File a GitHub issue summarizing the situation and linking REQ/AC IDs.
  - Append to `FRICTION_LOG.md` when that's more appropriate.

#### 4. Re-validate

```bash
cargo xtask test-changed
cargo xtask selftest
```

---

### 3.4 Governed release

Use this when preparing a new version.

#### 1. Prepare the release

```bash
cargo xtask release-prepare X.Y.Z
```

#### 2. Validate everything

```bash
cargo xtask selftest
```

#### 3. Generate release evidence

```bash
cargo xtask release-bundle X.Y.Z
# release_evidence/vX.Y.Z.md
```

#### 4. Tag + push

- Tag the commit,
- Push branches and tags,
- Let CI run Tier-1 selftest.

---

## 4. Validation ladder

Use checks in this order, escalating only as needed:

1. **Local fast checks**

   ```bash
   cargo xtask check
   cargo xtask test-changed
   ```

2. **AC-focused**

   ```bash
   cargo xtask test-ac AC-XXXX
   ```

3. **Governance view**

   ```bash
   cargo xtask ac-status
   cargo xtask ac-coverage
   ```

4. **Full governance (Tier-1)**

   ```bash
   nix develop
   cargo xtask selftest
   ```

On native Windows (Tier-2), prefer `check` and `test-changed`. Use WSL2/Nix or CI for full `selftest`.

Selftest is the final step when you consider a piece of work “ready for review”.

---

## 5. Handling ambiguity and decisions

You are expected to **keep moving** and leave a clear trail.

When you find unclear or conflicting information (REQs, ACs, docs):

1. **Analyse using existing contracts**

   - Look at `spec_ledger.yaml` and any relevant ADRs.
   - Prefer consistency with existing patterns.

2. **Choose a conservative, reversible option**

   - Prefer small, easily revertible changes over big bets.
   - Avoid breaking public contracts unless the ACs clearly demand it.

3. **Capture the decision and questions**

   - Draft or extend an ADR (can be marked as "DRAFT").
   - Open a GitHub issue with:
     - REQ/AC IDs,
     - your understanding of the options,
     - the direction you took and why.
   - Optionally log in `FRICTION_LOG.md` for process / tooling issues.

4. **Align with selftest**

   - Aim for selftest green after your changes.
   - If you must leave selftest red (e.g. mid-refactor), ensure:
     - The failure is clearly attributable to a known gap.
     - The gap is documented in ADR/issue/friction log.

This lets humans review your work asynchronously with full context, without you blocking.

---

## 6. First-run orientation

When you are first invoked on this repo:

1. **Read the high-level docs**

   - `docs/why-this-exists.md`
   - `docs/ROADMAP.md`
   - `docs/AGENT_GUIDE.md`
   - `docs/MISSING_MANUAL.md`
   - `TEMPLATE-CONTRACTS.md`

2. **Check baseline health**

   ```bash
   cargo xtask doctor
   cargo xtask ac-status
   ```

3. **Discover flows and tasks**

   ```bash
   cargo xtask help-flows
   cargo xtask tasks-list         # if present
   ```

4. **Pick a task and a Skill**

   - Identify the relevant Skill (feature, maintenance, release, governance debug).
   - Use bundles and suggest-next instead of scanning the whole repo.

---

## 7. Summary

This cell is designed for you to work **confidently and autonomously**:

- Specs and schemas tell you what the system is.
- BDD and tests verify that behaviour.
- xtask flows guide how work is done.
- Platform APIs and `/ui` reveal the same state that CI enforces.
- Selftest + CI are the guardrails that keep everything honest.

Your job is to:

- Use these contracts actively,
- Make reasonable, reversible decisions,
- Capture your reasoning and questions as artifacts,
- And finish work in a state where `cargo xtask selftest` can tell everyone "this is governed and ready to review."
