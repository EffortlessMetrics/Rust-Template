# Test Service (v3.3.3)

**Template Version:** v3.3.3 | **Kernel Baseline:** [v3.3.3-kernel](./docs/KERNEL_SNAPSHOT.md)

**A governed Rust service template where specs, tests, docs, policies, and infra all agree – and the repo can prove it.**

> **Using this as a template?** Start here:
> - **[Quick Start Guide](docs/QUICKSTART.md) – Get productive in 15 minutes** ⚡
> - **[Pre-Fork Checklist](docs/how-to/pre-fork-checklist.md) – Validate before forking** ✓
> - [Kernel snapshot](docs/KERNEL_SNAPSHOT.md) – what you're inheriting
> - [New service guide](docs/how-to/new-service-from-template.md) – step-by-step setup
> - [Troubleshooting Guide](docs/TROUBLESHOOTING.md) – when things go wrong
> - Run `cargo xtask kernel-smoke` after cloning – it should be green

A test service

- **Schema-driven specs** (`specs/spec_ledger.yaml`, `specs/config_schema.yaml`)
- **BDD-backed acceptance criteria** (Cucumber + Rust)
- **A 10-step selftest gate** (`cargo xtask selftest`)
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
cargo xtask bundle implement_ac # Generate LLM context (task from .llm/contextpack.yaml)
```

See:
- [QUICKSTART.md](./docs/QUICKSTART.md) for a 15-minute onboarding guide
- [CLAUDE.md](./CLAUDE.md) for the full agent guide
- [docs/how-to/new-service-from-template.md](./docs/how-to/new-service-from-template.md) to fork this template
- [examples/fork-customization](./examples/fork-customization/) for sample customization files
- [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) if you encounter problems

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

- `cargo xtask selftest` – the **single mandatory CI gate** (10 steps):
  1. Core checks (fmt, clippy, unit tests)
  2. Skills governance lint
  3. Agents governance lint
  4. BDD acceptance tests
  5. AC status + ADR mapping
  6. LLM context bundler checks
  7. Policy tests (OPA/Conftest)
  8. DevEx contract (required xtask commands, flows)
  9. Governance graph invariants (REQ/AC/command connectivity)
  10. AC coverage sanity

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

## 5. Detailed Setup Guide

### 5.1 Prerequisites

**Required:** [Nix with flakes](https://nixos.org/download.html) (Tier-1, recommended for exact CI parity)

**Optional:** Docker, WSL2 (if on Windows), VS Code (recommended editor)

**Note:** You can develop without Nix (Tier-2), but policy tests and exact CI parity require it. See [docs/reference/environment.md](docs/reference/environment.md) for detailed setup by platform and tier.

### 5.2 Getting up and running (5 minutes)

```bash
# Clone the template
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service

# Enter the Nix dev shell (Tier-1)
nix develop

# One-command environment setup and validation (recommended)
cargo xtask dev-up
# OR: Use the Claude Code skill directly
# /bootstrap-dev-env (requires Claude Code integration)
```

Alternatively, if you prefer manual setup:

```bash
# Check your environment
cargo xtask doctor

# Quick smoke test (validates template baseline – do this first!)
cargo xtask kernel-smoke
```

**Expected:** All checks green. If any fail, troubleshoot before proceeding.

Then:

```bash
# Install hooks (optional but recommended)
cargo xtask install-hooks

