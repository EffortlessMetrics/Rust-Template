````markdown
# Rust-as-Spec Platform Cell (v3.3.1)

**A governed Rust service template where specs, tests, docs, policies, and infra all agree – and the repo can prove it.**

This cell gives you a **single Rust service** with:

- **Schema-driven specs** (`specs/spec_ledger.yaml`, `config_schema.yaml`)
- **BDD-backed acceptance criteria** (Cucumber + Rust)
- **A 7-step selftest gate** (`cargo xtask selftest`)
- **Introspection APIs** under `/platform/*`
- **A Web UI** at `/ui` that shows the same governance state CI enforces

Use it when you want a Rust service that **explains itself**, is **safe for LLMs/agents to work inside**, and **stays governed over time**.

---

## 1. Who this is for

Use this template if:

- You ship Rust services in **regulated or multi-team environments** (FinTech, health, platform engineering).
- You want **spec-as-code and doc-as-code** with real teeth: CI fails if specs/tests/docs drift.
- You plan to use **LLMs/agents as contributors** and need hard guardrails.
- You want a **repeatable service cell** to plug into a portal/IDP (Backstage/Port/Humanitec/etc.).

Don’t use this if:

- You just want “hello world in Axum”.
- You don’t care about AC traceability or policy tests yet.
- You’re experimenting and don’t want any governance overhead.

---

## 2. What you actually get

### 2.1 Specs and governance as code

- `specs/spec_ledger.yaml` – stories → requirements → acceptance criteria (ACs) → tests → docs.
- `specs/config_schema.yaml` – configuration schema for the service.
- `specs/devex_flows.yaml` – developer workflows (flows + commands).
- `specs/tasks.yaml` – work items the platform can surface via CLI and HTTP.

### 2.2 Verification

- **BDD (Cucumber + Rust)** for platform and DevEx behaviour.
- **Unit tests** for spec runtime, graph invariants, config validation, etc.
- `cargo xtask ac-status` – computes AC → test → status mapping and writes:

  - `docs/feature_status.md` – **AC health dashboard**, auto-generated.

### 2.3 Enforcement

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

### 2.4 Introspection surfaces

- **HTTP APIs (`/platform/*`):**
  - `/platform/status` – governance health, policy status, auth mode, metadata.
  - `/platform/graph` – full governance graph as JSON.
  - `/platform/tasks` – tasks from `tasks.yaml` (list + status updates).
  - `/platform/devex/flows` – developer flows and commands.
  - `/platform/docs/index` – documentation inventory.
  - `/platform/schema` – machine-readable schema/OpenAPI for the platform.
  - `/platform/agent/hints` – task suggestions for agents.

- **Web UI (`/ui`):**
  - Dashboard view for status and governance metrics.
  - Graph visualization (Mermaid) of stories/REQs/ACs/docs/commands.
  - Flows and tasks views for day-to-day work.

The UI has **no separate database**: it calls the same loaders `selftest` uses.  
If the UI shows it, CI enforces it.

---

## 3. High-level architecture

The pipeline is simple:

