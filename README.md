# Rust Spec-as-Code Template (v3.4.7)

- See `TEMPLATE_OVERVIEW.md` for a high-level overview of this template.
- See `IMPLEMENTATION_PLAN.md` and `ADOPTION.md` for rollout and implementation guidance.

- Nix flake is the **canonical dev environment**; DevContainer wraps it.
- Contracts & policies as code; interface locks as blocking gates.
- Feature & flag manifests as code; OPA policy for governance.
- LLM context bundles for targeted editing.


## Quick Start

**Fastest path** - one command validates everything:

```bash
git clone <your-repo-url> && cd Rust-Template
nix develop
cargo run -p xtask -- quickstart
```

This runs all checks, tests, and bundler to prove the template works.

### Command Reference

All development operations go through `xtask`:

| Task                       | Command                             | What it does                            |
|----------------------------|-------------------------------------|-----------------------------------------|
| Full self-test suite       | `xtask selftest`                    | check + bdd + ac-status + bundler + policies |
| Quick validation           | `xtask quickstart`                  | Lightweight validation for first use    |
| Format + lint + tests      | `xtask check`                       | cargo fmt, clippy, test                 |
| Acceptance tests           | `xtask bdd`                         | Run cucumber scenarios, emit JUnit XML  |
| LLM context bundle         | `xtask bundle <task>`               | Generate focused context for AI coding  |

**Examples:**
```bash
cargo run -p xtask -- check                    # Before every commit
cargo run -p xtask -- bdd                      # Test acceptance criteria
cargo run -p xtask -- bundle implement_ac      # Get context for AC work
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

**📚 Complete documentation in `docs/`** - [Start Here](docs/README.md)

### For New Users

- 📘 **[Getting Started Tutorial](docs/tutorials/getting-started.md)** (30 min) - Clone, run, make first change
- 📕 **[Architecture Explanation](docs/explanation/architecture.md)** (20 min read) - Design & philosophy
- 📗 **[xtask Commands Reference](docs/reference/xtask-commands.md)** - All CLI commands

### For Development

- 🛠️ **[First AC Change](docs/tutorials/first-ac-change.md)** - Complete AC workflow
- 🛠️ **[Add HTTP Endpoint](docs/how-to/add-http-endpoint.md)** - Add new routes
- 🛠️ **[Use LLM Bundles](docs/how-to/use-llm-bundles.md)** - AI-assisted coding

### For Adoption

- 🚀 **[New Service from Template](docs/how-to/new-service-from-template.md)** - 10-minute setup
- 🚀 **[Branch Protection Profiles](docs/reference/branch-protection-profiles.md)** - CI configuration
- 🚀 **[Template API](TEMPLATE_API.md)** - Stable interfaces

**→ [Browse all documentation](docs/README.md)**

---

### v3.4.7 changes
- Added `IMPLEMENTATION_PLAN.md` capturing phased work to instantiate or re-implement the template
- Added `ADOPTION.md` with greenfield, brownfield, and strict adoption guidance
- Seeded a Diátaxis-aligned docs tree under `docs/` (tutorials, how-tos, reference, explanations)
- Kept existing AC mapping, contract gates, coverage, and checksum behavior from prior 3.4.x releases