# Run the full selftest gate (before submitting PR)
cargo xtask selftest
```

Start the service:

```bash
cargo run -p app-http
# UI:   http://localhost:8080/ui
# APIs: http://localhost:8080/platform/status
```

**VS Code users:** Press `F5` to run with debugger, or `Ctrl+Shift+B` to run `kernel: smoke` (default build task). See [QUICKSTART.md](./docs/QUICKSTART.md#editor-setup-vs-code) for full editor setup.

---

## 6. Platform support

| Platform | Tier | Environment  | Selftest | Notes                                      |
| -------- | ---- | ------------ | -------- | ------------------------------------------ |
| Linux    | 1    | Nix devshell | ✅        | Canonical dev + CI environment             |
| macOS    | 1    | Nix devshell | ✅        | Intel and Apple Silicon supported          |
| WSL2     | 1    | Nix in WSL2  | ✅        | Recommended for Windows; use Tier-1 setup  |
| Windows  | 2    | Native       | ⚠️       | Core features work; some tools skipped     |

**Tier-1**: Full `selftest` with all gates (fmt, policy, clippy). Same environment as CI.
**Tier-2**: Core functionality works. Some tools skipped. Use `XTASK_LOW_RESOURCES=1` if needed.

For detailed setup by platform and troubleshooting, see **[docs/reference/environment.md](docs/reference/environment.md)**.

---

## 7. Governance model in practice

### 7.1 Spec ledger

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

### 7.2 AC status and coverage

To see current coverage:

```bash
# Recompute AC statuses from tests
cargo xtask ac-status

# Open the generated dashboard
less docs/feature_status.md
```

Kernel ACs must be `[PASS]`. Non-kernel or template-only ACs may be `[UNKNOWN]` and are documented as such.

---

## 8. Platform APIs and auth

### 8.1 Auth modes

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

### 8.2 Key endpoints

**Core governance and discovery:**

```bash
# Governance health + metadata
curl http://localhost:8080/platform/status

# Full governance graph (stories/REQs/ACs/docs/commands)
curl http://localhost:8080/platform/graph

# Schema/OpenAPI for the platform
curl http://localhost:8080/platform/schema

# Schema for a specific type
curl http://localhost:8080/platform/schema/{name}

# Docs index (design docs, ADRs, how-tos)
curl http://localhost:8080/platform/docs/index

# AC coverage summary (BDD + test results)
curl http://localhost:8080/platform/coverage
```

**Work and task management:**

```bash
# Tasks from specs/tasks.yaml with filtering
curl http://localhost:8080/platform/tasks
curl http://localhost:8080/platform/tasks?status=Todo&req=REQ-MYSERV-001

# Suggested next work (for agents) – given a task, recommend sequence
curl "http://localhost:8080/platform/tasks/suggest-next?task=TASK-001"

# Task dependency graph (JSON or Mermaid format)
curl http://localhost:8080/platform/tasks/graph
curl "http://localhost:8080/platform/tasks/graph?format=mermaid"

# Update task status
curl -X POST http://localhost:8080/platform/tasks/{id}/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'
```

**Developer and agent workflows:**

```bash
# Available developer flows and xtask commands
curl http://localhost:8080/platform/devex/flows

# Agent hints (prioritized work suggestions for Todo/InProgress tasks)
# Returns: task ID, title, owner, labels, REQ/AC IDs, and recommended commands
curl http://localhost:8080/platform/agent/hints
```

**Metadata and issues:**

```bash
# Development friction log (DevEx issues)
curl http://localhost:8080/platform/friction

# Friction entry by ID
curl http://localhost:8080/platform/friction/{id}

# Design questions and ambiguities
curl http://localhost:8080/platform/questions

# Question by ID
curl http://localhost:8080/platform/questions/{id}

# Fork/branch information (forks of this template)
curl http://localhost:8080/platform/forks

# Fork information by name
curl http://localhost:8080/platform/forks/{name}
```

---

## 9. Developer workflows (xtask)

Everything runs through the `xtask` binary. It's designed to be friendly for humans and agents.

### 9.1 Onboarding & sanity

```bash
# Environment sanity check
cargo xtask doctor

# Quick code quality check (fmt, clippy, unit tests)
cargo xtask check

# Full governance gate (Tier-1) – includes all 10 selftest steps
cargo xtask selftest

