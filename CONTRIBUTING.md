# Contributing to Rust Spec-as-Code Template

Thanks for taking the time to contribute.

This repo is a **governed Rust service template + library**. It bakes in:

- spec-as-code (ledger → features → tests → code),
- policy-as-code (Rego),
- LLM-native workflows (bundles, CLAUDE.md),
- Nix-pinned dev environment.

That means contributions should preserve (or improve) those properties, not bypass them.

---

## 1. Before you start

### 1.1 Prerequisites

- Rust (managed via `rustup`, but we use `rust-toolchain.toml` in Nix)
- Nix (recommended) or a native toolchain
- Git, GitHub account

Recommended:

- `just`
- `direnv` (optional, if you want auto devshell activation)

### 1.2 Dev environment

**Golden path (Nix):**

```bash
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# Enter devshell (Rust, conftest, yq, etc. pinned to match CI)
nix develop

# Sanity check
cargo run -p xtask -- selftest
```

**Fallback (no Nix):**

See [`docs/dev-environment.md`](docs/dev-environment.md). In this mode:

* Core checks and BDD work as usual.
* Policy tests are *skipped* locally unless you install `conftest` yourself.
* CI always enforces policies.

---

## 2. Common Workflows

The `xtask` command provides a complete developer experience for this template.
**Use flows, not individual commands, wherever possible.**

### New developer / new machine

1. `nix develop`
2. `cargo xtask doctor`
3. `cargo xtask check`

* `doctor` validates Rust, Cargo, Nix, and supporting tools
* `check` is the fast dev loop (fmt + clippy + tests)

### Adding new behavior (AC-first)

1. Scaffold the AC:
   ```bash
   cargo xtask ac-new AC-TPL-010 "User can cancel account" \
     --story US-TPL-002 --requirement REQ-TPL-CANCEL
   ```

2. Paste the AC snippet into `specs/spec_ledger.yaml` under the requirement

3. Add a `@AC-TPL-010` scenario to the appropriate file in `specs/features/`

4. Regenerate the LLM bundle:
   ```bash
   cargo xtask bundle implement_ac
   ```

5. Implement code + tests with LLM assistance

6. Run BDD and full gate:
   ```bash
   cargo xtask bdd
   cargo xtask selftest
   ```

### Making an architecture decision

```bash
cargo xtask adr-new "Introduce order cancellation"
# edit docs/adr/NNNN-introduce-order-cancellation.md
cargo xtask adr-check
```

* Record the decision in the ADR
* Link it from `specs/spec_ledger.yaml` where appropriate

### Changing dependencies

Whenever you touch `Cargo.toml` or `Cargo.lock`:

```bash
cargo xtask audit
```

* Fix any vulnerabilities or license violations
* If you must accept a risk, document it in ADR-0007 before merging

### Preparing a release

```bash
cargo xtask release-prepare X.Y.Z    # bump versions & seed changelog
# edit CHANGELOG.md entry as needed
cargo xtask release-verify           # selftest + audit + docs-check

git commit -am "Release vX.Y.Z"
git tag -s vX.Y.Z -m "Release vX.Y.Z"  # -s flag for GPG signing (recommended)
git push origin main --follow-tags
```

**Recommended:** Configure GPG signing for tags to ensure release authenticity. See [`docs/how-to/setup-tag-signing.md`](docs/how-to/setup-tag-signing.md) for setup instructions.

### All flows

```bash
cargo xtask help-flows
```

Shows a categorized map of all available commands organized by workflow.

---

## 3. Pre-commit, Selftest, and CI

This template treats governance as code. That shows up in three places:

- **Pre-commit hooks**
- **Local selftest**
- **CI workflows**

### 3.1 What runs on commit

If you've installed the hooks via:

```bash
cargo xtask install-hooks
```

then `git commit` will run:

```text
.git/hooks/pre-commit
  → cargo run -p xtask -- precommit
      → cargo xtask selftest (plus docs+spell checks)
```

Behaviour:

* The hook runs inside the Nix devshell when available.
* It runs the full 8-step selftest, **excluding** `@ci-only` BDD scenarios.
* If anything fails, the commit is blocked with a clear summary of which gate failed.

You should treat a red pre-commit as "something is actually wrong", not "test harness flaked".

### 3.2 What runs in CI

The CI configuration mirrors and extends the local gates:

* **Tier 1:** `cargo xtask selftest` (required on main)
* **Policy checks:** OPA/Rego tests under `policy/*.rego`
* **Coverage/graph:** AC coverage and graph invariants
* **Meta tests (CI-only):**

  * `AC-TPL-BDD-EXIT-CODES` – harness exit semantics
  * `AC-TPL-EXAMPLE-FORK-BUILDS` – example fork builds and passes its own selftest

CI runs **all** BDD scenarios, including those tagged `@ci-only`:

```bash
CUCUMBER_TAG_EXPRESSION="" cargo test -p acceptance --test acceptance
```

