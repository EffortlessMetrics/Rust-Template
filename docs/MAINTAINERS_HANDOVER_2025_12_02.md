<!-- doclint:disable orphan-version -->
# Maintainers' Handover – Kernel v3.3.6 + December 2 Improvements

## 0. TL;DR for the next person

You're picking up a **large but coherent change set** on top of the v3.3.6 kernel:

* Local state is **selftest-green** inside Nix after environment fixes.
* Git state: ~9 files changed vs `origin/main` in code, plus a set of new/updated docs (47 paths in `git status` including untracked/aux docs).
* There is a draft **PR slicing plan** (`PR_ORGANIZATION_PLAN.md`) that splits this into 5 reviewable PRs.
* A new **Nix/devshell fix** (zlib) resolves the previous `libz.so.1` / sccache failure.
* Two new docs harden the story for Nix/env issues:

  * `docs/how-to/nix-environment-issues.md`
  * updated `docs/TROUBLESHOOTING.md` "libz / sccache / Nix" section.

CI on GitHub is still red for reasons that are **global to the repo**:

* Missing `cargo llvm-cov`, `protoc`.
* Outdated `cargo-audit`/`cargo-deny` vs `Cargo.lock` v4 / Edition 2024.
* Gitleaks license, CodeQL permissions, macOS Nix installer, artifact upload name collisions.

Those are **not** introduced by these changes.

Your job, if you want to take this over the line:

1. Understand the current branch and environment.
2. Use the PR plan to break work into mergeable slices.
3. Treat global CI failures as separate "CI cleanup" work, not blockers for the doc/devex changes.

The rest of this document gives you the map.

---

## 1. Current repo / branch / environment state

### 1.1. Git snapshot

From the snapshot:

* `git diff origin/main...HEAD --stat` → **9 files changed, 216 insertions, 319 deletions** in code/docs.
* `git status --short` → 47 entries including:

  * `M .claude/agents/example-agent.md`
  * `A COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md`
  * `MM Cargo.lock`
  * `M flake.nix`, `M flake.lock`
  * New/modified how-to docs (`docs/how-to/*`), devex flows, feature files.

There is a **big narrative doc**:

* `COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md` (~547 lines)
  This is effectively a *changelog + rationale* for the December improvement push: docs governance, IDP adoption, CI behavior, fork stories, etc. It's not required for runtime, but it's very useful context for reviewers.

There is a **PR planning doc**:

* `PR_ORGANIZATION_PLAN.md` (~236 lines) – describes how to slice this change set into 5 specific PRs, ordered and scoped.

### 1.2. Version & kernel state

* `specs/spec_ledger.yaml`:

  ```yaml
  metadata:
    schema_version: "1.0"
    template_version: "3.3.6"
    ...
  ```

* Kernel tag `v3.3.6-kernel` exists on `main`.

Local validation (after environment fixes):

```bash
nix develop -c cargo xtask check
nix develop -c cargo xtask selftest
```

Both pass locally.

---

## 2. Nix/devshell / environment: what was broken and what's fixed

### 2.1. The original problem

Previously, in CI and sometimes locally, `rustc`/`sccache` failed with:

* `error: could not execute process 'sccache rustc -vV' (exit status: 127)`
* Or runtime errors around missing `libz.so.1`.

Diagnosis:

* In the Nix `devShell`, `pkgs.zlib` was **not included**, and `LD_LIBRARY_PATH` wasn't exporting the zlib library path.
* When `RUSTC_WRAPPER=sccache` is used via `nix develop -c`, the wrapper's dynamic deps weren't found.

This was documented (incompletely) in `docs/TROUBLESHOOTING.md` as "libz / sccache issue", but the story was too WSL-specific and implied this was a Windows-only quirk.

### 2.2. The fix in `flake.nix`

`flake.nix` now:

* Adds `pkgs.zlib` to the devshell packages.
* Adds `buildInputs = [ pkgs.zlib ];`
* Exports `LD_LIBRARY_PATH` to include the zlib library path.

