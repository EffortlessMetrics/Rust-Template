# Template Adoption Guide

This document explains how to adopt this template for new and existing services,
and how to tune which checks are enforced on your `main` branch.

The template is opinionated, but you can adopt it in phases.

---

## 1. Profiles

We recommend thinking about adoption in terms of **profiles**.

### Minimal

For experimental or very small services.

- Required checks on `main`:
  - `Lints`
  - `Nix Flake Check`
  - `MSRV`
- Optional (run but not required):
  - `Coverage`
  - Contract breakers for domains actually used (`OpenAPI`, `Proto`, `DB`, `EventSchemas`, `Privacy`)
  - `Security`

### Standard (default)

For most production services.

- Required checks on `main`:
  - Quality / tests:
    - `Lints`
    - `Coverage`
    - `MSRV`
    - `Nix Flake Check`
  - Contracts:
    - `OpenAPI` (if `specs/openapi` exists)
    - `Proto` (if `specs/proto` exists)
    - `DB` (if `specs/db` exists)
    - `EventSchemas` (if `specs/events` exists)
    - `Privacy` (if `specs/privacy.yaml` exists)
  - Requirements & policy:
    - `ACs`
    - `Gherkin`
    - `Features`
    - `Flags`
    - `PolicyVerify`
  - Security:
    - At least the dependency and secrets jobs from `Security`

- Optional but recommended:
  - `Docs`

### Strict

For systems with tight compliance / risk requirements.

- Everything in **Standard**, plus:

  - All contract gates required (`OpenAPI`, `Proto`, `DB`, `EventSchemas`, `Privacy`)
  - `Security` in full (CodeQL + deps + secrets)
  - `ScopeGuard` (if configured)
  - `Docs` (if you depend on generated docs for customers)

---

## 2. Greenfield adoption (new services)

Suggested sequence for a brand new service:

1. **Create a repo from this template.**
   - Keep the initial specs, ledger, features, and flags even if they are “toy” examples.
   - Confirm `nix develop`, `xtask check`, and `xtask bdd` work locally.

2. **Turn on Minimal profile.**
   - Mark `Lints`, `MSRV`, and `Nix Flake Check` as required on `main`.
   - Keep coverage and contract gates non‑required initially.

3. **Define your first real feature and ACs.**
   - Add user stories, requirements, and ACs in `specs/spec_ledger.yaml`.
   - Add Gherkin scenarios under `specs/features` with `@AC-####` tags.
   - Add a `features/FT-####.yaml` manifest referencing those ACs and a flag.

4. **Wire the behavior.**
   - Implement the behavior in `crates/core` and supporting crates.
   - Add or update acceptance step definitions in `crates/acceptance`.
   - Run `xtask bdd` locally and fix failures.

5. **Promote to Standard profile.**
   - Mark contract breakers, `ACs`, `Gherkin`, `Features`, `Flags`, and `PolicyVerify`
     as required on `main`.
   - Set a reasonable coverage floor in `ci-coverage.yml` (e.g. 60%) and make
     `Coverage` required once the service has a base of tests.

6. **Iterate.**
   - As you add more stories, repeat the cycle: ledger → specs → features → flags → tests.

---

## 3. Brownfield adoption (existing services)

Suggested staged approach for migrating an existing service.

1. **Start with hygiene gates.**
   - Introduce `Lints`, `MSRV`, and `Nix Flake Check` first.
   - Enable `Security` (at least deps + secrets) if your service is internet-facing.
   - Make these required on `main`.

2. **Stabilise contracts.**
   - Add OpenAPI/Proto/DB specs that reflect the current service behavior.
   - Wire `OpenAPI`, `Proto`, `DB`, `EventSchemas`, and `Privacy` workflows to run,
     but keep them *non‑required* initially.
   - Fix the most critical contract drift until these jobs are usually green.
   - Once they’re stable, mark them as required checks.

3. **Introduce the ledger and ACs gradually.**
   - Start by modeling only one or two critical flows in `specs/spec_ledger.yaml`.
   - Add or tag Gherkin scenarios for those flows with `@AC-####` in `specs/features`.
   - Implement or connect acceptance tests in `crates/acceptance` so that `ci-ac`
     exercises those ACs.

4. **Turn on AC and feature/flag policy.**
   - Enable `ACs`, `Gherkin`, `Features`, `Flags`, and `Privacy` workflows on PRs.
   - Make them required once you have coverage for the most important flows.
   - Fix policy violations as you touch areas of the system instead of trying to
     retrofit the entire system in one go.

5. **Phase in coverage.**
   - Set a conservative coverage floor (e.g. 30–40%) in `ci-coverage.yml`.
   - Mark `Coverage` as required only when you are confident most new work meets it.
   - Consider higher floors for specific crates (e.g. `crates/core/critical-*`).

6. **Move to Standard or Strict profile.**
   - Once contracts, ACs, and policies are reliable, switch branch protection from
     Minimal to Standard or Strict, depending on the risk profile of the service.

---

## 4. Strict profile adoption

Use this when you need the template’s full power from day one (e.g. new systems
in regulated environments).

1. Start as in **Greenfield**, but set branch protection to **Strict** immediately.
2. Require all relevant contract gates and security jobs.
3. Ensure every story/feature that goes live has ACs traced and enforced via `ACs`,
   `Gherkin`, `Features`, and `Flags`.
4. Keep `ADOPTION.md` in your service updated with any local deviations from this
   template’s defaults.
