# Rust Spec-as-Code Template (v3.4.7)

- See `TEMPLATE_OVERVIEW.md` for a high-level overview of this template.
- See `IMPLEMENTATION_PLAN.md` and `ADOPTION.md` for rollout and implementation guidance.

- Nix flake is the **canonical dev environment**; DevContainer wraps it.
- Contracts & policies as code; interface locks as blocking gates.
- Feature & flag manifests as code; OPA policy for governance.
- LLM context bundles for targeted editing.


### v3.4.7 changes
- Added `IMPLEMENTATION_PLAN.md` capturing phased work to instantiate or re-implement the template
- Added `ADOPTION.md` with greenfield, brownfield, and strict adoption guidance
- Seeded a Diátaxis-aligned docs tree under `docs/` (tutorials, how-tos, reference, explanations)
- Kept existing AC mapping, contract gates, coverage, and checksum behavior from prior 3.4.x releases