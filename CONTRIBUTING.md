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

- Core checks and BDD work as usual.
- Policy tests are *skipped* locally unless you install `conftest` yourself.
- CI always enforces policies.

---

## 2. Common Workflows

The `xtask` command provides a complete developer experience for this template.
**Use flows, not individual commands, wherever possible.**

### New developer / new machine

1. `nix develop`
2. `cargo xtask doctor`
3. `cargo xtask check`

- `doctor` validates Rust, Cargo, Nix, and supporting tools
- `check` is the fast dev loop (fmt + clippy + tests)

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

- Record the decision in the ADR
- Link it from `specs/spec_ledger.yaml` where appropriate

### Changing dependencies

Whenever you touch `Cargo.toml` or `Cargo.lock`:

```bash
cargo xtask audit
```

- Fix any vulnerabilities or license violations
- If you must accept a risk, document it in ADR-0007 before merging

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

### Version bump checklist (Docs-as-Code v2)

Version numbers are centrally tracked in `specs/spec_ledger.yaml`. All other files **derive from** this canonical source.

**Canonical version authority:** `specs/spec_ledger.yaml → metadata.template_version`

**Files that must match:**

| File | Pattern |
|------|---------|
| `README.md` | H1 `# ... (vX.Y.Z)` or `**Template Version:** vX.Y.Z` |
| `CLAUDE.md` | H1 `# ... (vX.Y.Z)` |
| `docs/ROADMAP.md` | H1 `# ... (vX.Y.Z)` |
| `docs/KERNEL_SNAPSHOT.md` | H1 `# Kernel Snapshot vX.Y.Z` |
| `docs/explanation/TEMPLATE-CONTRACTS.md` | `**Template Version:** vX.Y.Z` |
| `specs/service_metadata.yaml` | `template_version: vX.Y.Z` |
| `specs/doc_index.yaml` | `template_version: "X.Y.Z"` |
| `CHANGELOG.md` | First version section `## [X.Y.Z]` after `[Unreleased]` |

**Bump workflow:**

1. Update the canonical version:

   ```bash
   # In specs/spec_ledger.yaml
   metadata:
     template_version: "X.Y.Z"
     last_updated: "YYYY-MM-DD"
   ```

2. Run `cargo xtask release-prepare X.Y.Z` (updates most files automatically)

3. Run `cargo xtask docs-check` to see any remaining mismatches

4. Fix any files that weren't auto-updated

5. Run `cargo xtask selftest` to validate

6. Tag as `vX.Y.Z-kernel`:

   ```bash
   git tag -s vX.Y.Z-kernel -m "Kernel release vX.Y.Z"
   ```

**Verify canonical version programmatically:**

```bash
cargo xtask version --json | jq .kernel_version
```

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

**Behaviour:**

- The hook runs inside the Nix devshell when available.
- It **automatically fixes and stages**:
  - Rust formatting (`cargo fmt --all`)
  - Skills format (`SKILL.md` tidiness)
  - AC status report (`docs/feature_status.md`)
- It **blocks the commit** (hard gates) on:
  - Clippy warnings
  - Test failures
  - Skills/Agents governance violations (policy errors)
- It **warns but doesn't block** (soft gates) on:
  - Documentation consistency issues
  - Spelling errors (unless `XTASK_STRICT_PRECOMMIT=1`)
- It runs the full 8-step selftest, **excluding** `@ci-only` BDD scenarios.

**TL;DR:** Just run `git commit`. Mechanical fixes are silent. Real failures show up clearly. If precommit fails, run `nix develop -c cargo xtask precommit` to see details, fix the issue, and commit again.

You should treat a red pre-commit as "something is actually wrong", not "test harness flaked".

### 3.2 What runs in CI

The CI configuration mirrors and extends the local gates:

- **Tier 1:** `cargo xtask selftest` (required on main)
- **Policy checks:** OPA/Rego tests under `policy/*.rego`
- **Coverage/graph:** AC coverage and graph invariants
- **Meta tests (CI-only):**

  - `AC-TPL-BDD-EXIT-CODES` – harness exit semantics
  - `AC-TPL-EXAMPLE-FORK-BUILDS` – example fork builds and passes its own selftest

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

### 3.4 Version bump checklist (AC-PLT-009 / AC-PLT-010)

When bumping the template/kernel version:

1. Update `specs/spec_ledger.yaml` `metadata.template_version` and `last_updated`.
2. Update `README.md` template/kernel badges and "Kernel Version" in §14.
3. Add/update the `[X.Y.Z]` section in `CHANGELOG.md`.
4. Update `docs/ROADMAP.md` frontmatter `last_updated` and H1 `(vX.Y.Z)`.
5. If needed, update `docs/KERNEL_SNAPSHOT.md`.
6. Run:

   ```bash
   nix develop
   cargo xtask docs-check
   cargo xtask selftest
   ```

If docs-check or selftest fail, fix the misalignment before tagging.

---

## 4. How to work on changes

