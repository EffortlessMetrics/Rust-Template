# Rust-as-Spec Platform Cell (v3.2.0) 🎉

[![CI](https://github.com/EffortlessMetrics/Rust-Template/actions/workflows/ci-template-selftest.yml/badge.svg)](https://github.com/EffortlessMetrics/Rust-Template/actions/workflows/ci-template-selftest.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

**A self-healing platform cell where governance enforces itself and agents can move fast safely.**

> **What is this?**
>
> A **governed Rust service** that maintains a living contract between specs, code, docs, and policies—enforced automatically by CI and introspectable via HTTP APIs and a web UI.

> **Status: Kernel Frozen** (v2.5.0)
>
> The platform successfully used its own governance contracts to build itself. All 7 selftest steps pass. Next phase: real-world service pilot.

Unlike conventional templates that drift over time, this repository:
- **Self-heals**: Specs, docs, and policies are validated as rigorously as code
- **Self-explains**: Platform APIs (`/platform/*`) and Web UI (`/ui`) expose governance state in real-time
- **Self-guides**: Context-aware `suggest-next` tells humans and agents exactly what to do

**Use this when:**
- ✅ You need **governed starting point** for Rust services (health/version/metrics, policies, selftest)
- ✅ You're building in a **regulated/multi-team environment** (FinTech, HealthTech, Platform Engineering)
- ✅ You're using **LLMs/agents as contributors** and need trust-but-verify guardrails
- ✅ You want **specs → code → docs** linkage enforced by CI, not manual reviews

**Don't use this if:**
- ❌ You just want "hello world in Axum" (too heavy)
- ❌ You're prototyping without governance requirements (overkill)
- ❌ You don't need AC traceability between specs, ACs, and code (simpler templates exist)

**Read more:** [Why This Template Exists](docs/why-this-exists.md) | [ROADMAP.md](docs/ROADMAP.md) | [Technical Overview](docs/explanation/rust-as-spec-overview.md)

---

## What Makes This Different

### Conventional Template
```
README.md says:          specs are in docs/specs/
Reality 6 months later:  specs were moved to Notion
                         README outdated
                         No one knows what's governed
```

### Rust-as-Spec Platform Cell
```
specs/spec_ledger.yaml:  Stories → Requirements → ACs
    ↓ (loaded by)
spec-runtime (Rust):     Type-checked structs
    ↓ (enforced by)
cargo xtask selftest:    7-step validation (CI gate)
    ↓ (exposed by)
/platform/status (API):  Real-time governance metrics
/ui (Web UI):            Visual governance console
```

**Result:** If specs drift from code, CI fails. If code doesn't match specs, CI fails. There is no third state.

**Read more:**
- [ROADMAP.md](docs/ROADMAP.md) - Strategic direction and pilot plan
- [Technical Overview](docs/explanation/rust-as-spec-overview.md) - Deep dive into the architecture
- [Why This Template Exists](docs/why-this-exists.md) - Problem statement and philosophy

---

## Quick Start

### Prerequisites

This template is **Nix-first**. Development environments should be declarative, reproducible, and match CI exactly.

**Platform Recommendation:** For the best experience, use a **Tier 1** platform (Linux, macOS, or WSL2 on Windows) with Nix devshell for exact CI parity and strict governance validation. See [Platform Support at a Glance](#platform-support-at-a-glance) below for tier details.

```bash
# Install Nix (one-time setup)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# Enter development shell
nix develop

# Verify environment
cargo xtask doctor
```

**Without Nix?** See [manual setup](docs/how-to/setup-without-nix.md) (requires Rust 1.91+, conftest, cargo-binstall). Native Windows users should review [Platform Support Reference](docs/reference/platform-support.md) for known limitations.

### 30-Second Tour

```bash
# 1. One-command bootstrap
cargo run -p xtask -- dev-up
# → Installs pre-commit hooks
# → Checks Docker
# → Runs governance check (low-resource mode)

# 2. Start the service
cargo run -p app-http
# → Open http://localhost:3000/ui (governance dashboard)
# → Open http://localhost:3000/platform/status (JSON API)

# 3. Get guidance on next steps
cargo xtask suggest-next --task implement_ac
# → Shows what to do and what's already done (context-aware)

# 4. Implement a feature (AC-first workflow)
cargo xtask ac-new AC-EXAMPLE-001 "Health endpoint returns 200" --requirement REQ-TPL-HEALTH
cargo xtask bundle implement_ac
cargo xtask selftest
```

---

## Platform Support at a Glance

This template supports multiple platforms with different validation guarantees:

| Platform | Tier | Environment | Supports selftest | Best For |
|----------|------|-------------|------------------|----------|
| Linux | 1 | Nix devshell | ✅ Strict gates | Server development, CI/CD |
| macOS | 1 | Nix devshell | ✅ Strict gates | Cross-platform development |
| WSL2 | 1 | Nix on WSL2 | ✅ Strict gates | Windows users needing Unix |
| Windows | 2 | Native PowerShell/Bash | ⚠️ With caveats | Native Windows workflows |

**Tier Definitions:**
- **Tier 1** (Fully Validated): Exact CI parity, all governance gates strictly enforced, canonical environments
- **Tier 2** (Supported): Core functionality validated, known platform limitations (e.g., file locking on Windows)

**📖 Full details:** See [Platform Support Reference](docs/reference/platform-support.md) for detailed setup instructions, known issues, and troubleshooting.

---

## Platform Support Details

All **Tier 1** platforms (Linux, macOS, WSL2 with Nix devshell) run `cargo xtask selftest` with strict, hard gates on all kernel ACs and provide exact CI parity.

**Tier 2** (Native Windows 10/11 with PowerShell/Git Bash) is fully supported with known caveats: selftest passes in normal conditions but may intermittently encounter file locking errors (`os error 5`) if antivirus or other processes lock binaries during rebuild.

**Quick fixes for Windows file locking:**
1. Close any running xtask or cargo processes
2. Exclude `target/` from real-time antivirus scanning
3. Re-run `cargo xtask selftest`, or use WSL2 for canonical validation

**For complete platform details, setup instructions, and troubleshooting:** See [Platform Support Reference](docs/reference/platform-support.md)

---

## Platform Introspection

The same specs that power CI are exposed at runtime:

### HTTP APIs (`/platform/*`)

```bash
# Governance health
curl http://localhost:8080/platform/status

# Full governance graph (JSON)
curl http://localhost:8080/platform/graph

# Available tasks
curl http://localhost:8080/platform/tasks

# Context-aware guidance
curl "http://localhost:8080/platform/tasks/suggest-next?task=implement_ac"

# Developer workflows
curl http://localhost:8080/platform/devex/flows

# Documentation inventory
curl http://localhost:8080/platform/docs/index
```

### Web UI (`/ui`)

```bash
# Dashboard: Platform health + governance contracts
open http://localhost:8080/ui

# Graph: Interactive Mermaid visualization
open http://localhost:8080/ui/graph

# Flows: DevEx workflows + task guidance
open http://localhost:8080/ui/flows
```

**Key Property:** UI has no database. Every page load calls the same `load_spec_ledger()` / `load_devex_flows()` functions that `selftest` uses. **If the UI shows it, CI enforces it.**

---

## The 7-Step Selftest Contract

`cargo xtask selftest` is the **single mandatory gate** for all changes:

```
[1/7] Running core checks (fmt, clippy, tests)...
      ✓ Code quality baseline

[2/7] Running BDD acceptance tests...
      ✓ Behavior matches AC text

[3/7] Running AC status mapping & ADR references...
      ✓ Traceability (every AC has tests, every ADR exists)

[4/7] Testing LLM context bundler...
      ✓ Agent safety (bounded context generation)

[5/7] Running policy tests...
      ✓ Compliance (OPA/Rego policies pass)

[6/7] Checking DevEx contract...
      ✓ Usability (required commands exist and are reachable)

[7/7] Checking governance graph invariants...
      ✓ Structural integrity (no orphans, missing ACs, unreachable commands)
```

**What this prevents:**
- Requirements with `must_have_ac: true` but no ACs → `REQ_HAS_NO_AC` violation
- Required commands not part of any flow → `COMMAND_UNREACHABLE` violation
- Docs referenced but missing from disk → `DOC_ORPHANED` violation
- ACs without BDD scenario tags → Policy violation

---

## Governance Health & Coverage

The template includes commands for real-time traceability from requirements → ACs → tests → code:

### Check Governance Status

```bash
# Overall governance health dashboard
cargo xtask status

# Detailed AC coverage report (shows which ACs have tests)
cargo xtask ac-coverage

# Export governance graph for visualization
cargo xtask graph-export
```

### Validation Loop

```bash
# Fast dev loop (fmt, clippy, tests, basic checks)
cargo xtask check

# Full governance validation (7-step contract)
cargo xtask selftest
```

### Extend Governance

```bash
# Generate BDD scenario stubs for an AC
cargo xtask ac-suggest-scenarios AC-PLT-XXX

# Create new AC
cargo xtask ac-new AC-MYPROJ-001 "Description" --requirement REQ-MYPROJ-FEATURE

# Create new Architecture Decision Record
cargo xtask adr-new "My Architecture Decision"
```

**Key Property:** Every kernel AC has passing BDD or unit coverage. Run `cargo xtask ac-coverage` anytime to see real-time traceability.

---

## How to Use This Project

| Your  Situation | Recommended Path | Documentation |
|---------------|------------------|-----------------|
| **Starting a new service** | Clone this template | [Getting Started](#getting-started) |
| **Adding governance to existing repo** | Use the library | [Brownfield Guide](docs/how-to/add-governance-to-existing-repo.md) |
| **Multiple services, want updates** | Template as upstream | [Adoption Patterns](docs/explanation/adoption-patterns.md#pattern-b-template-as-upstream) |
| **Platform team (10+ services)** | Generator-based | [Adoption Patterns](docs/explanation/adoption-patterns.md#pattern-c-generator-based-platform-team) |

---

## Getting Started

### 1. Clone and Initialize

```bash
# Clone the template
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service

# Initialize your service
./scripts/init-service.sh my-service "My Service Description"

# Enter Nix shell
nix develop

# Verify everything works
cargo xtask selftest
```

### 2. Understand the Structure

```
specs/
  spec_ledger.yaml   # Stories → Requirements → ACs
  devex_flows.yaml   # Developer workflows (flows + commands)
  doc_index.yaml     # Documentation inventory
  tasks.yaml         # Work units with recommended sequences
  features/          # BDD scenarios (Cucumber/Gherkin)

docs/
  adr/               # Architecture Decision Records
  design/            # Design docs (linked to requirements)
  how-to/            # Runbooks and guides
  tutorials/         # Step-by-step tutorials

policy/
  *.rego             # OPA/Rego compliance policies
  testdata/          # Policy test fixtures

crates/
  app-http/          # HTTP service (Axum)
  business-core/     # Business logic
  spec-runtime/      # Spec loaders + graph logic
  xtask/             # CLI orchestration
  ...
```

### 3. Implement Your First Feature (AC-First Workflow)

```bash
# 1. Create a new AC
cargo xtask ac-new AC-MYSERV-001 "Users can list todos" --requirement REQ-MYSERV-TODOS

# 2. Edit the spec ledger to add context
#    (specs/spec_ledger.yaml is updated by step 1)

# 3. Create a BDD scenario
#    Edit specs/features/todos.feature

# 4. Generate LLM context bundle
cargo xtask bundle implement_ac

# 5. Implement the feature
#    (Use the bundle with your LLM or write code manually)

# 6. Run tests
cargo xtask bdd

# 7. Verify governance
cargo xtask selftest
```

**For LLM-assisted development:** See [CLAUDE.md](CLAUDE.md) for standard prompts and workflows.

---

## Key Workflows

### Developer Workflows

```bash
# Onboarding (first time)
cargo xtask doctor
cargo xtask check
cargo xtask selftest

# AC-first feature development
cargo xtask ac-new AC-ID "Description" --requirement REQ-ID
cargo xtask bundle implement_ac
cargo xtask bdd
cargo xtask selftest

# Design decision
cargo xtask adr-new "Use PostgreSQL for persistence"
cargo xtask design-new REQ-ID "Database Schema Design"

# Dependency management
cargo xtask audit
cargo xtask outdated
cargo xtask upgrade-deps

# Release preparation
cargo xtask changelog
cargo xtask sbom-local
cargo xtask release-prep
```

### Platform Workflows

```bash
# Governance checks
cargo xtask graph-export --format mermaid
cargo xtask ac-status
cargo xtask policy-test
cargo xtask docs-check

# Maintenance
cargo xtask clean
cargo xtask fmt-all
cargo xtask hakari

# Help
cargo xtask help
cargo xtask help-flows
```

---

## Architecture

### The Four-Phase Pipeline

Every aspect of governance flows through:

```
Spec (YAML) → Loader (Rust) → Enforce (CI) → Introspect (API/UI)
```

1. **Spec**: Structured YAML files in `specs/`
2. **Loader**: Type-safe deserialization via `spec-runtime` (serde)
3. **Enforce**: 7-step `selftest` validates specs ↔ code ↔ docs ↔ policies
4. **Introspect**: `/platform/*` APIs and `/ui` expose governance state

**Key Property:** If specs are invalid, they fail to load. If they load, they're valid. No ambiguity.

### Hexagonal Architecture

```
┌─────────────────────────────────────┐
│  Adapters (HTTP, gRPC)              │
│  ├─ app-http (Axum)                 │
│  └─ adapters-grpc (Tonic)           │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│  Business Core                      │
│  ├─ business-core (domain models)   │
│  ├─ governance (policies + specs)   │
│  └─ telemetry (metrics + tracing)   │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│  Infrastructure                     │
│  ├─ spec-runtime (loaders + graph)  │
│  ├─ model (shared types)            │
│  └─ proto (gRPC definitions)        │
└─────────────────────────────────────┘
```

---

## Current Status: v2.5.0 – "The Governing Kernel" (Frozen) 🎉

**Phase 4 Complete** (All 4 Epics):
- ✅ **Epic 1**: Graph invariants as mandatory CI gate
- ✅ **Epic 2**: Context-aware `suggest-next` (tracks satisfied vs pending steps)
- ✅ **Epic 3**: Real-time policy status via `/platform/status`
- ✅ **Epic 4**: Rust-native Web UI (maud + htmx, zero build step)

**Pilot Complete** (Agent-Ready):
- ✅ Local runtime (`docker-compose.yaml` with Postgres + Jaeger)
- ✅ Governance hooks (`cargo xtask install-hooks`)
- ✅ Agent skills (`.claude/skills/*`)

**Validated:**
- All 7 selftest steps pass
- 22/22 policy tests pass
- Platform used itself to build final features

**Status: Kernel Frozen**  
No new platform features until validated by real-world service pilot.

**Next:** Real-world service pilot → friction log → v2.5.x hardening

**See:** [ROADMAP.md](docs/ROADMAP.md) for full details on current state, pilot plan, and future direction.

---

## Documentation

### Getting Started
- [Quick Start (this README)](#quick-start)
- [First AC Change (Tutorial)](docs/tutorials/first-ac-change.md)
- [Setup Without Nix](docs/how-to/setup-without-nix.md)

### Core Concepts
- [Why This Template Exists](docs/why-this-exists.md)
- [Rust-as-Spec Technical Overview](docs/explanation/rust-as-spec-overview.md)
- [ROADMAP.md](docs/ROADMAP.md)

### How-To Guides
- [Add Governance to Existing Repo](docs/how-to/add-governance-to-existing-repo.md)
- [Create a New Service from Template](docs/how-to/new-service-from-template.md)
- [Update Templates from Upstream](docs/how-to/template-updates.md)

### Reference
- [Platform Support Reference](docs/reference/platform-support.md)
- [ADR Index](docs/adr/README.md)
- [Policy Reference](policy/README.md)
- [xtask Command Reference](docs/reference/xtask-commands.md)

### For LLMs/Agents
- [CLAUDE.md](CLAUDE.md) - System prompt and standard workflows
- [Agent Skills Guide](docs/AGENT_SKILLS.md) - How to author Skills for this repo
- [Agent Guide](docs/AGENT_GUIDE.md) - Operational procedures for agents
- [Platform API Reference](docs/reference/platform-api.md)

---

## Contributing

This template is in **Pilot Phase**. We're validating the governance model with 2-3 real services before wider adoption.

**How to contribute:**
1. **Use it:** Clone and build a service
2. **Log friction:** Maintain a `FRICTION_LOG.md` as you work
3. **Share feedback:** Open issues for blockers or nice-to-haves
4. **Propose changes:** PRs welcome, but discuss large changes first

**See:** [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.

---

## License

Dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

Choose whichever works best for your use case.

---

## Acknowledgments

Built with:
- Rust + Cargo workspaces
- Axum (HTTP), Tonic (gRPC)
- Cucumber-rs (BDD)
- OPA/Conftest (policies)
- Maud (type-safe HTML)
- HTMX + Mermaid.js (UI)
- Nix (dev environment)

Inspired by:
- [Zero to Production in Rust](https://www.zero2prod.com/) (Luca Palmieri)
- [Backstage](https://backstage.io/) (Spotify's service catalog)
- [SpecKit](https://speckit.io/) (API contract testing)
- Platform Engineering community

---

**Ready to start?**

```bash
nix develop
cargo xtask doctor
cargo run -p app-http
# Open http://localhost:8080/ui
```

**Questions?** See [docs/](docs/) or open an issue.
