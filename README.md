# Rust Spec-as-Code Template (v1.1.0)

**Heavy governance so LLMs can move fast**

AC-first, policy-first template for building Rust services with AI-assisted development.

- See `TEMPLATE_OVERVIEW.md` for a high-level overview of this template.
- See `IMPLEMENTATION_PLAN.md` and `ADOPTION.md` for rollout and implementation guidance.

**Key Features:**
- 🦀 Rust-native development with xtask orchestration
- 🎯 AC-first workflow: specs → tests → implementation
- 🔒 Policy-as-code governance (OPA/Rego)
- 🤖 LLM context bundles for AI-assisted development
- 🏗️ Hexagonal architecture with Axum + tracing
- 🧪 BDD acceptance tests with real HTTP integration


### Who This Is For

This template is opinionated and governance-heavy **on purpose**:

- Built for teams using LLMs/agents as real contributors to their codebase
- Provides ACs, policies, and bundles so you can **trust but verify** AI work
- Assumes you care about invariants, privacy, and production safety

If you're hand-writing everything and want a light Axum starter, this is overkill.

If you want to let an LLM move fast inside a Rust service **without** guessing what it broke, this is the right weight.


### Current Status

- Runtime & AC workflow: ✅ Fully working
- LLM bundler: ✅ Producing bounded, governed context
- Policy suite: ✅ Enforcing ledger, features, K8s, privacy rules
- Selftest harness: ✅ Honest failure reporting

This template is at v1.1.0 and pilot-ready.


## Quick Start

**Fastest path** - one command validates everything:

```bash
git clone <your-repo-url> && cd Rust-Template
nix develop
cargo run -p xtask -- quickstart
```

This runs all checks, tests, and bundler to prove the template works.

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

```
crates/
├── app-http/       → HTTP adapter (Axum, routes, DTOs)
├── core/           → Domain logic (business rules, no HTTP)
├── model/          → Domain entities and value objects
├── telemetry/      → Observability (tracing setup)
├── acceptance/     → BDD tests (cucumber-rs)
└── xtask/          → Dev/CI tooling
```

**Key pattern:** Dependencies point inward
`app-http` → `core` ✓  (adapters call domain)
`core` → `app-http` ✗  (domain never calls adapters)

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