Local runs default to `not @ci-only` to avoid recursive selftest and git-worktree flakiness.

### 3.3 Expectations for contributors

Before opening a PR:

```bash
nix develop
cargo xtask kernel-smoke
cargo xtask check
cargo xtask selftest
```

If `selftest` is red:

1. Look at which gate failed (Core checks, BDD, Policy tests, etc.).
2. Use `cargo xtask ac-status` to see which ACs are `[FAIL]`.
3. Fix the root cause or adjust the AC/tests in `specs/spec_ledger.yaml` if you're intentionally changing the contract.

---

## 4. How to work on changes

### 4.1 Branching

Use topic branches:

* `feat/<area>-<short-description>`
* `fix/<area>-<short-description>`
* `docs/<area>-<short-description>`

Example:

* `feat/app-http-refund-api`
* `fix/policy-k8s-envfrom`
* `docs/llm-workflow-clarifications`

### 4.2 Commit messages

Short, imperative subject:

* `feat(app-http): add refund endpoint`
* `fix(policy): avoid var shadowing in envFrom rule`
* `docs: clarify Nix-first dev workflow`

If a commit addresses a specific AC or issue, reference it in the body:

```text
Implements AC-TPL-007 for metrics endpoint
Fixes #123
```

---

## 5. What "done" means here

### 5.1 Always run `selftest`

Before opening a PR:

```bash
# Inside nix develop (preferred)
cargo run -p xtask -- selftest
```

That runs:

1. `fmt` + `clippy` + tests
2. BDD (Cucumber)
3. AC status mapping
4. LLM bundler checks
5. Policy tests (if `conftest` available, always in CI)

If you *must* skip some work locally (e.g., no Nix), make sure CI is green.

### 5.2 Supply chain workflows

CI includes supply-chain hardening workflows (SBOM and provenance generation):

- **Supply chain workflows**:
  - `.github/workflows/ci-supply-chain.yml` runs on `v*.*.*` tags
  - Generates an SBOM (`sbom.spdx.json`) and build provenance for a source tarball
  - If this workflow fails on a release tag, fix it before publishing the GitHub Release

### 5.3 Keep the hexagonal architecture intact

* Domain logic stays in `business-core` and `model`.
* Adapters (HTTP, gRPC, DB) live in their crates and depend inward.
* Don't pull adapters into `business-core`.

If you're unsure: check `docs/explanation/hexagonal-architecture.md`.

### 5.4 Respect the governance model

When you add or change behavior:

1. **Spec** – update `specs/spec_ledger.yaml` with ACs and stories if needed.
2. **BDD** – add/update `.feature` scenarios in `specs/features/`.
3. **Code** – implement in the right crate (model/core/app-http/etc.).
4. **Tests** – add unit/integration tests as needed.
5. **Policies** – if infra/LLM behavior changes, update Rego + testdata.
6. **ADRs** – for significant architectural changes:
   - Copy `docs/templates/ADR-TEMPLATE.md` to `docs/adr/00XX-your-decision.md`
   - Fill in context, decision, and consequences
   - Reference it in `specs/spec_ledger.yaml` (at story/requirement/AC level as appropriate)
   - Run `cargo run -p xtask -- adr-check` to validate references

---

## 6. Making changes: step-by-step

A typical feature flow:

1. Pick an AC (or add one) in `specs/spec_ledger.yaml`.

2. Add/update the matching scenario in `specs/features/*.feature`.

3. Generate an LLM bundle:

   ```bash
   cargo run -p xtask -- bundle implement_ac
   ```

4. Use the instructions and prompts in `CLAUDE.md` to drive LLM-native changes.

5. Apply/curate the changes locally.

6. Run:

   ```bash
   cargo run -p xtask -- selftest
   ```

7. Commit and open a PR.

---

## 7. Tests

Useful commands:

```bash
# Fast path
cargo run -p xtask -- check

# Full template validation
cargo run -p xtask -- selftest

# Unit tests
cargo test --workspace

# BDD only
cargo run -p xtask -- bdd

# Policy tests only (if conftest installed)
cargo run -p xtask -- policy-test
```

Some tests are marked `#[ignore]` when they:

* manipulate global state (e.g., `set_current_dir`),
* or require external dependencies (e.g., Docker).

Those tests include comments explaining how to run them explicitly.

---

## 8. Style and tooling

* Rust code: `rustfmt` + `clippy -D warnings`
* Rego: keep rules small and name things for intent, not implementation.
* YAML: prefer explicit structure over "clever" anchors for template users.
* Docs: short sections, headings, and examples. Avoid walls of text where possible.

---

## 9. Questions / Discussions

If you're unsure whether a change fits:

* Open a draft PR and describe:

  * the problem,
  * the AC or use case,
  * your proposed approach.
* Or open an issue referencing relevant ACs / features / policies.

Thanks again for contributing. This template is expressly meant to be **used** and **evolved** by people who care about governed, AI-assisted Rust services.
