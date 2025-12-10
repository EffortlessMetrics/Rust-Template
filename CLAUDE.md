# CLAUDE.md – Test Service (v3.3.8)

**Template Version:** v3.3.8

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

- **Specs and schemas are your brief** – everything flows from `spec_ledger.yaml`, `config_schema.yaml`, `devex_flows.yaml`.
- **Flows and skills are your playbook** – `.claude/skills/*.md` describe the governed patterns you follow.
- **Platform APIs are your telemetry** – `/platform/*` endpoints give you real-time state visibility.
- **xtask is your control surface** – commands for every workflow, from onboarding to release.
- **Selftest is your referee** – 11-step governance gate that decides what's acceptable.
- **Friction log is your feedback** – capture DevEx issues as they arise; don't accumulate pain.

You can make decisions locally. When something is unclear, you:

1. Infer the best option from existing contracts (specs, ADRs, patterns).
2. Capture your reasoning in a draft ADR, GitHub issue, or friction log entry.
3. Keep the repo as close to selftest-green as possible.
4. Leave a clear trail so humans can review artifacts asynchronously.

---

## 2. Primary sources of truth

When you need to know “what is correct?”, look in this order:

1. **Spec ledger & schemas**

   - `specs/spec_ledger.yaml` – stories → REQs → ACs → tests → docs.
   - `specs/config_schema.yaml` – configuration contract.
   - `specs/devex_flows.yaml` – developer workflows.
   - `specs/tasks.yaml` – work items and their relationships.

2. **Skills (governed workflows)**

   - `.claude/skills/*/SKILL.md` are workflow recipes that are governed:
     - **bootstrap-dev-env** – one-command environment setup and validation
     - **governed-feature-dev** – AC-first feature development workflow
     - **governed-maintenance** – fixing drift, updating deps, and health checks
     - **governed-release** – preparing, validating, and publishing releases
     - **governed-governance-debug** – debugging selftest failures and governance issues
   - All Skills are validated by `cargo xtask skills-lint` for name format, description quality, allowed-tools safety, and no hardcoded secrets.
   - See `docs/SKILLS_GOVERNANCE.md` and `docs/SKILLS_TEMPLATE.md` for creation rules.

3. **Agents (governed LLM agents)**

   - `.claude/agents/*.md` are first-class, governed artifacts defining long-lived, specialized agents.
   - Each agent has:
     - **name** – kebab-case, unique, max 64 chars
     - **description** – what the agent does + when to use it (≤1024 chars)
     - **tools** – explicit tool list (least-privilege)
     - **permissionMode** – `restricted` or `permissive` with justification
     - **model** – agent model preference (default: `inherit`)
     - **skills** – optional list of Skill names to include
     - **system** – optional system prompt for LLM context
   - All agents are validated by `cargo xtask agents-lint` for name format, descriptions, tools, model policy, skill references, and no hardcoded secrets.
   - See `docs/AGENTS_GOVERNANCE.md` and `docs/AGENTS_TEMPLATE.md` for creation rules and validation expectations.
   - **Do not include secrets, API keys, tokens in agent definitions.** Use environment variables and configuration files.

4. **Platform APIs**

   - **Governance & discovery:**
     - `/platform/status` – governance health, policy status, auth mode, metadata.
     - `/platform/graph` – full governance graph (stories → REQs → ACs → tests → docs).
     - `/platform/schema` – machine-readable schema/OpenAPI for the platform.
     - `/platform/devex/flows` – developer flows and xtask commands.
     - `/platform/docs/index` – documentation inventory.
     - `/platform/coverage` – AC coverage summary (BDD + test results).
   - **Work & tasks:**
     - `/platform/tasks` – tasks from specs/tasks.yaml (with filtering).
     - `/platform/tasks/suggest-next` – recommended next work (given a task).
     - `/platform/tasks/graph` – task dependencies (JSON or Mermaid).
   - **Metadata & issues:**
     - `/platform/agent/hints` – prioritized work suggestions for agents (Todo/InProgress tasks).
     - `/platform/friction` – development friction log (DevEx issues).
     - `/platform/questions` – design questions and ambiguities.
     - `/platform/forks` – fork/branch information.