# Local precommit gate (what CI enforces before merge)
cargo xtask precommit
```

### 9.2 Governance-specific checks

```bash
# Validate and lint Skills (name format, descriptions, allowed-tools safety, no secrets)
cargo xtask skills-lint

# Format Skills SKILL.md files (frontmatter, headings)
cargo xtask skills-fmt

# Validate and lint Agents (name format, descriptions, tools, model policy, skills references, no secrets)
cargo xtask agents-lint

# Format Agent .md files
cargo xtask agents-fmt

# Compute AC statuses from test results and write docs/feature_status.md
cargo xtask ac-status

# AC coverage summary (what % of ACs have passing tests)
cargo xtask ac-coverage
```

### 9.3 AC-first feature development

```bash
# 1. Create a new AC (requires both --story and --requirement)
cargo xtask ac-new AC-MYSERV-001 "Users can list todos" \
  --story US-MYSERV-001 \
  --requirement REQ-MYSERV-TODOS

# 2. Edit the spec ledger to add context
#    (specs/spec_ledger.yaml is updated by step 1)

# 3. Create a BDD scenario
#    Edit specs/features/todos.feature and tag with @AC-MYSERV-001

# 4. Generate an LLM context bundle (task name from .llm/contextpack.yaml)
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

### 9.4 Selective testing

```bash
# Test only what changed vs origin/main (fast loop)
cargo xtask test-changed

# Plan-only mode (see what would be run)
XTASK_TEST_CHANGED_PLAN_ONLY=1 cargo xtask test-changed

# Test a specific acceptance criterion
cargo xtask test-ac AC-PLT-001

# List available tasks
cargo xtask tasks-list

# Discover available flows and commands
cargo xtask help-flows
```

### 9.5 Releases

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

## 10. LLM / agent ergonomics

This repo is designed so that an agent can act as a **real teammate**, not a glorified autocomplete:

- **Specs & schemas** give a precise brief (`spec_ledger.yaml`, `config_schema.yaml`, `devex_flows.yaml`, `tasks.yaml`).
- **Bundles** give a bounded context:
  - `cargo xtask bundle <TASK>` produces curated context packs (e.g., `implement_ac` is a registered task).
- **Flows & Skills** provide the patterns:
  - `.claude/skills/*/SKILL.md` describe governed-feature-dev, governed-maintenance, governed-release, and governance-debug flows.
  - **Skills are governed**: `skills-lint` validates name format, descriptions, allowed-tools safety, and prevents hardcoded secrets.
- **Agents** are first-class governed artifacts:
  - `.claude/agents/*.md` define long-lived, specialized agents with system prompts and tool bindings.
  - **Agents are governed**: `agents-lint` validates name format, descriptions, tools, model policy, skills references, and prevents hardcoded secrets.
  - See `docs/AGENTS_GOVERNANCE.md` and `docs/AGENTS_TEMPLATE.md` for governance rules and creation checklist.
- **Platform APIs** provide live telemetry:
  - `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints`, `/platform/docs/index`, `/platform/schema`, `/platform/devex/flows`.

The expected agent loop looks like:

1. **Orient**

   - Run `cargo xtask doctor`, `cargo xtask ac-status`, `cargo xtask help-flows`.
   - Call `/platform/status` and `/platform/graph` to understand the current state.

2. **Pick work**

   - Use `cargo xtask tasks-list` and `/platform/agent/hints` to identify a task.
   - Read the relevant REQ/AC entries in `spec_ledger.yaml`.

3. **Plan with a bundle**

   - Generate a bundle with `cargo xtask bundle implement_ac` or another task name from `.llm/contextpack.yaml`.
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

- [CLAUDE.md](./CLAUDE.md) – agent operational prompt
- [AGENT_GUIDE.md](./docs/AGENT_GUIDE.md) – deeper guidance for agent-driven work
- [SELECTIVE_TESTING.md](./docs/SELECTIVE_TESTING.md) – validation ladder and change-aware testing
- [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) – common problems and solutions

