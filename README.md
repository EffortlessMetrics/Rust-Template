# Rust Spec-as-Code Template (v3.4.7)

- See `TEMPLATE_OVERVIEW.md` for a high-level overview of this template.
- See `IMPLEMENTATION_PLAN.md` and `ADOPTION.md` for rollout and implementation guidance.

- Nix flake is the **canonical dev environment**; DevContainer wraps it.
- Contracts & policies as code; interface locks as blocking gates.
- Feature & flag manifests as code; OPA policy for governance.
- LLM context bundles for targeted editing.


## Quick Start

Clone and verify in 2 minutes:

```bash
# 1. Clone the template
git clone <your-repo-url>
cd Rust-Template

# 2. Enter Nix development shell (installs all tools)
nix develop

# 3. Run all checks
cargo run -p xtask -- check

# 4. Run BDD acceptance tests
cargo run -p xtask -- bdd

# 5. Generate LLM context bundle
cargo run -p xtask -- bundle implement_ac
```

**Expected results:**
- ✓ All checks pass (format, clippy, tests)
- ✓ BDD scenario passes, creates `target/junit/acceptance.xml`
- ✓ Bundle created at `.llm/bundle/implement_ac.md`

**What's working:**
- `xtask` CLI as single entrypoint for all operations
- BDD acceptance tests with cucumber-rs
- AC status mapping from tests → features → ledger
- OPA/Rego policies for ledger, features, flags, privacy
- LLM context bundler for targeted AI assistance
- 22 GitHub Actions workflows for comprehensive CI

**Next steps:**
- See `docs/how-to/new-service-from-template.md` for adoption guide
- See `TEMPLATE_API.md` for stable interface documentation
- See `docs/tutorials/first-ac-change.md` for AC-first development

---

### v3.4.7 changes
- Added `IMPLEMENTATION_PLAN.md` capturing phased work to instantiate or re-implement the template
- Added `ADOPTION.md` with greenfield, brownfield, and strict adoption guidance
- Seeded a Diátaxis-aligned docs tree under `docs/` (tutorials, how-tos, reference, explanations)
- Kept existing AC mapping, contract gates, coverage, and checksum behavior from prior 3.4.x releases