```text
Specs (YAML) ──> Loader (Rust) ──> Selftest (CI) ──> Introspection (HTTP/UI)
   spec_ledger      spec-runtime        xtask selftest      /platform/*, /ui
   config_schema    graph model
   devex_flows
   tasks, docs
````

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

* Linux, macOS, or **WSL2** on Windows.
* [Nix](https://nixos.org/) with flakes enabled.

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

* **Stories** (`US-*`) – user-facing goals.
* **Requirements** (`REQ-*`) – what must be true.
* **Acceptance criteria** (`AC-*`) – concrete behaviour.
* **Tests** – how each AC is verified (BDD tags, unit tests).
* **Docs** – which design/runbook/tutorial covers it.

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

* `PLATFORM_AUTH_MODE=none` (default)

  * All `/platform/*` routes are open.
  * Intended for local/dev use.

* `PLATFORM_AUTH_MODE=basic`

  * All **non-GET** `/platform/*` routes require a shared token.
  * Read endpoints (GET) remain open or can share the same guard.
  * If `basic` is enabled without a token, the service:

    * Logs a startup warning.
    * Surfaces `token_present: false` in `/platform/status`.

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

Everything runs through the `xtask` binary.

### 8.1 Onboarding & sanity

```bash
# Environment sanity check
cargo xtask doctor

# Quick code quality check (fmt, clippy, unit tests)
cargo xtask check

# Full governance gate
cargo xtask selftest
```

### 8.2 AC-first feature development

```bash
# 1. Define a new AC
cargo xtask ac-new AC-MYSERV-001 "Users can list todos" --requirement REQ-MYSERV-TODOS

# 2. Add or adjust BDD scenarios in specs/features/*.feature

# 3. Generate an LLM context bundle for the AC
cargo xtask bundle implement_ac

# 4. Implement the feature (by hand or with an LLM using the bundle)

# 5. Run just the relevant BDD tests
cargo xtask bdd

# 6. Update AC status and validate everything
cargo xtask ac-status
cargo xtask selftest
```

### 8.3 Selective testing

```bash
# Test only what changed vs origin/main (fast loop)
cargo xtask test-changed

# Plan-only mode (see what would be run)
XTASK_TEST_CHANGED_PLAN_ONLY=1 cargo xtask test-changed
```

### 8.4 Releases

```bash
# Build a local SBOM
cargo xtask sbom-local

# Prepare a release (version bump + docs)
cargo xtask release-prepare 3.3.1

# Generate release evidence bundle
cargo xtask release-bundle 3.3.1
# -> release_evidence/v3.3.1.md
```

---

## 9. LLM / agent ergonomics

This repo is designed for agents to do the mechanical work safely, and humans to review.

Key pieces:

* **Specs are structured** (ledger, flows, tasks).

* **Bundles are explicit** (`cargo xtask bundle implement_ac`).

* **Suggest-next is task-aware**:

  ```bash
  cargo xtask suggest-next --task IMPLEMENT_AC
  ```

* **Platform hints**:

  * `/platform/agent/hints` exposes high-priority tasks and recommended sequences.

LLMs should:

1. Use `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints` to understand the work.
2. Use `xtask` commands to make changes and run checks.
3. Let `selftest` be the final arbiter.

See:

* `CLAUDE.md` – system prompt and recommended workflows.
* `docs/AGENT_GUIDE.md` – how agents should operate in this repo.
* `docs/AGENT_SKILLS.md` – Skills and their mapping to flows.

---

## 10. Adoption patterns

### 10.1 Greenfield: new service

* Clone this template into a new repo.
* Adjust `service_metadata.yaml`, ledger entries, tasks to your domain.
* Keep the platform kernel as-is; build your business logic in new crates or modules.
* Use `selftest` as the gate from day one.

### 10.2 Brownfield: existing repo

See `docs/how-to/add-governance-to-existing-repo.md`.

At a high level:

* Add `governance/` subtree (specs, policy, docs) to your existing repo.
* Add `spec-runtime` + `xtask` crates to your workspace.
* Configure your CI to run `cargo xtask selftest` as a gate.
* Gradually map your existing tests/docs into the spec ledger.

---

## 11. Relationship to portals/IDPs

This template is **not** a portal. It is the **per-service kernel** that a portal/IDP can trust.

* If you use Backstage/Port/OpsLevel/etc.:

  * They provide catalogs, scorecards, golden paths.
  * This template defines what a “good Rust service” is in concrete, enforceable terms.

* If you use a platform orchestrator (Humanitec, Argo CD, etc.):

  * They standardize deployments and environments.
  * This template standardizes the **service contract**: specs, policies, AC coverage, `/platform/*` introspection.

The integration surface is `/platform/*` and the evidence bundles under `release_evidence/`.

---

## 12. Current status & roadmap

* **Template version:** v3.3.1
* **Kernel status:** “Governing Kernel” is frozen – the core contract is in place and validated via selftest.
* **Known non-kernel ACs:** Some advanced features (idempotent flows, question artifacts, K8s/TF IaC alignment) are tracked as ACs with `[UNKNOWN]` status and explicitly documented as such.

See:

* `docs/ROADMAP.md` – where the template is headed.
* `docs/feature_status.md` – current AC health.
* `docs/feature_status_notes.md` – explanations for template/future ACs.

---

## 13. License

This template is dual-licensed:

* MIT – see `LICENSE-MIT`
* Apache 2.0 – see `LICENSE-APACHE`

You may use it under either license.

---

```
::contentReference[oaicite:0]{index=0}
```
