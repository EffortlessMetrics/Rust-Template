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
| Validate template          | `xtask quickstart`                  | Run all checks + BDD + bundler          |
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
- ✅ **BDD acceptance tests** - cucumber-rs with JUnit output
- ✅ **AC status mapping** - tests → features → ledger traceability
- ✅ **Policy-as-code** - OPA/Rego for ledger, features, flags, privacy
- ✅ **LLM bundler** - Curated context for AI-assisted development
- ✅ **CI workflows** - 22 GitHub Actions for comprehensive validation

### Next Steps

- **Adopt the template**: `docs/how-to/new-service-from-template.md`
- **API reference**: `TEMPLATE_API.md`
- **AC-first development**: `docs/tutorials/first-ac-change.md`

---

### v3.4.7 changes
- Added `IMPLEMENTATION_PLAN.md` capturing phased work to instantiate or re-implement the template
- Added `ADOPTION.md` with greenfield, brownfield, and strict adoption guidance
- Seeded a Diátaxis-aligned docs tree under `docs/` (tutorials, how-tos, reference, explanations)
- Kept existing AC mapping, contract gates, coverage, and checksum behavior from prior 3.4.x releases