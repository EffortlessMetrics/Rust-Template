# Testing Strategy

**Version**: v2.4.0
**Last Updated**: 2025-11-18

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

This runs five phases:

1. **Core checks** – `fmt`, `clippy -D warnings`, `cargo test --workspace`
2. **BDD** – Cucumber scenarios under `specs/features/`
3. **AC status** – Maps ACs in `specs/spec_ledger.yaml` to scenarios & tests
4. **LLM bundler** – Validates `.llm/contextpack.yaml` and bundle generation
5. **Policy tests** – Runs Rego policies via `conftest` (K8s, flags, privacy, LLM, etc.)

In CI, **all five** must pass.

Locally:

* If `conftest` is **missing**, phase 5 prints a warning and the selftest still passes.
* If you run inside `nix develop`, `conftest` is available and policies run exactly as in CI.

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

Having this doc gives future-you (and other contributors) one place to understand "how testing works here."

You can then link it from `docs/README.md` under "For Development".