Concrete changes (paraphrased):

```nix
devShells = forAllSystems ({ pkgs, rust, ... }: {
  default = pkgs.mkShell {
    packages = [
      rust
      pkgs.cargo-audit
      pkgs.cargo-deny
      pkgs.cargo-nextest
      pkgs.zlib  # Required for rustc/sccache dynamic linking
    ];

    buildInputs = [ pkgs.zlib ];

    shellHook = ''
      export PATH="$PWD/.tools/bin:$PATH"
      export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.zlib ]}:$LD_LIBRARY_PATH"
      echo "DevShell ready — try: just checks"
    '';
  };
});
```

Result:

* `nix develop -c cargo xtask check` now runs without `libz` failures.
* `nix develop -c cargo xtask selftest` completes successfully.

### 2.3. Docs added/updated to reflect this

Two key docs now encode this knowledge:

1. `docs/how-to/nix-environment-issues.md` (new):

   * Long-form guide under id `GUIDE-TPL-NIX-ENV-ISSUES`.
   * Explains:

     * How the devshell is composed (flake, pkgs, shellHook).
     * How `RUSTC_WRAPPER` + dynamic libs can break.
     * Symptoms (sccache/rustc failing, empty JUnit).
     * Permanent fix: include zlib (and similar) in devshell + LD path.
     * Temporary workarounds for constrained environments.

2. `docs/TROUBLESHOOTING.md` – updated "libz / sccache / Nix" section:

   * Reframes "libz" as a **Nix environment config issue**, not "WSL is cursed".
   * Shows exact `flake.nix` snippet to add `pkgs.zlib` and `LD_LIBRARY_PATH`.
   * Explains impact on AC status and feature_status generation.
   * Points at the new how-to doc for more detail.

As of now:

```bash
nix flake update
nix develop -c cargo xtask check
nix develop -c cargo xtask selftest
```

works and is the **recommended** validation path.

---

## 3. The PR slicing plan (PR_ORGANIZATION_PLAN.md)

`PR_ORGANIZATION_PLAN.md` is your roadmap for splitting this monolithic branch into 5 coherent PRs.

### 3.1. PR-1 – Environment & Nix flake fixes (FOUNDATION)

**Goal:** make `cargo xtask check` / `selftest` reliable inside Nix; document the environment story.

Likely contents:

* `flake.nix`, `flake.lock`:

  * Add `pkgs.zlib` to `packages` and `buildInputs`.
  * Export `LD_LIBRARY_PATH` with zlib path.
* `docs/TROUBLESHOOTING.md`:

  * Replace WSL-only framing with Nix/devshell framing.
  * Add explicit snippet of the fixed devshell.
* `docs/how-to/nix-environment-issues.md` (new):

  * Explain context, symptoms, fixes, and workarounds.

**Validation criteria:**

* `nix develop -c cargo xtask check` passes.
* `nix develop -c cargo xtask selftest` passes.
* `docs-check` passes (no orphan-version regressions, doc_index aligned).

### 3.2. PR-2 – Agents governance system (agents-lint / agents-fmt) – **mentioned in plan, not in detail here**

The PR plan indicates a second PR around:

* `.claude/agents/example-agent.md`
* Agents governance docs and/or commands (`agents-lint`, `agents-fmt`).

This handover doesn't have the full diff for that, but PR_ORGANIZATION_PLAN enumerates the files and intent. When you pick that up:

* Treat it as: "bring agents governance to parity with skills governance".
* Ensure:

  * `xtask agents-lint` exists and is wired into `check` / `selftest`.
  * Docs updated (AGENTS_GOVERNANCE.md, INDEX entries).

### 3.3. PR-3 – Fork customization & reconciliation guides

From `git diff --cached --stat`, we know there's a heavy doc:

* `docs/how-to/reconcile-kernel-updates.md` (~880 lines).
* Potentially related how-tos (FIRST_FORK, change-opinion, etc. per PR_ORGANIZATION_PLAN).

The intent here is:

