# Testing Strategy

**Version**: v3.3.6
**Last Updated**: 2025-11-30

This repo ships with a layered test strategy:

- **Core checks** via `xtask selftest`
- **Unit / integration tests** via `cargo test`
- **BDD** via Cucumber
- **Policy tests** via `conftest`
- **LLM bundle checks** via `xtask bundle`

The goal is to make "did I break anything?" a single command in the happy path, while still allowing deeper, targeted runs when needed.

---

## 1. Selftest: the primary gate

The main entrypoint is:

```bash
# Preferred: inside nix devshell
nix develop
cargo run -p xtask -- selftest
```

This runs **eight steps**:

1. **Core checks** – `fmt`, `clippy -D warnings`, `cargo test --workspace`
2. **BDD** – Cucumber scenarios under `specs/features/`
3. **AC/ADR mapping** – Maps ACs in `specs/spec_ledger.yaml` to scenarios & tests, validates ADR references
4. **LLM bundler** – Validates `.llm/contextpack.yaml` and bundle generation
5. **Policy tests** – Runs Rego policies via `conftest` (K8s, flags, privacy, LLM, etc.)
6. **DevEx contract** – Validates required commands exist per `specs/devex_flows.yaml`
7. **Graph invariants** – Checks governance graph structural integrity
8. **AC coverage** – Validates kernel ACs (must_have_ac=true) are green

### Platform-Specific Behavior

**Tier-1 platforms** (Linux/macOS/WSL2 with Nix):
* All 8 steps must pass with strict gates
* Exact CI parity, canonical validation

**Tier-2 platforms** (Windows native):
* Steps 1-7 are strictly enforced
* Step 8 may intermittently fail with `os error 5` during `cargo rebuild` due to Windows file locking
* This is a platform limitation, NOT a test failure (see Section 7 for details)

In CI:
* Linux and macOS runners use Tier-1 validation (all 8 steps block merge)
* Windows runner is informational only

Locally:
* If `conftest` is **missing**, step 5 prints a warning and the selftest still passes (on non-CI environments)
* If you run inside `nix develop`, `conftest` is available and policies run exactly as in CI

---

## 2. Unit & Integration tests

Standard Rust tests:

```bash
# All tests
cargo test --workspace

# Single crate
cargo test -p app-http

# Single test file
cargo test -p rust_iac_config --test integration_tests

# Show stdout from a specific test
cargo test -p rust_iac_config test_manifests_directory_not_found -- --nocapture
```

### Ignored tests

Some tests are marked `#[ignore]` because they:

* Manipulate global process state (`set_current_dir`), or
* Depend on external services (e.g., Docker / Postgres).

To run them:

```bash
cargo test -p rust_iac_config --test integration_tests -- --ignored
```

Each ignored test has a comment explaining:

* What it covers
* Why it's ignored
* How to run it explicitly

---

## 2.4. How to Read AC Status

> **How to read AC status**
>
> - `[PASS]` = at least one test (BDD or unit) ran and passed.
> - `[FAIL]` = at least one test ran and failed.
> - `[UNKNOWN]` = no local test ran. In this repo, `[UNKNOWN]` is only used
>   for meta / CI-only ACs (see `docs/feature_status_notes.md` for details).

When you run `cargo xtask ac-coverage`, you'll see output like:

```
Feature: Platform Status
  ✅ AC-PLT-001: Status endpoint returns JSON
  ❌ AC-PLT-002: Graph endpoint includes all nodes
  ⚠️  AC-PLT-003: Cached status updates [UNKNOWN]
```

**What this means:**

- ✅ = AC has tests wired and they pass
- ❌ = AC has tests but they're failing (fix needed)
- ⚠️ = AC has no local tests (typical for meta/CI-only ACs)

**Important:** All kernel ACs (marked `must_have_ac: true` in `spec_ledger.yaml`) **must** show ✅ before work is considered complete.

---

## 2.5. Debugging Test Failures

When selftest fails, use this quick reference to debug each step:

