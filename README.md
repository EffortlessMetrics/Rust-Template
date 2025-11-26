# Rust-as-Spec Platform Cell (v3.3.3)

**Template Version:** v3.3.3 | **Kernel Baseline:** [v3.3.3-kernel](./docs/KERNEL_SNAPSHOT.md)

**A governed Rust service template where specs, tests, docs, policies, and infra all agree – and the repo can prove it.**

> **Using this as a template?** Start here:
> - [Kernel snapshot](docs/KERNEL_SNAPSHOT.md) – what you're inheriting
> - [New service guide](docs/how-to/new-service-from-template.md) – step-by-step setup
> - Run `cargo xtask kernel-smoke` after cloning – it should be green

This cell gives you a **single Rust service** with:

- **Schema-driven specs** (`specs/spec_ledger.yaml`, `specs/config_schema.yaml`)
- **BDD-backed acceptance criteria** (Cucumber + Rust)
- **An 8-step selftest gate** (`cargo xtask selftest`)
- **Introspection APIs** under `/platform/*`
- **A Web UI** at `/ui` that shows the same governance state CI enforces

Use it when you want a Rust service that:

- **Explains itself** through specs and schemas
- Is **safe for humans and agents to work inside autonomously**
- **Stays governed over time** by design, not by habit

---

## 1. Who this is for

Use this template if:

- You ship Rust services in **regulated or multi-team environments** (FinTech, health, platform/platform-engineering).
- You want **spec-as-code and doc-as-code** with real teeth: CI fails if specs/tests/docs drift.
- You plan to use **LLMs/agents as active contributors**, not just copilots.
- You want a **repeatable service cell** to plug into a portal/IDP (Backstage/Port/Humanitec/etc.).

Don’t use this if:

- You just need a quick Axum “hello world”.
- You’re experimenting without any governance requirements.
- You don’t care about AC traceability or policy tests yet.

---

## 2. Quick Start

**For humans getting oriented:**
```bash
nix develop                    # Enter the dev shell (installs all tools)
cargo xtask doctor             # Check your environment
cargo xtask selftest           # Run full governance validation
cargo run -p app-http          # Start the service (http://localhost:8080)
```

Visit http://localhost:8080/ui to see the governance dashboard.

**For agents working autonomously:**
```bash
cargo xtask help-flows         # Discover available flows and commands
cargo xtask ac-status          # View AC health
cargo xtask bundle implement_ac # Generate LLM context for implementation
```

See [CLAUDE.md](./CLAUDE.md) for the full agent guide and [docs/how-to/new-service-from-template.md](./docs/how-to/new-service-from-template.md) to fork this template.

---

## 3. What you actually get

### 3.1 Specs and governance as code

- `specs/spec_ledger.yaml` – stories → requirements → acceptance criteria (ACs) → tests → docs.
- `specs/config_schema.yaml` – configuration schema for the service.
- `specs/devex_flows.yaml` – developer workflows (flows + commands).
- `specs/tasks.yaml` – work items the platform can surface via CLI and HTTP.

### 3.2 Verification

- **BDD (Cucumber + Rust)** for platform and DevEx behaviour.
- **Unit tests** for spec runtime, graph invariants, config validation, etc.
- `cargo xtask ac-status` – computes AC → test → status mapping and writes:
  - `docs/feature_status.md` – **AC health dashboard**, auto-generated.

### 3.3 Enforcement

- `cargo xtask selftest` – the **single mandatory CI gate**:
  1. Core checks (fmt, clippy, unit tests)
  2. BDD acceptance tests
  3. AC status + ADR mapping
  4. LLM context bundler checks
  5. Policy tests (OPA/Conftest)
  6. DevEx contract (required xtask commands, flows)
  7. Governance graph invariants (REQ/AC/command connectivity)
  8. AC coverage sanity

If selftest is red, the service is **not** in a governed state.

### 3.4 Introspection surfaces