---

## 11. Adoption patterns

### 11.1 Greenfield: new service

- Clone this template into a new repo.
- Follow the [Pre-Fork Checklist](./docs/how-to/pre-fork-checklist.md) to validate your environment.
- Use the [QUICKSTART.md](./docs/QUICKSTART.md) guide to get oriented.
- Follow the [New Service Guide](./docs/how-to/new-service-from-template.md) for step-by-step setup.
- Adjust `service_metadata.yaml`, ledger entries, and tasks to your domain.
- Keep the platform kernel as-is; build your business logic in new crates or modules.
- Use `selftest` as the gate from day one.

### 11.2 Brownfield: existing repo

See [docs/how-to/add-governance-to-existing-repo.md](./docs/how-to/add-governance-to-existing-repo.md).

High-level flow:

- Add a `governance/` subtree (specs, policy, docs) to your existing repo.
- Add `spec-runtime` + `xtask` crates to your workspace.
- Configure your CI to run `cargo xtask selftest` as a gate.
- Gradually map your existing tests/docs into the spec ledger.

---

## 12. Relationship to portals/IDPs

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

## 13. Documentation Guide

Documentation is organized by audience and purpose:

### 13.1 Getting Started (New Users)

Start here if you're using this template for the first time:

- **[First Fork Runbook](docs/how-to/FIRST_FORK.md)** – One-page quick start for forking (do this first!)
- **[Quick Start Guide](docs/QUICKSTART.md)** – Get productive in 15 minutes
- **[Pre-Fork Checklist](docs/how-to/pre-fork-checklist.md)** – Validate your environment before forking
- [New service guide](docs/how-to/new-service-from-template.md) – Step-by-step setup after forking
- [Kernel snapshot](docs/KERNEL_SNAPSHOT.md) – What you're inheriting from the template
- **[Troubleshooting Guide](docs/TROUBLESHOOTING.md)** – Solutions for common problems

### 13.2 Daily Development (Contributors)

Use these guides while working in the codebase:

- [CLAUDE.md](./CLAUDE.md) – Agent operational instructions
- [AGENT_GUIDE.md](docs/AGENT_GUIDE.md) – Deeper guidance for agent-driven work
- [Selective Testing Guide](docs/SELECTIVE_TESTING.md) – Validation ladder and change-aware testing
- [Change Acceptance Criterion](docs/how-to/change-acceptance-criterion.md) – How to modify ACs (day-2 contracts)
- [Add Acceptance Criterion](docs/how-to/add-acceptance-criterion.md) – How to add new ACs
- [Add HTTP endpoint](docs/how-to/add-http-endpoint.md) – How to add new endpoints
- [Change OpenAPI safely](docs/how-to/change-openapi-safely.md) – How to evolve APIs
- [LLM bundles guide](docs/how-to/use-llm-bundles.md) – How to use context bundles
- [Windows Development Guide](docs/how-to/windows-development.md) – Platform-specific guidance for Windows

**Governance & LLM surfaces:**

- [SKILLS_GOVERNANCE.md](docs/SKILLS_GOVERNANCE.md) – Skills governance rules, lifecycle, and validation
- [SKILLS_TEMPLATE.md](docs/SKILLS_TEMPLATE.md) – Copy-paste template for creating new Skills
- [AGENTS_GOVERNANCE.md](docs/AGENTS_GOVERNANCE.md) – Agents governance rules, lifecycle, and validation
- [AGENTS_TEMPLATE.md](docs/AGENTS_TEMPLATE.md) – Copy-paste template for creating new Agents
- [AGENTS_VALIDATION.md](docs/AGENTS_VALIDATION.md) – Agent validation rules reference

### 13.3 Platform Setup (Maintainers)

Use these guides to configure the production environment:

