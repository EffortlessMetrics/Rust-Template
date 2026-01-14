# Required checks for `main`

These are the workflows this template expects to be **required** on the `main` branch.

## Contracts / interfaces

- `OpenAPI`
- `Proto`
- `EventSchemas`
- `DB`
- `Privacy` (if `specs/privacy.yaml` is in active use)

## Behaviour / tests

- `Lints`
- `ACs` (acceptance BDD)
- `MSRV`

## Policy / governance

- `Features`
- `Flags`
- `ACs` (policy) / `PolicyVerify`
- `Nix Flake Check`
- `Security`
- `ScopeGuard` (if enabled)
- `Docs`

## Advisory jobs (usually not required)

- `Coverage` (tag-only release receipt; do not require on PRs)
- `FlagsWarn` (weekly, scheduled)
- `Maintenance – Pin Actions`
- `Release` (SBOM & signing, tag‑only)

---

## Profiles (suggested)

These names are for documentation; GitHub branch protection only sees
individual checks. Use them as shorthand when discussing policies.

- **Profile: Minimal**
  - Required: `Lints`, `MSRV`, `Nix Flake Check`
  - Optional: `Coverage` (tag-only), `OpenAPI`, `Proto`, `DB`, `EventSchemas`, `Privacy`, `Security`

- **Profile: Standard**
  - Required: `Lints`, `MSRV`, `Nix Flake Check`
  - Required: `OpenAPI`, `Proto`, `DB`, `EventSchemas`, `Privacy` (where specs exist)
  - Required: `ACs`, `Gherkin`, `Features`, `Flags`, `PolicyVerify`
  - Required: `Security` (at least deps + secrets)
  - Optional: `Coverage` (tag-only), `Docs`

- **Profile: Strict**
  - Required: all of the above, plus:
    - `ScopeGuard` (if enabled)
    - Full `Security` (CodeQL + deps + secrets)
    - `Docs` where documentation is part of your external contract
  - Optional: `Coverage` (tag-only; enforce on release tags, not PRs)