5. **xtask CLI**

   - **Bootstrap:** `dev-up` (one-command setup), `doctor`, `kernel-smoke`, `install-hooks`
   - **Sanity:** `check`, `selftest`, `precommit`, `ci-local`
   - **Governance:** `skills-lint`, `skills-fmt`, `agents-lint`, `agents-fmt`, `ac-status`, `ac-coverage`
   - **AC-first development:** `ac-new`, `ac-suggest-scenarios`, `ac-tests`, `test-ac`, `bundle`, `test-changed`
   - **Design & docs:** `adr-new`, `adr-check`, `design-new`, `docs-check`, `spellcheck`
   - **Friction & feedback:** `friction-list` (DevEx issues, process problems)
   - **Release:** `release-prepare`, `release-bundle`, `sbom-local`
   - **Exploration:** `help-flows`, `tasks-list`, `suggest-next` (agent guidance)

6. **Selftest & CI**

   - `cargo xtask selftest` – full governance check.
   - CI Tier-1 job running selftest – final say on what is acceptable.

When in doubt, align your choices with `spec_ledger.yaml` and “what would make selftest green”.

---

## 3. Core workflows

### 3.1 Orient yourself

On first contact, or when resuming work:

**One-command setup (recommended):**

```bash
cargo xtask dev-up         # Bootstrap env, check health, install hooks, run quick tests
```

**Or step-by-step:**

```bash
cargo xtask doctor         # Environment sanity check
cargo xtask kernel-smoke   # Quick smoke test of template baseline
cargo xtask ac-status      # Current AC coverage
cargo xtask help-flows     # Available flows and commands
```

Then use introspection endpoints to understand the repo state. Start the service:

```bash
cargo run -p app-http &   # Start HTTP service on http://localhost:8080
```

Then query the platform:

```bash
# Governance health and counts (status, AC coverage, token config)
curl http://localhost:8080/platform/status

# Full governance graph (stories → REQs → ACs → tests → docs → commands)
curl http://localhost:8080/platform/graph

# Developer flows and available xtask commands
curl http://localhost:8080/platform/devex/flows

# Documentation inventory (design docs, ADRs, how-tos)
curl http://localhost:8080/platform/docs/index

# All tasks with optional filtering
curl http://localhost:8080/platform/tasks?status=Todo

# Prioritized next work for agents (Todo/InProgress with RC/AC IDs)
curl http://localhost:8080/platform/agent/hints

# Development friction log (DevEx issues, process problems)
curl http://localhost:8080/platform/friction

# Design questions and ambiguities
curl http://localhost:8080/platform/questions
```

This gives you a complete picture: health, governance graph, available flows, docs, tasks, recommended work, and DevEx feedback.

**Prefer introspection endpoints over manual grepping.** They are:

- **Authoritative:** generated from the same specs that CI enforces.
- **Fast:** computed at load time and cached.
- **Agent-friendly:** structured JSON that's easy to parse and act on.

See `docs/AGENT_GUIDE.md` for deeper guidance on using these APIs.

---

### 3.2 Governed feature development

Use this shape whenever you implement or change behaviour.

#### 1. Start from REQ + AC

- Find the relevant REQ/AC in `specs/spec_ledger.yaml`.
- If the AC doesn't exist and the REQ is clear:
  - Create it via `cargo xtask ac-new <AC_ID> <DESCRIPTION> --story <STORY> --requirement <REQUIREMENT>`
    (e.g., `cargo xtask ac-new AC-MYSERV-001 "Users can list todos" --story US-MYSERV-001 --requirement REQ-MYSERV-TODOS`)
  - Or edit `specs/spec_ledger.yaml` directly following existing conventions.
  - Keep it small and precise.

#### 2. Add or update BDD

- Use `cargo xtask ac-suggest-scenarios AC-MYSERV-001` for interactive scenario suggestions.
- Edit `specs/features/*.feature` and tag scenarios with `@AC-MYSERV-001`.
- Run `cargo xtask bdd` to validate Gherkin syntax.

#### 3. Generate a bundle

```bash
cargo xtask bundle implement_ac
# Task name from .llm/contextpack.yaml; other tasks available for different contexts
```

Bundle output structure:

- `bundle/<TASK>/bundle.yaml` – manifest with task_id, requirement_ids, ac_ids, specs, docs, tests
- `bundle/<TASK>/context.md` – markdown-formatted bundled files

Workflow:

- Read `bundle.yaml` to understand task scope and dependencies
- Use `context.md` as your primary working context
- Prefer staying within the bundle instead of scanning the entire repo
- See `.llm/contextpack.yaml` for available bundle tasks (implement_ac, debug_tests, etc.).
- **Note:** Bundles are ephemeral. The `bundle/` directory is ignored by git and regenerated on-demand with timestamps and git SHAs baked in. Only the **contract** (specs, ACs, BDD, tests, docs) is versioned; validate everything via tests and selftest, not by checking in bundle artifacts.