- **HTTP APIs (`/platform/*`):**
  - `/platform/status` – governance health, policy status, auth mode, metadata.
  - `/platform/graph` – full governance graph as JSON.
  - `/platform/tasks` – tasks from `tasks.yaml` (list + status updates).
  - `/platform/devex/flows` – developer flows and commands.
  - `/platform/docs/index` – documentation inventory.
  - `/platform/schema` – machine-readable schema/OpenAPI for the platform.
  - `/platform/agent/hints` – suggestions for where to work next.

- **Web UI (`/ui`):**
  - Dashboard view for status and governance metrics.
  - Graph visualization (Mermaid) of stories/REQs/ACs/docs/commands.
  - Flows and tasks views for day-to-day work.

The UI has **no separate database**: it calls the same loaders `selftest` uses.  
If the UI shows it, CI enforces it.

---

## 4. High-level architecture

The pipeline is:

```text
Specs (YAML) ──> Loader (Rust) ──> Selftest (CI) ──> Introspection (HTTP/UI)
   spec_ledger      spec-runtime        xtask selftest      /platform/*, /ui
   config_schema    graph model
   devex_flows
   tasks, docs
```

Under the hood:

```text
┌─────────────────────────────────────┐
│ Adapters (HTTP, gRPC)              │
│  ├─ app-http (Axum)                │
│  └─ adapters-grpc (Tonic)          │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Business Core                       │
│  ├─ business-core (domain)         │
│  ├─ governance (specs + policies)  │
│  └─ telemetry (metrics + tracing)  │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Infrastructure                      │
│  ├─ spec-runtime (loaders + graph) │
│  ├─ model (shared types)           │
│  └─ proto (gRPC definitions)       │
└─────────────────────────────────────┘
```

---

## 4. Quick start

### 4.1 Tier-1 prerequisites (recommended)

For **Tier-1** parity (same environment as CI):