| Step | What It Tests | How to Debug | Common Failures |
|------|---------------|--------------|-----------------|
| **1. Core checks** | fmt, clippy, tests | `cargo run -p xtask -- check` | Formatting issues, clippy warnings, unit test failures |
| **2. BDD** | Cucumber scenarios | `cargo run -p xtask -- bdd` | Scenario failures, missing step implementations |
| **3. AC/ADR mapping** | Spec traceability | `cargo run -p xtask -- adr-check` | Missing ADR files, invalid references in specs |
| **4. LLM bundler** | Context generation | `cargo run -p xtask -- bundle implement_ac` | Missing files, bundle size limits, invalid contextpack.yaml |
| **5. Policy tests** | Rego policies | `cargo run -p xtask -- policy-test` | conftest not installed, policy violations |
| **6. DevEx contract** | Required commands | Check `specs/devex_flows.yaml` | Missing xtask commands |
| **7. Graph invariants** | Governance structure | `cargo run -p xtask -- graph-export` | Structural violations in spec graph |
| **8. AC coverage** | Kernel AC status | `cargo run -p xtask -- ac-coverage` | Failing kernel ACs, missing BDD scenarios |

### Real Failures vs. Infrastructure Issues

**Real failures (Steps 1-7):**
* Indicate actual problems with code, specs, or governance
* Must be fixed before merging
* Block CI on all platforms

**Step 8 on Windows native (Tier-2):**
* May show `error: failed to remove xtask.exe: os error 5`
* This is **Windows file locking**, not a code failure
* Caused by antivirus or system processes holding the executable during rebuild
* Does NOT indicate test problems
* See Section 7 for platform-specific details and workarounds

---

## 3. BDD (Cucumber)

BDD scenarios live in `specs/features/*.feature`.

To run only BDD:

```bash
cargo run -p xtask -- bdd
```

* Output is JUnit XML (`target/junit/acceptance.xml`) plus human-readable summaries.
* Scenarios are tagged with `@AC-XXX` to keep spec ↔ test mapping real.

When you add or change behavior:

1. Update or add an AC in `specs/spec_ledger.yaml`.
2. Add/update a scenario in `specs/features/*.feature`.
3. Implement the change in Rust.
4. Run `xtask bdd` (or full `selftest`).

### The @ci-only Tag

Some BDD scenarios are tagged `@ci-only` to exclude them from local development runs. This is used for:

* **Recursive scenarios** - Tests that run `selftest` from within selftest
* **Git worktree scenarios** - Tests that create temporary worktrees (can flake with VS Code Git extension or other tools accessing `.git`)
* **Heavy integration tests** - Tests that spawn processes or access external resources

**How it works:**

When running locally (not in CI), `cargo xtask bdd` automatically sets:
```
CUCUMBER_TAG_EXPRESSION="not @ci-only"
```

This excludes @ci-only scenarios from local runs while CI still executes them.

**When to use @ci-only:**

Tag a scenario `@ci-only` when:
* It runs `cargo xtask selftest` from within BDD (recursive execution)
* It creates git worktrees or modifies `.git` state
* It depends on CI-specific environment (clean checkout, no VS Code)
* It's inherently slow or resource-intensive

**Important:** Always ensure the AC has unit test coverage or stable BDD scenarios for local validation. @ci-only should supplement, not replace, local testing.

**Override:** Set `CUCUMBER_TAG_EXPRESSION` explicitly to run all scenarios:
```bash
CUCUMBER_TAG_EXPRESSION="" cargo xtask bdd
```

See `docs/feature_status_notes.md` for the full list of @ci-only scenarios and the rationale.

---

## 4. Policy tests

Policies live under `policy/` and are tested via `conftest`.

**In CI:**

* Workflows under `.github/workflows/ci-*-policy.yml` run policy tests in a Nix devshell.
* conftest is pinned via the flake (currently `0.52.0`).

**Locally, simplest path:**

```bash
nix develop
cargo run -p xtask -- selftest
```

If you want to run only policies:

```bash
nix develop
cargo run -p xtask -- policy-test
# or directly
conftest test -p policy/k8s.rego policy/testdata/k8s_valid.yaml
```

For native (non-Nix) setup, see `CLAUDE.md` and `docs/dev-environment.md` for conftest installation steps.

---

## 5. LLM / bundle checks

Bundles are generated via:

```bash
cargo run -p xtask -- bundle implement_ac
```

This is validated in selftest:

* `.llm/contextpack.yaml` structure (policy/llm.rego)
* Files included
* Bundle size limits

If bundle generation fails, selftest fails and will point you at the specific issue (missing file, misconfigured include, etc.).

---

## 6. Test profiles in practice

Typical flows:

**Before every commit:**

```bash
nix develop
cargo run -p xtask -- check
```

**Before pushing / opening a PR:**