#### 4. Implement code + tests

- Keep changes scoped to what the AC needs.
- Maintain alignment with the spec and existing patterns.
- Run `cargo xtask ac-tests AC-MYSERV-001` to see all tests mapped to this AC.

#### 5. Validate with the ladder

```bash
cargo xtask check              # fmt, clippy, unit tests (fast)
cargo xtask test-changed       # Only changed code
cargo xtask test-ac AC-MYSERV-001  # AC + related tests
cargo xtask ac-status          # AC health mapping
cargo xtask selftest           # Full governance gate (before PR)
```

- Aim to finish with selftest green.
- If selftest is red for reasons you can't safely resolve, capture why in an ADR or issue (see Section 5: Handling ambiguity and decisions).

**About git pre-commit:**

The repository installs a git pre-commit hook (`.git/hooks/pre-commit`) that:

1. Ensures you are inside `nix develop` (so rustc, sccache, and libs are correct).
2. Calls `cargo xtask precommit`, which automatically:
   - Runs `cargo fmt --all` and **auto-stages** any formatted Rust files
   - Runs **Skills format** and **auto-stages** formatted SKILL.md files
   - Runs **clippy + tests** (the same verification gate as CI)
   - Runs **Skills + Agents governance checks** (hard gates; these block if violated)
   - Regenerates and **auto-stages** `docs/feature_status.md` (soft; warns if ACs are failing)
   - Runs **docs-check** and **spellcheck** (soft by default; only hard if `XTASK_STRICT_PRECOMMIT=1`)

This means:

- **Mechanical issues are silently auto-fixed and staged** (formatting, SKILL.md tidiness, AC status report).
- **Only real governance failures block the commit** (test failures, clippy violations, Skills/Agents policy breaches).

You do **not** need to modify the hook or run the `pre-commit` tool directly; just `git commit` normally. If it fails, run `cargo xtask precommit` to see what broke, fix it, and commit again.

**In CI**, we run `cargo xtask check` (no auto-fix) and `cargo xtask selftest` (full governance), so CI checks what is truly necessary without the auto-fix convenience.

---

### 3.3 Governed maintenance

Use this when you're fixing drift, updating dependencies, handling tool feedback, or resolving DevEx friction.

#### 1. Run health checks

```bash
cargo xtask doctor         # Environment check
cargo xtask check          # fmt, clippy, unit tests
cargo xtask test-changed   # Only changed code
cargo xtask ac-status      # AC → test mapping
cargo xtask friction-list  # DevEx issues to fix
```

#### 2. Apply clear fixes

- Align config with `config_schema.yaml`.
- Fix tests and specs where behaviour is clearly wrong.
- Update docs when they no longer match code or ACs.
- Fix DevEx issues captured in the friction log.

#### 3. Capture non-trivial findings

- If you discover deeper design questions or tradeoffs:
  - **For design decisions:** Create or update an ADR (`docs/adr/ADR-*.md`)
    - Use `cargo xtask adr-new "Title"` to create new ADR with correct frontmatter.
    - Link to relevant REQ/AC IDs in the ADR.
  - **For feature work:** File a GitHub issue linking REQ/AC IDs.
  - **For process/tooling friction:** Add to friction log via `FRICTION_LOG.md` or create entry directly in `friction/FRICTION-*.yaml`
    - Use `cargo xtask friction-list --status open` to view current issues.
    - Surfaces via CLI and `/platform/friction` API.

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

### 3.5 Writing and maintaining Skills & Agents

#### Skills (workflow recipes)

When creating or updating a Skill (`.claude/skills/SKILL_NAME/SKILL.md`):

