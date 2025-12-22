# Template Implementation Plan

This document describes the work needed to instantiate or maintain this template
from an empty repository to a fully working, spec-as-code, AC-driven,
policy-enforced Rust service template.

It is organised in phases. Teams can execute them sequentially or in parallel
where dependencies allow.

---

## Phase 0 – Bootstrap & plumbing

### 0.1 Repository and base config

- Initialise a Git repository (ideally as a GitHub template repo).
- Add `.gitignore` tuned for Rust, Nix, Node, and build artifacts.
- Add `LICENSE` and a minimal `README.md` announcing this as a template.
- Enable Issues / Discussions if you plan to collect feedback there.

### 0.2 Nix flake

- Create or update `flake.nix` with:
  - a `devShells.default` that provides:
    - Rust toolchain (stable) with `rust-src`
    - `cargo-nextest`, `cargo-llvm-cov`, `sccache`
    - `conftest`, `gitleaks`, `jq`, `yq`
    - `cargo-deny`, `cargo-audit`
    - `just`, `nodejs` (if needed)
  - an optional `shellHook` that:
    - prepends `.tools/bin` to `PATH`
    - prints a short “welcome” message with key commands

- Add or update `flake.lock` once you are happy with inputs.

### 0.3 DevContainer (optional but recommended)

- Add `.devcontainer/devcontainer.json` that:
  - uses a Nix-capable base image
  - runs `nix develop -c ./bootstrap-tools.sh` as a post-create command
- Validate that a developer can:
  - open the repo in a DevContainer
  - run `nix develop`
  - run `cargo run -p xtask -- check`

---

## Phase 1 – Specification Plane

### 1.1 Directory skeleton

Create the following directories and placeholder files (if not already present):

- `specs/openapi/`         → `openapi.yaml` (valid but minimal)
- `specs/proto/`           → README or stub `.proto`
- `specs/graphql/`         → `schema.graphql` stub
- `specs/events/json-schema/`
- `specs/db/atlas/migrations/`
- `specs/userstories/`
- `specs/features/`
- `features/`
- `flags/`
- `policy/`
- `docs/`

Ensure the tree is visible and navigable from day one.

### 1.2 Spec ledger

- Define or refine `specs/spec_ledger.yaml` with structure:

  - `stories[]` each with:
    - `id` (e.g. `US-0001`)
    - `title`
    - `requirements[]`:
      - `id` (e.g. `REQ-0001`)
      - `text`
      - `acceptance_criteria[]`:
        - `id` (e.g. `AC-0001`)
        - `text`
        - `tests[]` with `{type, tag/path}`

- Seed at least one story with one requirement and one AC.
- Ensure AC IDs (`AC-####`) match the tagging scheme in `.feature` files.

### 1.3 Gherkin features

- Ensure `specs/features/*.feature` exist with:
  - `Feature: ...`
  - One or more `Scenario` blocks, each with exactly one `@AC-####` tag.
  - At least one `Scenario Outline` with `Examples` to exercise Outline handling.

### 1.4 Features and flags

- Ensure at least one `features/FT-0001-example.yaml` containing:
  - `id`, `title`, `owner`, `tracker`
  - `acceptance_criteria: [AC-0001]`
  - `flag: example_flag`

- Ensure `flags/registry.yaml` contains:

  ```yaml
  flags:
    - key: example_flag
      owner: team-example
      description: Example feature flag
      default: false
      expires_at: 2026-01-01
      linked_features: [FT-0001]
  ```

- Ensure `flags/rollouts.yaml` contains:

  ```yaml
  environments:
    dev:     { example_flag: 100 }
    staging: { example_flag:  50 }
    prod:    { example_flag:   0 }
  ```

### 1.5 Privacy spec

- Ensure `specs/privacy.yaml` contains at least one PII entry:

  ```yaml
  fields:
    - path: user.email
      classification: PII
      owner: team-identity
      retention: "365d"
  ```

- Keep this aligned with `policy/privacy.rego` expectations.

---

## Phase 2 – Rust Workspace & Crates

### 2.1 Workspace and core crates

- Confirm root `Cargo.toml`:

  - `[workspace]` with `members = ["crates/*"]`
  - `rust-version = "<MSRV>"`

- Ensure `crates/core` and `crates/model` exist:

  - `crates/core/src/lib.rs` with minimal domain code.
  - `crates/model/src/lib.rs` for shared types (stubbed is fine).

### 2.2 Acceptance crate

- Ensure `crates/acceptance`:

  - Depends on a BDD framework in `Cargo.toml`.
  - Provides a test harness in `tests/acceptance.rs` that:
    - points at `specs/features`
    - runs feature files
    - emits JUnit XML at `target/junit/acceptance.xml`

- Implement placeholder step definitions to get a green run.

### 2.3 xtask crate

- Ensure `crates/xtask` implements a binary with subcommands:
  - `check` → fmt + clippy + nextest
  - `bdd` → run acceptance tests
  - `bundle <task>` → invoke LLM bundler script