- **[Branch Protection Setup](docs/BRANCH-PROTECTION-SETUP.md)** – Configure branch protection rules
- **[Tag Signing Setup](docs/how-to/setup-tag-signing.md)** – Configure GPG/SSH tag signing
- [Branch Protection Profiles](docs/reference/branch-protection-profiles.md) – Different protection levels explained
- [Required Checks Reference](docs/reference/required-checks.md) – CI checks that must pass
- [Platform Support](docs/reference/platform-support.md) – Tier-1 vs Tier-2 platforms
- [Supply Chain Hardening](docs/explanation/supply-chain-hardening.md) – Security best practices
- [Integrate with IDP or Agent](docs/how-to/integrate-idp-or-agent.md) – Use `/platform/*` APIs with Backstage, Port.io, or LLM agents

### 13.4 Understanding the System (Architecture)

Read these to understand how the template works:

- [Why This Exists](docs/why-this-exists.md) – Motivation and philosophy
- [Architecture Overview](docs/explanation/architecture.md) – System design
- [Template Architecture](docs/explanation/template-architecture.md) – Template structure
- [Rust-as-Spec Overview](docs/explanation/rust-as-spec-overview.md) – Core concept
- [Controls as Code](docs/explanation/controls-as-code.md) – Governance approach
- [IDP Positioning](docs/explanation/idp-positioning.md) – How this fits with platform tools
- [Template Contracts](docs/explanation/TEMPLATE-CONTRACTS.md) – What the template guarantees

**Design docs:**

- [AC Governance Design](docs/design/ac-governance.md) – How ACs are governed and enforced
- [Skills Governance Design](docs/design/skills-governance.md) – How Skills are governed
- [Agents Governance Design](docs/design/agents-governance.md) – How Agents are governed

**ADRs (Architecture Decision Records):**

- [ADR-0020: Claude Code Skills Governance](docs/adr/0020-claude-code-skills-governance.md)
- [ADR-0021: Claude Code Agents Governance](docs/adr/0021-claude-code-agents-governance.md)
- [ADR-0022: Platform Metadata and Test Isolation](docs/adr/0022-platform-metadata-and-test-isolation.md)

### 13.5 Process & Planning

Roadmaps, ADRs, and planning documents:

- [ROADMAP.md](docs/ROADMAP.md) – Current state and future plans
- [BACKLOG.md](docs/BACKLOG.md) – Known gaps and future work
- [Missing Manual](docs/MISSING_MANUAL.md) – Unwritten docs and known documentation gaps
- [ADR Index](docs/adr/README.md) – Architecture decision records
- [Release Playbook](docs/RELEASE_PLAYBOOK.md) – How to cut releases

### 13.6 Reference Material

Technical references and command documentation:

- [Documentation Sources](docs/reference/doc-sources.md) – What to trust when docs disagree
- [xtask Commands](docs/reference/xtask-commands.md) – Complete CLI reference
- [CI Coverage](docs/reference/ci-coverage.md) – What CI tests
- [Feature Status](docs/feature_status.md) – Auto-generated AC health dashboard
- [Testing Strategy](docs/testing-strategy.md) – How testing works
- [CONSTITUTION.md](docs/CONSTITUTION.md) – Core principles and boundaries

---

## 14. Current Status & Roadmap

**Kernel Version:** v3.3.3 (see [KERNEL_SNAPSHOT.md](docs/KERNEL_SNAPSHOT.md) for what you inherit)

**AC Health:** See:

- [Feature Status](docs/feature_status.md) – Acceptance criteria health (auto-generated)
- [Feature Status Notes](docs/feature_status_notes.md) – How to read the AC table (Kernel/Template/Meta)

**Strategy:** See [ROADMAP.md](docs/ROADMAP.md) for current direction and [BACKLOG.md](docs/BACKLOG.md) for known gaps.

---

## 15. License

This template is dual-licensed:

- MIT – see `LICENSE-MIT`
- Apache 2.0 – see `LICENSE-APACHE`

You may use it under either license.