1. **Name**: kebab-case, lowercase, max 64 chars, unique.
2. **Description**: include both WHAT (capability) and WHEN (context/triggers) in ≤1024 chars.
3. **Allowed tools**: follow least-privilege (don't grant Write/Edit to read-only workflows).
4. **Linked flows**: reference existing flows in `specs/devex_flows.yaml`.
5. **No secrets**: never hardcode API keys, tokens, or credentials. Use environment variables or pass config at runtime.
6. **Validate**: `cargo xtask skills-lint` will check format, descriptions, tools, and detect hardcoded secrets.

Template: see `docs/SKILLS_TEMPLATE.md` for copy-paste starting point.

#### Agents (governed LLM agents)

When creating or updating an Agent (`.claude/agents/AGENT_NAME.md`):

1. **Name**: kebab-case, lowercase, max 64 chars, unique within project.
2. **Description**: include both WHAT (capability) and WHEN (use case) in ≤1024 chars.
3. **Tools**: be explicit and least-privilege. Only grant tools the agent actually needs.
4. **PermissionMode**: `restricted` (default, safe) or `permissive` (with justification).
5. **Model**: prefer `inherit` (use repo default) or explicitly name an approved model.
6. **Skills**: reference existing Skills from `.claude/skills/*` or omit.
7. **System prompt**: optional; keep secure and don't include secrets.
8. **No secrets**: never hardcode API keys, tokens, credentials. Use environment variables.
9. **Validate**: `cargo xtask agents-lint` will check format, descriptions, tools, model, skill refs, and detect hardcoded secrets.

Template: see `docs/AGENTS_TEMPLATE.md` for copy-paste starting point.

**Security reminder for both Skills and Agents:**

- `skills-lint` and `agents-lint` will **error** on common secret patterns (API_KEY=, password:, token:, etc.).
- Best practice: phrase docs like "reads the API key from the `API_KEY` environment variable" instead of embedding examples with real or fake keys.
- See `docs/SKILLS_GOVERNANCE.md` and `docs/AGENTS_GOVERNANCE.md` for full rules.

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

When you find unclear or conflicting information (REQs, ACs, docs, or DevEx issues):

1. **Analyse using existing contracts**

   - Look at `spec_ledger.yaml`, ADRs, and any relevant design docs.
   - Prefer consistency with existing patterns and governance rules.

2. **Choose a conservative, reversible option**

   - Prefer small, easily revertible changes over big bets.
   - Avoid breaking public contracts unless the ACs clearly demand it.
   - If unsure, ask clarifying questions via GitHub issues or ADRs.

3. **Capture the decision and questions appropriately**

   - **Design/architectural decisions:** Draft or extend an ADR (`docs/adr/ADR-*.md`)
     - Use `cargo xtask adr-new "Title"` to create with correct frontmatter.
     - Can mark as "DRAFT" if incomplete; link to relevant REQ/AC IDs.
     - Humans review these for direction alignment.
   - **Feature/business work:** Open a GitHub issue linking REQ/AC IDs with:
     - Your understanding of the options.
     - The direction you took and why.
     - Any blockers or open questions.
   - **Process/tooling friction:** Log in friction system:
     - `FRICTION_LOG.md` for human-readable summary.
     - `friction/FRICTION-*.yaml` for structured tracking.
     - Surfaces via `cargo xtask friction-list` and `/platform/friction` API.
   - **Design questions/ambiguities:** File via GitHub issues or note in comments.

4. **Align with selftest**

   - Aim for selftest green after your changes.
   - If you must leave selftest red (e.g., mid-refactor):
     - The failure must be clearly attributable to a known gap.
     - Document the gap and your plan in ADR/issue/friction log.
     - Add a comment in the code pointing to the issue.

This lets humans review your work asynchronously with full context, without blocking your progress. The repo's governance artifacts (specs, ADRs, issues, friction log) become the memory of why decisions were made.

---

## 6. First-run orientation

When you are first invoked on this repo:

1. **Read the high-level docs**

   - `docs/why-this-exists.md`
   - `docs/ROADMAP.md`
   - `docs/AGENT_GUIDE.md`
   - `docs/MISSING_MANUAL.md`
   - `docs/explanation/TEMPLATE-CONTRACTS.md`
   - `docs/how-to/ai-first-hour.md` – your first-hour script for this cell
   - `docs/how-to/trust-a-cell.md` – how to verify the cell is safe to work in
   - `docs/how-to/evolve-the-kernel.md` – what to do if a contract needs to change

2. **Check baseline health**

   ```bash
   cargo xtask doctor
   cargo xtask ac-status
   ```

3. **Discover flows and tasks**

   ```bash
   cargo xtask help-flows
   cargo xtask tasks-list         # List available tasks
   ```

4. **Pick a task and a Skill**

   - **For first-time setup:** Use the `bootstrap-dev-env` skill.
   - **For regular work:** Identify the relevant Skill (feature-dev, maintenance, release, governance-debug).
   - Use bundles and `/platform/agent/hints` instead of scanning the whole repo.

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