### 4.1 Branching

Use topic branches:

- `feat/<area>-<short-description>`
- `fix/<area>-<short-description>`
- `docs/<area>-<short-description>`

Example:

- `feat/app-http-tasks-api`
- `fix/policy-k8s-envfrom`
- `docs/llm-workflow-clarifications`

### 4.2 Making documentation changes

For any doc changes, run `cargo xtask docs-check` before pushing.
CI treats doc issues as hard failures via `XTASK_STRICT_PRECOMMIT=1`.

See [`docs/how-to/change-docs-safely.md`](docs/how-to/change-docs-safely.md) for detailed workflows.

### 4.3 Commit messages

Short, imperative subject:

- `feat(app-http): add tasks endpoint`
- `fix(policy): avoid var shadowing in envFrom rule`
- `docs: clarify Nix-first dev workflow`

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

- Domain logic stays in `business-core` and `model`.
- Adapters (HTTP, gRPC, DB) live in their crates and depend inward.
- Don't pull adapters into `business-core`.

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

- manipulate global state (e.g., `set_current_dir`),
- or require external dependencies (e.g., Docker).

Those tests include comments explaining how to run them explicitly.

---

## 8. Style and tooling

- Rust code: `rustfmt` + `clippy -D warnings`
- Rego: keep rules small and name things for intent, not implementation.
- YAML: prefer explicit structure over "clever" anchors for template users.
- Docs: short sections, headings, and examples. Avoid walls of text where possible.

---

## 9. How to Evolve the Kernel

This section is for changes that affect the **kernel contract** – the stable surfaces that forks and IDPs rely on.

### 9.1 What's a kernel change?

A change is a **kernel change** if it modifies:

- `must_have_ac` ACs in `specs/spec_ledger.yaml`
- `/platform/*` endpoint response shapes
- `xtask` governance commands (selftest, ac-status, kernel-status, idp-snapshot)
- Schema files (`specs/openapi/**`, `specs/platform_schema.yaml`)
- Kernel documentation (`docs/KERNEL_SNAPSHOT.md`, `docs/IDP_CELL_CONTRACT.md`)

### 9.2 Kernel change protocol

> **Rule:** Any kernel contract change requires ADR → version bump → kernel tag.

**Step-by-step:**

1. **Draft an ADR** explaining the change:

   ```bash
   cargo xtask adr-new "Change X in kernel contract"
   # Edit docs/adr/NNNN-change-x-in-kernel-contract.md
   ```

2. **Update the canonical version** in `specs/spec_ledger.yaml`:

   ```yaml
   metadata:
     template_version: "X.Y.Z"  # Increment appropriately
     last_updated: "YYYY-MM-DD"
   ```

3. **Run release automation**:

   ```bash
   cargo xtask release-prepare X.Y.Z
   ```

4. **Validate everything**:

   ```bash
   cargo xtask docs-check
   cargo xtask selftest
   ```

5. **Generate release evidence**:

   ```bash
   cargo xtask release-bundle X.Y.Z
   ```

6. **Tag the kernel**:

   ```bash
   git tag -s "vX.Y.Z-kernel" -m "Kernel release vX.Y.Z"
   git push origin main --tags
   ```

### 9.3 Version increment rules

| Change Type | Example | Version Bump |
|-------------|---------|--------------|
| **Patch** (3.3.8 → 3.3.9) | Bug fixes, doc clarifications, no API changes | Z |
| **Minor** (3.3.8 → 3.4.0) | New ACs, new endpoints, backward-compatible | Y |
| **Major** (3.3.8 → 4.0.0) | Breaking changes, removed/renamed endpoints | X |

### 9.4 CODEOWNERS protection

Kernel-critical files are protected by `CODEOWNERS`. Changes to these files require review from kernel maintainers:

- `specs/spec_ledger.yaml`
- `specs/openapi/**`
- `docs/KERNEL_SNAPSHOT.md`, `docs/IDP_CELL_CONTRACT.md`
- `crates/xtask/**`
- `crates/spec-runtime/**`
- `.github/workflows/tier1-selftest.yml`

See [`CODEOWNERS`](./CODEOWNERS) for the full list.

### 9.5 More details

- **Detailed maintainer guide:** [`docs/how-to/maintain-kernel.md`](docs/how-to/maintain-kernel.md)
- **Kernel evolution playbook:** [`docs/how-to/evolve-the-kernel.md`](docs/how-to/evolve-the-kernel.md)
- **Current kernel snapshot:** [`docs/KERNEL_SNAPSHOT.md`](docs/KERNEL_SNAPSHOT.md)

---

## 10. Questions / Discussions

If you're unsure whether a change fits:

- Open a draft PR and describe:

  - the problem,
  - the AC or use case,
  - your proposed approach.
- Or open an issue referencing relevant ACs / features / policies.

Thanks again for contributing. This template is expressly meant to be **used** and **evolved** by people who care about governed, AI-assisted Rust services.