- Linux, macOS, or **WSL2** on Windows.
- [Nix](https://nixos.org/) with flakes enabled.

This repo is Nix-first. You *can* run it without Nix, but you lose exact CI parity.

### 4.2 Getting up and running

```bash
# Clone the template
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service

# Enter the Nix dev shell (Tier-1)
nix develop

# Check your environment
cargo xtask doctor

# Install hooks (optional but recommended)
cargo xtask install-hooks

# Run the full selftest gate
cargo xtask selftest
```

Start the service:

```bash
cargo run -p app-http
# UI:   http://localhost:8080/ui
# APIs: http://localhost:8080/platform/status
```

---

## 5. Platform support

| Platform | Tier | Environment  | Selftest | Notes                         |
| -------- | ---- | ------------ | -------- | ----------------------------- |
| Linux    | 1    | Nix devshell | ✅        | Canonical dev + CI            |
| macOS    | 1    | Nix devshell | ✅        | Cross-platform development    |
| WSL2     | 1    | Nix in WSL2  | ✅        | Recommended for Windows users |
| Windows  | 2    | Native       | ⚠️       | fmt/policy may be skipped     |

**Tier-1**: full `selftest` including fmt, policy tests, and all gates.
**Tier-2**: core functionality works, but file locking and toolchain differences mean some steps may be skipped or run with `XTASK_LOW_RESOURCES=1`.

See `docs/reference/platform-support.md` for details and recommended commands per tier.

---

## 6. Governance model in practice

### 6.1 Spec ledger

`specs/spec_ledger.yaml` is the core artifact:

- **Stories** (`US-*`) – user-facing goals.
- **Requirements** (`REQ-*`) – what must be true.
- **Acceptance criteria** (`AC-*`) – concrete behaviour.
- **Tests** – how each AC is verified (BDD tags, unit tests).
- **Docs** – which design/runbook/tutorial covers it.

Example fragment:

```yaml
- id: REQ-TPL-PLATFORM-AUTH
  title: "Platform introspection supports authenticated mode"
  must_have_ac: true
  acceptance_criteria:
    - id: AC-TPL-PLATFORM-AUTH-BASIC
      text: >
        When PLATFORM_AUTH_MODE=basic, write endpoints under /platform/*
        reject unauthenticated requests with 401/403 and accept requests
        with the configured credential header; read endpoints may remain
        open or use the same guard.
      tests:
        - { type: bdd, tag: "@AC-TPL-PLATFORM-AUTH-BASIC", file: "specs/features/platform_security.feature" }
```

### 6.2 AC status and coverage

To see current coverage:

```bash
# Recompute AC statuses from tests
cargo xtask ac-status

# Open the generated dashboard
less docs/feature_status.md
```

Kernel ACs must be `[PASS]`. Non-kernel or template-only ACs may be `[UNKNOWN]` and are documented as such.

---

## 7. Platform APIs and auth

### 7.1 Auth modes

Auth is applied once at the `/platform/*` router:

- `PLATFORM_AUTH_MODE=none` (default)
  - All `/platform/*` routes are open.
  - Intended for local/dev use.

- `PLATFORM_AUTH_MODE=basic`
  - All **non-GET** `/platform/*` routes require a shared token.
  - Read endpoints (GET) remain open or can share the same guard.
  - If `basic` is enabled without a token, the service:
    - Logs a startup warning.
    - Surfaces `token_present: false` in `/platform/status`.

Typical production setup:

```bash
export PLATFORM_AUTH_MODE=basic
export PLATFORM_AUTH_TOKEN="some-long-random-secret"
```

### 7.2 Key endpoints

```bash
# Governance health + metadata
curl http://localhost:8080/platform/status

# Full governance graph (stories/REQs/ACs/docs/commands)
curl http://localhost:8080/platform/graph

# Tasks from specs/tasks.yaml
curl http://localhost:8080/platform/tasks

# DevEx flows
curl http://localhost:8080/platform/devex/flows

# Docs index
curl http://localhost:8080/platform/docs/index

# Schema/OpenAPI for the platform
curl http://localhost:8080/platform/schema
```

---

## 8. Developer workflows (xtask)

Everything runs through the `xtask` binary. It’s designed to be friendly for humans and agents.

### 8.1 Onboarding & sanity

```bash
# Environment sanity check
cargo xtask doctor

# Quick code quality check (fmt, clippy, unit tests)
cargo xtask check

# Full governance gate (Tier-1)
cargo xtask selftest
```

### 8.2 AC-first feature development

```bash
# 1. Create a new AC
cargo xtask ac-new AC-MYSERV-001 "Users can list todos" --requirement REQ-MYSERV-TODOS

# 2. Edit the spec ledger to add context
#    (specs/spec_ledger.yaml is updated by step 1)

# 3. Create a BDD scenario
#    Edit specs/features/todos.feature and tag with @AC-MYSERV-001

# 4. Generate an LLM context bundle
cargo xtask bundle implement_ac

# 5. Implement the feature
#    (Use the bundle with your own editor or an LLM)

# 6. Run focused tests
cargo xtask test-ac AC-MYSERV-001
cargo xtask test-changed

# 7. Verify governance
cargo xtask ac-status
cargo xtask selftest
```

### 8.3 Selective testing

```bash
# Test only what changed vs origin/main (fast loop)
cargo xtask test-changed

# Plan-only mode (see what would be run)
XTASK_TEST_CHANGED_PLAN_ONLY=1 cargo xtask test-changed

# Test a specific acceptance criterion
cargo xtask test-ac AC-PLT-001
```

### 8.4 Releases

```bash
# Build a local SBOM
cargo xtask sbom-local

# Prepare a release (version bump + docs)
cargo xtask release-prepare 3.3.1

# Generate release evidence
cargo xtask release-bundle 3.3.1
# -> release_evidence/v3.3.1.md
```

---

## 9. LLM / agent ergonomics

This repo is designed so that an agent can act as a **real teammate**, not a glorified autocomplete:

- **Specs & schemas** give a precise brief (`spec_ledger.yaml`, `config_schema.yaml`, `devex_flows.yaml`, `tasks.yaml`).
- **Bundles** give a bounded context:
  - `cargo xtask bundle implement_ac` produces a curated context pack for a feature or AC.
- **Flows & Skills** provide the patterns:
  - `.claude/skills/*/SKILL.md` describe governed-feature-dev, governed-maintenance, governed-release, and governance-debug flows.
- **Platform APIs** provide live telemetry:
  - `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints`, `/platform/docs/index`, `/platform/schema`.

The expected agent loop looks like:

1. **Orient**

   - Run `cargo xtask doctor`, `cargo xtask ac-status`, `cargo xtask help-flows`.
   - Call `/platform/status` and `/platform/graph` to understand the current state.

2. **Pick work**

   - Use `cargo xtask tasks-list` and `/platform/agent/hints` to identify a task.
   - Read the relevant REQ/AC entries in `spec_ledger.yaml`.

3. **Plan with a bundle**

   - Generate a bundle with `cargo xtask bundle implement_ac` or a task-specific bundle.
   - Use only what's in the bundle plus linked specs/docs unless you have a good reason to widen scope.

4. **Execute via xtask and code**

   - Follow the appropriate Skill (feature dev, maintenance, release).
   - Use `test-changed` and `test-ac` while iterating.

5. **Validate and capture decisions**

   - End with `cargo xtask selftest` in Tier-1.
   - If you had to make non-obvious choices, record them:
     - draft ADRs,
     - GitHub issues,
     - friction log entries.

Agents don’t need synchronous human approval to move forward; the spec ledger, flows, xtask commands, `/platform/*` APIs, and selftest provide the guardrails. Humans review the artifacts and CI results asynchronously.

For details, see:

- `CLAUDE.md` – agent operational prompt
- `docs/AGENT_GUIDE.md` – deeper guidance for agent-driven work
- `docs/SELECTIVE_TESTING.md` – validation ladder and change-aware testing

---

## 10. Adoption patterns

### 10.1 Greenfield: new service

- Clone this template into a new repo.
- Adjust `service_metadata.yaml`, ledger entries, and tasks to your domain.
- Keep the platform kernel as-is; build your business logic in new crates or modules.
- Use `selftest` as the gate from day one.

### 10.2 Brownfield: existing repo

See `docs/how-to/add-governance-to-existing-repo.md`.

High-level flow:

- Add a `governance/` subtree (specs, policy, docs) to your existing repo.
- Add `spec-runtime` + `xtask` crates to your workspace.
- Configure your CI to run `cargo xtask selftest` as a gate.
- Gradually map your existing tests/docs into the spec ledger.

---

## 11. Relationship to portals/IDPs

This template is **not** a portal. It is the **per-service kernel** that a portal/IDP can rely on.

- If you use Backstage/Port/OpsLevel/etc.:
  - They provide catalogs, scorecards, golden paths.
  - This template defines what a "good Rust service" is in concrete, enforceable terms.

- If you use a platform orchestrator (Humanitec, Argo CD, etc.):
  - They standardize deployments and environments.
  - This template standardizes the **service contract**: specs, policies, AC coverage, `/platform/*` introspection.

The integration surface is:

- `/platform/*` APIs
- `release_evidence/vX.Y.Z.md` bundles
- `docs/feature_status.md` and `service_metadata.yaml`

---

## 12. Current status & roadmap

- **Template version:** v3.3.3
- **Kernel status:** All 65 ACs pass. All 8 selftest gates pass.
- **Known gaps:** Branch protection, IDP docs, second service validation (see ROADMAP.md)

See:

- `docs/ROADMAP.md` – current state and path forward options
- `docs/KERNEL_SNAPSHOT.md` – frozen baseline details
- `docs/feature_status.md` – AC health dashboard

---

## 13. License

This template is dual-licensed:

- MIT – see `LICENSE-MIT`
- Apache 2.0 – see `LICENSE-APACHE`

You may use it under either license.