* Give consumers a **repeatable playbook** for:

  * Taking the Rust-Template kernel (`v3.3.x`) and creating a service fork.
  * Reconciling future kernel updates (e.g. `v3.3.6` → `v3.4.0`) into those forks.

* That doc likely covers:

  * When you should merge vs cherry-pick vs rebase.
  * How to use `cargo xtask docs-check`, `selftest`, `ac-status` before/after reconciliation.
  * How to handle spec_ledger deltas, new ACs, and changed policies.

Your job when slicing this PR:

* Collect **only** docs + `specs/doc_index.yaml` changes relevant to fork/reconciliation.
* Ensure:

  * New docs have proper front-matter (id, doc_type, requirements, etc.).
  * doc_index entries exist and `docs-frontmatter-sync` passes.
  * `docs-check` + `selftest` remain green.

### 3.4. PR-4 – Performance & code quality (BDD & runtime)

Per the plan, this PR is about:

* BDD execution behavior (parallelization, skip conditions).
* Rust code quality / clippy nits where they're **not** structural.
* Possibly some acceptance/runtime tweaks from COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md.

Since we don't have that diff spelled out here, treat PR_ORGANIZATION_PLAN as canonical:

* Make this PR **code-only**, minimal doc modifications.
* Keep it safe: no semantic changes to external contracts without accompanying spec/docs updates.

### 3.5. PR-5 – Doctor enhancements & coverage/build tracking

The plan calls for:

* `xtask doctor` improvements (env diagnostics, toolchain checks, Nix vs non-Nix).
* Possibly additional coverage/reporting surfaces.

Again, treat this as a follow-up once PR-1...PR-4 are in.

---

## 4. What's **already** merged vs what's only local

From earlier work (already merged to `main`):

* **PR #8** – `feat(xtask): add env-mode command for debugging CI/env detection`
* **PR #7** – `feat(port-integration): add --dump-only flag for offline IDP testing`
* **PR #6** – `docs: demo docs-check catching version drift`

This handover's new local changes **build on top** of that:

* `flake.nix` / `flake.lock` updates: Nix + zlib.
* Nix environment doc (`docs/how-to/nix-environment-issues.md`).
* Troubleshooting doc updates (`docs/TROUBLESHOOTING.md`).
* Potentially large additional how-tos and spec/docs updates as called out in `PR_ORGANIZATION_PLAN.md` and `COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md`.

When you proceed:

* Don't rescope #6/7/8 – they're already in main.
* Focus on slicing the **new** work since that last known good point.

---

## 5. CI: what the red checks actually mean

When you look at GitHub Actions, you'll see a forest of red checks. Here's how to interpret them:

### 5.1. Structural / infra failures

These are **not caused** by the December changes:

* `coverage` job:

  * `error: no such command: 'llvm-cov'`
    → Add `cargo-llvm-cov` to Nix or disable for now.

* `docs` job:

  * `mkdocs build --strict -f backstage/mkdocs.yml` → `backstage/docs` missing.
    → Backstage docs not scaffolded yet.

* `deps` (cargo-audit / cargo-deny):

  * Cargo-audit doesn't support `Cargo.lock` v4.
  * cargo-deny doesn't support Edition 2024 in `Cargo.toml`.
  * These require **tooling upgrades** in the devShell or reconfigurations.

* `secrets` (gitleaks):

  * Requires `GITLEAKS_LICENSE` secret in org/repo; otherwise fails immediately.

* `Self-Test / selftest`, `rust-lints`, Tier jobs:

  * On CI, `nix develop -c cargo ...` is sometimes misconfigured (RUSTC_WRAPPER or path), or Nix install fails on macOS.
  * `adapters-grpc` compilation needs `protoc` available.

* `CodeQL`:

  * Completes analysis but fails writing status/telemetry due to GitHub App permission.

* Artifact upload 409 conflicts:

  * Reused artifact names across jobs in the same workflow run.

**Recommendation:** track these in a separate "CI Hardening" epic. Don't block the environment/docs/devex PRs on this.

