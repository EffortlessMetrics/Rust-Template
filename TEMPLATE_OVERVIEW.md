# Template Overview (Rust + Spec‑as‑Code + LLM‑Native)

This repository is a **template**, not an application. It is designed to be cloned and adapted per service.

## Layout

- `specs/openapi/` – HTTP contracts (OpenAPI)
- `specs/proto/` – gRPC contracts (Protobuf)
- `specs/graphql/` – GraphQL schema
- `specs/events/` – event schemas (JSON Schema + subject registry)
- `specs/db/` – database schema & migrations (Atlas)
- `specs/privacy.yaml` – PII catalogue (fields, owners, retention)
- `specs/userstories/` – story markdown (`US-####`)
- `specs/spec_ledger.yaml` – story → requirement → AC → tests mapping

- `features/FT-*.yaml` – feature manifests (feature id ↔ ACs ↔ flag)
- `flags/` – feature flag registry & environment rollouts

- `policy/` – OPA/Conftest policies for:
  - feature ↔ AC linkage
  - flags & rollouts
  - ledger coverage
  - privacy

- `crates/` – Rust workspace:
  - `core`, `model`, etc. – application code
  - `acceptance` – BDD test runner (Cucumber)
  - `xtask` – dev/CI helper commands

- `.llm/` – LLM context bundles & config
- `scripts/` – helper scripts (schema compat, scope guard, actions pinning, context bundles)

## Typical developer flow

1. **Add or update a story**
   - Create / edit `specs/userstories/US-####.md`.
   - Add or update entries in `specs/spec_ledger.yaml` for new ACs.

2. **Define or change contracts**
   - Edit OpenAPI / Protobuf / events / DB / privacy specs under `specs/**`.
   - CI will block breaking changes unless the baseline is updated intentionally.

3. **Wire features & flags**
   - Create `features/FT-####.yaml` referencing AC IDs.
   - Add / update flags in `flags/registry.yaml` and env rollouts in `flags/rollouts.yaml`.

4. **Implement behaviour**
   - Implement or update Rust code under `crates/**`.
   - Add or update BDD scenarios in `specs/features/*.feature`.
   - Tag scenarios with exactly one `@AC-####` per Scenario/Outline.

5. **Run local checks**
   - `nix develop` (or use the DevContainer) to enter the dev shell.
   - `just checks` – fmt, clippy, nextest (unit / integration)
   - `just bdd` – run acceptance tests (`crates/acceptance`)
   - `cargo run -p xtask -- bundle implement_ac` – generate an LLM context bundle for AC implementation.

6. **Open a PR**
   - CI enforces:
     - Contract breakers (OpenAPI, Proto, events, DB)
     - Lints, coverage, MSRV
     - AC BDD + AC policy
     - Features/flags/privacy policies
     - Nix flake check, security, docs

## Adapting this template

- To disable a subsystem (e.g., no Protobuf), remove the relevant `specs/**` subtree and CI workflow.
- To relax enforcement temporarily (e.g., coverage floor), update the corresponding workflow step.
- For large repos, consider pushing some checks (fuzzing, heavy property tests) to scheduled workflows instead of PR‑blocking checks.

This template is **opinionated** by design. Adjust the opinions, but try not to remove the invariants that make LLM‑assisted development safe: contracts, AC‑level traceability, and policy‑enforced governance.