```bash
nix develop
cargo run -p xtask -- selftest
```

**Working on policies:**

```bash
nix develop
cargo run -p xtask -- policy-test
```

**Working on a specific crate:**

```bash
nix develop
cargo test -p business-core
```

---

## 7. Platform Support & Test Expectations

This template supports development across multiple platforms with different validation guarantees for `cargo xtask selftest`.

### Tier-1 Platforms (Fully Validated - Recommended)

**Platforms:**
* Linux with Nix devshell
* macOS with Nix devshell
* WSL2 on Windows with Nix devshell

**Test expectations:**
* All 8 selftest steps must pass with strict gates
* Exact CI parity - if it passes locally, it passes in CI
* Canonical validation environment

**Getting started:**
```bash
# Install Nix (one-time setup)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install

# Enter devshell
nix develop

# Run selftest
cargo run -p xtask -- selftest
```

**Why Tier-1 is canonical:**
* Nix devshells ensure environment reproducibility matching CI exactly
* No local drift between dev and CI
* All tools (conftest, etc.) pinned to exact versions
* All 8 steps enforce strict governance

---

### Tier-2 Platforms (Supported with Known Caveats)

**Platforms:**
* Windows 10/11 with native PowerShell or Git Bash
* Requires manual installation: Rust 1.91+, conftest, Docker Desktop

**Test expectations:**
* Steps 1-7 are strictly enforced (same as Tier-1)
* Step 8 (AC coverage) may **intermittently fail** with platform limitations

**The "os error 5" Issue:**

On Windows native, you may see this during step 8:
```
error: failed to remove C:\...\target\debug\xtask.exe: os error 5 (Access is denied)
```

**What this means:**
* This is **Windows file locking**, NOT a test failure
* The AC coverage tests themselves are passing
* The issue occurs during `cargo rebuild` when checking AC status
* Windows holds executable locks longer than Unix-like systems

**Why it happens:**
* Antivirus real-time scanning of `target/` directory
* Windows Defender file indexing
* Background system processes accessing the executable
* Stricter file locking compared to Unix

**This is NOT a test failure because:**
* The actual tests (steps 1-7) completed successfully
* AC status validation completed successfully
* The error occurs in infrastructure (cargo rebuild), not test logic
* Code quality and governance are verified

**Workarounds:**

1. **Exclude from antivirus** (recommended for daily dev):
   ```powershell
   # Add target/ to Windows Defender exclusions
   Add-MpPreference -ExclusionPath "C:\Code\Rust\Rust-Template\target"
   ```

2. **Use WSL2 for final validation** (recommended for PRs):
   ```bash
   # In WSL2 with Nix
   nix develop
   cargo run -p xtask -- selftest  # All 8 steps will pass reliably
   ```

3. **Wait and retry**:
   * Close other programs accessing the repository
   * Wait a few seconds and run again
   * Usually succeeds on second attempt

---

### When to Use Each Tier

| Scenario | Recommended Platform |
|----------|---------------------|
| **First setup** | WSL2 + Nix (Tier-1) for exact CI match |
| **Daily development on Windows** | Native Windows (Tier-2) with antivirus exclusions |
| **Before opening PR** | WSL2 + Nix (Tier-1) for canonical validation |
| **Debugging test failures** | Use Tier-1 to eliminate platform variables |
| **CI/CD** | Always Tier-1 (Linux/macOS with Nix) |

---

### CI Guarantees

**What blocks merge:**
* Linux CI runner (Tier-1): All 8 steps must pass
* macOS CI runner (Tier-1): All 8 steps must pass

**What is informational:**
* Windows CI runner (Tier-2): Runs but does not block merge
* Used to detect Windows-specific issues early
* Known file locking issues do not fail the build

**Validation contract:**
* If selftest passes on Tier-1 locally, it WILL pass in CI
* Tier-2 is suitable for development, Tier-1 required for final validation
* All merged code has passed strict 8-step validation on Tier-1

---

### Platform Setup Details

For complete platform setup instructions, troubleshooting, and known issues:
* **Tier-1 (recommended):** See `README.md` → "Quick Start" → Nix devshell section
* **Tier-2 (Windows native):** See `docs/MISSING_MANUAL.md` → "Platform Support" → "Tier 2: Native Windows"
* **General dev environment:** See `docs/dev-environment.md`

Having this doc gives future-you (and other contributors) one place to understand "how testing works here."

You can then link it from `docs/README.md` under "For Development".