### 5.2. Template code warnings

Clippy warnings in `app-http/src/platform/idp.rs` (if-collapsible, `map_or` simplifications, etc.) are still present and were **not** addressed in this slice. You can decide whether to:

* Tackle them in a dedicated "clippy cleanup" PR, or
* Keep treating them as warnings only.

---

## 6. Concrete next actions for the new maintainer

Here's how to take this over the line, step-by-step.

### 6.1. Step 1 – Confirm local baseline

On your machine:

```bash
git fetch origin
git status           # ensure you know which branch you're on
nix develop -c cargo xtask check
nix develop -c cargo xtask selftest
```

If both pass, you're starting from the same healthy state.

### 6.2. Step 2 – Review `COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md` & `PR_ORGANIZATION_PLAN.md`

* `COMPREHENSIVE_IMPROVEMENTS_2025_12_02.md` tells you:

  * What's been changed across the board (governance, IDP, docs, env).
  * The intent and narrative behind it.

* `PR_ORGANIZATION_PLAN.md` tells you:

  * How to slice that into 5 PRs.
  * Rough file lists and priorities per PR.

Treat these as your "design docs".

### 6.3. Step 3 – Implement PR-1 (Environment & Nix fixes)

On a fresh branch from `main`, e.g.:

```bash
git checkout main
git pull
git checkout -b pr1-env-nix-fixes
```

Apply only:

* `flake.nix`, `flake.lock` changes.
* `docs/TROUBLESHOOTING.md` Nix/libz section.
* `docs/how-to/nix-environment-issues.md` (plus doc_index entry, if not already added).

Then:

```bash
nix flake update       # if appropriate
nix develop -c cargo xtask check
nix develop -c cargo xtask selftest
git add flake.nix flake.lock docs/TROUBLESHOOTING.md docs/how-to/nix-environment-issues.md specs/doc_index.yaml
git commit -m "chore(env): stabilize Nix devshell (zlib) and document Nix issues"
git push -u origin pr1-env-nix-fixes
```

Open PR–1. In its description, explicitly call out that CI red jobs are inherited infra issues.

### 6.4. Step 4 – Implement PR-2 .. PR-5 as per the plan

Repeat the pattern:

* New branch off `main`.

* Cherry-pick only relevant files.

* Run:

  ```bash
  nix develop -c cargo xtask docs-check
  nix develop -c cargo xtask selftest
  ```

* Commit and push.

* Reference `PR_ORGANIZATION_PLAN.md` in PR descriptions.

If you want to be precise, you can copy the "Validation criteria" subsections from the plan into each PR's checklist.

### 6.5. Step 5 – CI epic (optional but recommended)

Create an issue or tracking doc for:

* `protoc` in CI (for `adapters-grpc`).
* `cargo-llvm-cov` integration.
* `cargo-audit` / `cargo-deny` upgrades.
* macOS Nix installer job flakiness.
* gitleaks license.
* CodeQL permissions.
* Artifact naming conflicts.

Then, you or someone else can tackle those independently of the template/kernel changes.

---

## 7. If you remember nothing else

* **Spec ledger is truth.** `specs/spec_ledger.yaml` governs template version and AC contracts; everything else (README, ROADMAP, KERNEL_SNAPSHOT, docs examples) must follow.

* **Local Nix env is now stable** thanks to zlib in devShell. Use `nix develop -c cargo xtask check/selftest`.

* The December change set is large but **already decomposed** conceptually; your job is mostly **mechanical slicing into PRs** and explicitly calling out that CI red is infra, not regression.

* The three PRs #6, #7, #8 are **already merged** and form the foundation for docs governance, env-mode, and IDP dump-only usage. This handover covers the **next layer**: environment hardening and multi-PR organization.

You can treat this document as the "you in 3 months" note: if you come back later, read this, read `PR_ORGANIZATION_PLAN.md`, run `nix develop -c cargo xtask selftest`, and you'll be back in the right mental model.
