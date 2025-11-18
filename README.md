# Rust Spec-as-Code Template (v2.3.0)

**Heavy governance so LLMs can move fast**

This project serves **two purposes**:

1. **Full-Featured Template**: Clone it to start a new Rust service with governance, observability, and AI-assisted development built-in
2. **Governance Library**: Add the `rust_iac_xtask_core` library to your existing Rust project to gain AC tracking, policy enforcement, and LLM bundling

Whether you're starting fresh or adding governance to an existing codebase, this project provides the infrastructure for safe, fast AI-assisted development.

**Key Features:**
- 🦀 Rust-native development with xtask orchestration
- 🎯 AC-first workflow: specs → tests → implementation
- 🔒 Policy-as-code governance (OPA/Rego)
- 🤖 LLM context bundles for AI-assisted development
- 🏗️ Hexagonal architecture with Axum + tracing
- 🧪 BDD acceptance tests with real HTTP integration

---

## How to Use This Project

| Your Situation | Recommended Path | Documentation |
|---------------|------------------|---------------|
| **Starting a new service** | Clone this template | [Quick Start (Template)](#quick-start-template) |
| **Adding governance to existing repo** | Use the library | [Quick Start (Library)](#quick-start-library) / [Brownfield Guide](docs/how-to/add-governance-to-existing-repo.md) |
| **Multiple services, want updates** | Template as upstream | [Adoption Patterns](docs/explanation/adoption-patterns.md#pattern-b-template-as-upstream) |
| **Platform team (10+ services)** | Generator-based | [Adoption Patterns](docs/explanation/adoption-patterns.md#pattern-c-generator-based-platform-team) |


### Who This Is For

This project is opinionated and governance-heavy **on purpose**. It serves two primary audiences:

**Teams Starting New Services:**
- Want governance, observability, and AI-safety built-in from day 0
- Need AC-first development workflow with traceability
- Value hexagonal architecture and clean separation of concerns
- Using LLMs/agents as real contributors, need trust-but-verify guardrails

**Teams Retrofitting Governance:**
- Have existing Rust projects that need AC tracking and policy enforcement
- Want to add LLM-safe development workflows without rewriting everything
- Need incremental adoption: start with xtask commands, add policies over time
- Care about invariants, privacy, and production safety

**Not for:**
- Hand-writing everything without AI assistance → Use a lighter Axum starter
- Prototyping without governance → This is overkill
- Teams that don't need traceability between specs, ACs, and code

**Perfect for:**
- Letting LLMs move fast inside a Rust service without guessing what broke
- Maintaining audit trails from user stories to implementation
- Enforcing policies without manual reviews


### Current Status

- Runtime & AC workflow: ✅ Fully working
- LLM bundler: ✅ Producing bounded, governed context
- Policy suite: ✅ Enforcing ledger, features, K8s, privacy rules
- Selftest harness: ✅ Complete validation suite (local/CI properly differentiated)

This template is at v2.3.0 and production-ready. Currently in **Pilot Phase** for validation.

**Latest update (2025-11-18)**: Fixed selftest to gracefully skip policy tests when conftest is unavailable locally while still enforcing them in CI.

## Quick Start

Choose your path based on your situation:

### Quick Start (Pilot Project)

**Want to validate the template?** - Create a greenfield pilot to test real usage:

```bash
# From the template repository
./scripts/create-pilot.sh my-pilot-service ~/projects/

# Then follow Day 1 workflow
cd ~/projects/my-pilot-service
cargo run -p xtask -- selftest           # Verify setup
# Add your first AC to specs/spec_ledger.yaml
cargo run -p xtask -- bundle implement_ac # Get LLM context
# Track friction in FRICTION_LOG.md
```

This creates a fresh project from v2.3.0, sets up friction logging, and guides you through real feature development. Continue to [Pilot Workflow](#pilot-workflow).

### Quick Start (Template)

**For new services** - Clone and validate everything:

```bash
git clone <your-repo-url> && cd Rust-Template
nix develop
cargo run -p xtask -- quickstart
```

This runs all checks, tests, and bundler to prove the template works. Continue to [New Service from Template](docs/how-to/new-service-from-template.md).

### Quick Start (Library)

**For existing Rust projects** - Add governance incrementally:

```bash
# Add xtask tooling to your Cargo workspace
cargo add --path /path/to/rust-template/crates/xtask -p xtask

# Or use from crates.io (when published)
# cargo install rust_iac_xtask_core

# Create minimal specs/ directory structure
mkdir -p specs/{features,userstories}
touch specs/spec_ledger.yaml

# Run your first governance check
cargo run -p xtask -- ac-status
```

Continue to [Brownfield Guide](docs/how-to/add-governance-to-existing-repo.md) for incremental adoption.

### Pilot Workflow

**For validating the template in real usage** - The pilot workflow helps you identify template friction:

```bash
# 1. Create pilot project
./scripts/create-pilot.sh task-api ~/projects/

# 2. Day 0 - Verify setup
cd ~/projects/task-api
cargo run -p xtask -- selftest

# 3. Day 1+ - Implement features
# a. Add AC to specs/spec_ledger.yaml
# b. Add BDD scenario to specs/features/
# c. Generate context
cargo run -p xtask -- bundle implement_ac
# d. Feed .llm/bundle/implement_ac.md to your LLM
# e. Apply changes
# f. Validate
cargo run -p xtask -- selftest

# 4. Record friction
# Every rough edge → FRICTION_LOG.md
# Missing docs, confusing behavior, unclear errors

# 5. After 1-2 weeks
# Review FRICTION_LOG.md
# Classify: 🔴 Blockers / 🟡 Annoyances / 🟢 Nice-to-have
# Decide: Does template need patches or is it "good enough"?
```

**Goal:** Understand template usability through real development, not speculation.

See [Release Playbook](docs/RELEASE_PLAYBOOK.md) for how friction logs inform template evolution.

### Developer Workflow

All development operations use `cargo run -p xtask --` as the primary interface:

| Situation | Command | What it does |
|-----------|---------|--------------|
| **New dev/machine** | `nix develop` → `cargo run -p xtask -- check` | Enter environment, validate setup |
| **Everyday dev** | `cargo run -p xtask -- check` | Format, lint, tests (before every commit) |
| **Before push/in CI** | `cargo run -p xtask -- selftest` | Comprehensive validation suite |
| **Check AC coverage** | `cargo run -p xtask -- ac-status` | Generate AC status report |
| **Check policies** | `cargo run -p xtask -- policy-test` | Test Rego policies with conftest |
| **Build LLM bundle** | `cargo run -p xtask -- bundle <task>` | Generate focused context for AI coding |
| **First-time validation** | `cargo run -p xtask -- quickstart` | Quick health check |
| **Run acceptance tests** | `cargo run -p xtask -- bdd` | Execute BDD scenarios |

### Command Reference

Core `xtask` commands (see [full reference](docs/reference/xtask-commands.md)):

| Command | Purpose | When to use |
|---------|---------|-------------|
| `check` | Format, lint, tests | Every commit |
| `bdd` | BDD acceptance tests | After AC work |
| `ac-status` | AC coverage report | Check test coverage |
| `policy-test` | Rego policy tests | Validate governance |
| `bundle <task>` | LLM context bundle | Before AI coding |
| `quickstart` | Quick validation | First run |
| `selftest` | Full validation suite | CI, releases |

**Examples:**
```bash
cargo run -p xtask -- check                    # Before every commit
cargo run -p xtask -- bdd                      # Test acceptance criteria
cargo run -p xtask -- ac-status                # Check AC coverage
cargo run -p xtask -- policy-test              # Validate policies
cargo run -p xtask -- bundle implement_ac      # Get context for AC work
cargo run -p xtask -- selftest                 # Full validation (CI)
```

### What's Working

- ✅ **xtask** - Single Rust-native CLI for all operations
- ✅ **Runtime architecture** - Axum HTTP service with hexagonal layering
- ✅ **Observability** - tracing/logging from day 0 (RUST_LOG env support)
- ✅ **BDD acceptance tests** - cucumber-rs with JUnit output
- ✅ **AC status mapping** - tests → features → ledger traceability
- ✅ **Policy-as-code** - OPA/Rego for ledger, features, flags, privacy
- ✅ **LLM bundler** - Curated context for AI-assisted development
- ✅ **CI workflows** - 22 GitHub Actions for comprehensive validation

### Architecture

```text
crates/
├── app-http/         → HTTP adapter (Axum, routes, DTOs)
├── business-core/    → Domain logic (business rules, no HTTP)
├── model/            → Domain entities and value objects
├── adapters-db-sqlx/ → Postgres adapter (sqlx)
├── adapters-grpc/    → gRPC adapter (tonic)
├── telemetry/        → Observability (tracing & OTLP export)
├── acceptance/       → BDD tests (cucumber-rs)
└── xtask/            → Dev/CI tooling
```

**Key pattern:** Dependencies point inward
`app-http` → `core` ✓  (adapters call domain)
`core` → `app-http` ✗  (domain never calls adapters)

---

## As a Library

**Don't need the full template?** Reuse parts of this template in your existing Rust projects.

### Available Crates

| Crate | Purpose | When to Use |
|-------|---------|-------------|
| **`rust_iac_xtask_core`** | Governance & xtask orchestration | Add specs/policy/LLM governance to any project |
| **`rust_iac_config`** | Generic infra config validation | Validate K8s manifests, environments, resource constraints |

### rust_iac_xtask_core (Governance & Xtask Core)

The **governance spine** for Rust IaC projects:

| Component | What It Provides |
|-----------|-----------------|
| **xtask Commands** | `init`, `selftest`, `ac-status`, `policy-test`, `bundle` |
| **Spec Framework** | `specs/spec_ledger.yaml` + `specs/features/` directory |
| **Policy Layer** | `policy/` directory scaffolding for OPA/Rego |
| **LLM Context** | `.llm/contextpack.yaml` for task-scoped AI assistance |
| **Config** | `RUST_IAC.toml` for project metadata |

### rust_iac_config (Infrastructure Config)

Lightweight YAML-based config library for infra validation:

| Feature | What It Does |
|---------|-------------|
| **Environment Management** | Define dev/staging/prod with manifest paths |
| **Validation Rules** | Enforce required dirs, files, git repo presence |
| **Resource Constraints** | Check K8s resources, probes, labels |
| **Zero Assumptions** | No hardcoded AC IDs or template-specific logic |

### When to Use Library vs Template

| Use Case | Recommendation |
|----------|----------------|
| **New service, greenfield** | Clone the template |
| **Existing service, add governance** | Use the library |
| **Need hexagonal architecture** | Clone the template |
| **Just want xtask + policies** | Use the library |
| **Full observability stack** | Clone the template |
| **Incremental adoption** | Use the library |

### Getting Started as a Library

1. **Add xtask to your workspace:**
   ```bash
   # Add as workspace member in Cargo.toml
   [workspace]
   members = ["crates/*", "path/to/rust-template/crates/xtask"]
   ```

2. **Create minimal governance structure:**
   ```bash
   mkdir -p specs/{features,userstories}
   mkdir -p policy/
   touch specs/spec_ledger.yaml
   ```

3. **Run governance checks:**
   ```bash
   cargo run -p xtask -- ac-status    # Check AC coverage
   cargo run -p xtask -- bundle <task> # Generate LLM context
   ```

4. **Adopt incrementally:**
   - Start with AC tracking only
   - Add policies when you need enforcement
   - Integrate LLM bundling when ready
   - Adopt BDD testing at your own pace

### CI Integration Examples

Use xtask commands in your existing CI:

```yaml
# .github/workflows/governance.yml
- name: Check AC Coverage
  run: cargo run -p xtask -- ac-status

- name: Test Policies
  run: cargo run -p xtask -- policy-test

- name: Validate Spec Ledger
  run: cargo run -p xtask -- check
```

See [CI Examples](.github/workflows/) for 20+ workflow templates you can adapt.

**Learn more:** [Brownfield Guide](docs/how-to/add-governance-to-existing-repo.md)

---

## Documentation

**📚 Complete documentation in `docs/`** - [Documentation Index](docs/README.md)

### Quick Start (New to Template)

**Recommended learning path:**

1. 📘 **[Day 1: First Change](docs/tutorials/day-1-first-change.md)** (30 min) **← START HERE**
   - Clone, validate, run quickstart
   - Add trivial AC and see it go green
   - Learn AC-first development loop

2. 📘 **[Day 7: First Real Feature](docs/tutorials/day-7-first-real-feature.md)** (90 min)
   - Build complete task management feature
   - Multi-layer architecture (model → core → app-http)
   - Production-ready patterns with validation and errors

3. 📕 **[Adoption Patterns](docs/explanation/adoption-patterns.md)** (15 min)
   - Choose Pattern A/B/C for your organization
   - Understand template update strategies
   - Plan for scale

### For Development

- 🤖 **[Use LLM Bundles](docs/how-to/use-llm-bundles.md)** - AI-assisted workflow with policy protection
- 🛠️ **[Add HTTP Endpoint](docs/how-to/add-http-endpoint.md)** - Add new routes
- 📗 **[xtask Commands Reference](docs/reference/xtask-commands.md)** - All CLI commands

### For Adoption

- 🚀 **[New Service from Template](docs/how-to/new-service-from-template.md)** - 10-minute setup
- 🚀 **[Adoption Patterns](docs/explanation/adoption-patterns.md)** - Clone vs Upstream vs Generator
- 🚀 **[Branch Protection Profiles](docs/reference/branch-protection-profiles.md)** - CI configuration
- 🚀 **[Template API](TEMPLATE_API.md)** - Stable interfaces

**→ [Browse all documentation](docs/README.md)**

---

## Release Notes

### v2.3.0 (2025-11-17)

**OTLP Tracing + Pilot Infrastructure**

This release completes the three-release observability arc (v2.1.0 → v2.2.0 → v2.3.0) and adds pilot validation infrastructure.

**Added:**

- ✅ OTLP tracing via `telemetry/otlp` feature flag
- ✅ Pilot project tooling (`scripts/create-pilot.sh`)
- ✅ Friction log templates and pilot feature catalog
- ✅ Release Playbook for governed releases
- ✅ Complete observability stack (logs, metrics, traces)

**Strategic Position:**

- Template is **production-ready** for starting new services
- Now in **Pilot Phase** for validation through real usage
- Future evolution will be **demand-driven** based on pilot friction logs

See [CHANGELOG.md](CHANGELOG.md#230---2025-11-17) for full details.

---

### v2.2.0 (2025-11-17)

**Adapter Integration + LLM Ergonomics**

**Added:**

- ✅ DB adapter integration tests (testcontainers + Postgres)
- ✅ gRPC adapter smoke tests
- ✅ BDD scenario for `/metrics` endpoint
- ✅ Enhanced LLM contextpack with richer metadata
- ✅ VSCode task integration (optional)

See [CHANGELOG.md](CHANGELOG.md#220---2025-11-17) for details.

---

### v2.1.0 (2025-11-17)

**Prometheus Metrics Foundation**

**Added:**

- ✅ `/metrics` endpoint with Prometheus format
- ✅ HTTP request tracking with labels (method, path, status)
- ✅ K8s manifests with Prometheus scrape annotations
- ✅ Rego policies enforcing metrics in staging/prod

See [CHANGELOG.md](CHANGELOG.md#210---2025-11-17) for details.

---

### v2.0.0 (2025-11-17)

**Complete Workspace Stabilization**

Major reorganization into production-grade multi-crate workspace with stabilized APIs.

**Added:**

- ✅ 9 production crates with clear boundaries
- ✅ Hexagonal architecture fully realized
- ✅ Async traits for clean adapter interfaces
- ✅ Telemetry scaffolding (tracing, metrics stubs)
- ✅ Rust-native IaC libraries

See [CHANGELOG.md](CHANGELOG.md#200---2025-11-17) for details.

---

### v1.1.0 (2025-11-16)

**Positioning update** - "Heavy governance so LLMs can move fast."

This release clarifies the template's purpose: it's governance-heavy **on purpose** to enable safe, fast AI-assisted development. The structure exists so you can trust-but-verify LLM work without guessing what broke.

**Who should use this:**

- Teams using LLMs/agents as real contributors
- Projects that need invariants, privacy, and production safety
- Developers who want to let AI move fast under policy

**Who shouldn't:**

- Hand-writing everything → Use a lighter Axum starter
- Prototyping without governance → This is overkill

### v1.0.0 (2025-11-13)

**First stable release** - Production-ready template for Rust services.

**Core Infrastructure:**

- ✅ Rust-native xtask tooling (check, bdd, bundle, selftest, quickstart)
- ✅ Runtime architecture: Axum HTTP + hexagonal layering
- ✅ Telemetry: tracing + RUST_LOG configuration
- ✅ BDD vertical integration: tests hit real HTTP stack
- ✅ CI: Template self-test workflow validates full stack

**Governance & Traceability:**

- ✅ AC status mapping: specs → features → tests → code
- ✅ Policy-as-code: Rego policies for ledger, features, flags, privacy
- ✅ LLM context bundler: Curated bundles for AI-assisted development
- ✅ Branch protection profiles: Minimal/Standard/Strict

**Documentation:**

- ✅ Diátaxis framework: tutorials, how-tos, reference, explanations
- ✅ Complete architecture documentation (4,400 words)
- ✅ Adoption guides: greenfield, brownfield, strict
- ✅ Stable API contract in `TEMPLATE_API.md`

**Sample Implementation:**

- ✅ Refund API sample: coherent across specs, OpenAPI, handlers, tests
- ✅ Living enforcement: BDD validates production code paths
