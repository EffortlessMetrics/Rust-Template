# Effortless Metrics Clippy Policy

This workspace treats Clippy as a governed engineering surface, not a local taste
file. The root `Cargo.toml` contains the active lint block, `policy/clippy-lints.toml`
is the machine-readable ledger, and `cargo xtask check-lint-policy` verifies that
the two stay coherent.

## Operating standard

- **MSRV:** Rust `1.93` for the Rust workspace platform.
- **Panic posture:** production code and tests are panic-free by default.
- **Silent failure posture:** ignored futures, ignored `must_use` values, discarded
  `Result` state, and lossy line iteration are denied.
- **AST/string/indexing posture:** parser-sensitive string and slice operations are
  denied unless isolated behind reviewed abstractions.
- **Suppression posture:** prefer `#[expect(..., reason = "...")]`; do not add broad
  `#[allow]` attributes or test carveouts.

## No test carveouts

Do not add these `clippy.toml` switches:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
```

Tests should return `Result` and use helper assertions that preserve error context.

## Debt and exceptions

Temporary lint exceptions belong in `policy/clippy-debt.toml` with a lint, path,
owner, reason, and expiry. Panic-family exceptions belong in
`policy/no-panic-allowlist.toml` using semantic identity: `path + family + selector`.
Non-Rust programming/config surfaces belong in `policy/non-rust-allowlist.toml` with
an owner, reason, surface, classification, and CI coverage.

## Upgrade ledger

`policy/clippy-lints.toml` tracks planned Rust `1.94` and `1.95` flips before the
MSRV bump. Planned lints must not be activated in `Cargo.toml` until the workspace
MSRV reaches their `activate_when_msrv` value.

## Local check

Run:

```sh
cargo +1.93.0 xtask check-lint-policy
```

The check verifies MSRV alignment, active lint consistency, planned lint staging,
workspace lint inheritance, absence of `clippy.toml` test carveouts, and required
fields for policy/debt ledgers.