---

## Phase 3 – Tooling & Checksums

### 3.1 bootstrap-tools.sh

- Implement or refine `bootstrap-tools.sh` to:

  - Detect OS / arch
  - Download pinned versions of external tools into `.tools/bin`:
    - `oasdiff`, `buf`, `atlas`, etc.
  - Verify their SHA-256 checksums against `scripts/tools.sha256`
    when `ENFORCE_CHECKSUMS=1`.

### 3.2 Tool cache in CI

- In workflows that use `.tools/bin`, ensure an `actions/cache` step with:

  - `path: .tools/bin`
  - `key: tools-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('bootstrap-tools.sh') }}`

- Call `bootstrap-tools.sh` after the cache restore.

---

## Phase 4 – Policy (Rego)

### 4.1 Ledger policy

- Implement or refine `policy/ledger.rego` so that it:

  - Accepts ledger JSON as input.
  - Emits `deny` for any AC without at least one test mapping.

### 4.2 Features, flags, and privacy policies

- Ensure:

  - `policy/features.rego` → features reference only known AC IDs.
  - `policy/flags.rego` → flags and rollouts are valid and consistent.
  - `policy/flags_warn.rego` → warn on flags near expiry.
  - `policy/privacy.rego` → privacy entries must have owner and valid retention.

### 4.3 Policy verification

- Ensure a CI workflow runs `conftest verify -p policy` to catch
  syntax/package issues early.

---

## Phase 5 – CI Workflows

### 5.1 Contracts

- Confirm workflows exist and are wired for:
  - OpenAPI breaking checks + lint (`ci-openapi.yml`)
  - Proto breaking checks (`ci-proto.yml`)
  - Events compatibility (`ci-events.yml`, if applicable)
  - DB schema & migrations (`ci-db.yml`)
  - Privacy policy (`ci-privacy.yml`)

### 5.2 Quality

- Confirm workflows exist and are wired for:
  - Lints (fmt, clippy, nextest) – `ci-lints.yml`
  - Coverage with a JSON-based floor – `ci-coverage.yml`
  - MSRV build using `rust-version` – `ci-msrv.yml`
  - Nix flake check – `ci-nix.yml`

### 5.3 AC & governance

- Confirm workflows exist and are wired for:
  - Gherkin lint + tagging rules – `ci-gherkin.yml`
  - Acceptance tests + AC status markdown + ledger policy – `ci-ac.yml`
  - Features policy – `ci-features.yml`
  - Flags policy – `ci-flags.yml`
  - Weekly flag expiry warnings – `ci-flags-warn.yml`
  - Governance helper (impacted specs, PR title hints) – `ci-governance.yml`
  - Policy verification – `ci-policy-verify.yml`

### 5.4 Security & maintenance

- Confirm workflows exist and are wired for:
  - Security (CodeQL, gitleaks, `cargo-deny`, `cargo-audit`) – `ci-security.yml`
  - GH Actions SHA pinning via `scripts/pin-actions.sh` – `maintenance-pin-actions.yml`
  - Release SBOM & signing (tagged builds) – `release-sbom-sign.yml`

---

## Phase 6 – LLM Support

### 6.1 Contextpack configuration

- Define `.llm/contextpack.yaml` with tasks such as:
  - `implement_ac`
  - `implement_feature`

- Each task should specify:
  - A set of include globs (specs, ledger, features, relevant crates)
  - A `max_bytes` budget.

### 6.2 Bundler script

- Implement or refine `scripts/make-context.sh` so it:

  - Reads `.llm/contextpack.yaml`
  - Uses `git ls-files` + globs to select files
  - Respects `.llm/.llmignore` (and `.gitignore` where appropriate)
  - Writes `.llm/bundle/<task>.md` with clear file boundaries
  - Enforces per-task size budgets.

### 6.3 xtask integration

- Wire `xtask bundle <task>` to call the bundler script under `nix develop`.

---

## Phase 7 – Documentation & Adoption

### 7.1 Core template docs

- Maintain:
  - `TEMPLATE_OVERVIEW.md` → high-level description of the template.
  - `REQUIRED_CHECKS.md` → which workflows to mark as required on `main`.
  - `IMPLEMENTATION_PLAN.md` (this file).
  - Optionally: requirements, design, and architecture docs.

### 7.2 Diátaxis skeleton

- Add or refine an initial docs tree:
  - Tutorials → “first service”, “first AC change”
  - How-tos → “change OpenAPI safely”, “adjust coverage floor”
  - Reference → “CI workflows”, “coverage behavior”, “schema layout”
  - Explanations → “spec-as-code”, “LLM-native devex”, “governance & risk”

### 7.3 Branch protection and profiles

- Document recommended profiles:
  - Full (all contract + quality + policy checks required)
  - Minimal/transitional (for brownfield, with some checks warn-only)
- Provide explicit workflow names for branch protection configuration.

---

Treat this plan as a living document. As you evolve the template, update it so
future teams can reinstantiate the same architecture without relying on memory.
