# Effortless Metrics strict Clippy policy

This workspace treats Clippy as a governed engineering surface, not as a local taste file. The shared policy is intentionally uniform across Rust repositories: MSRV 1.93, panic-free production and test code, silent-failure prevention, suppression governance, AST/string/index safety, and an upgrade ledger for Rust 1.94 and 1.95 lints.

## Active workspace baseline

The active lint baseline lives in the root `Cargo.toml` under `[workspace.lints.rust]` and `[workspace.lints.clippy]`. Every workspace member must inherit it with:

```toml
[lints]
workspace = true
```

The machine-readable source of truth is `policy/clippy-lints.toml`. `cargo xtask check-lint-policy` verifies that active ledger entries are present in the root manifest at the same level.

## No test carveouts

This policy is workspace panic-free, not only production panic-free. Do not add Clippy test carveouts such as:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
```

Tests should return `Result` and use fallible helpers instead of `unwrap`, `expect`, or panic-driven setup.

## Suppression style

Use narrow `#[expect(..., reason = "...")]` suppressions when a local exception is justified. Silent `#[allow(...)]` suppressions are rejected by policy unless they are explicitly modeled as temporary debt.

A reviewed suppression should explain why the exception is safe and why a cleaner shape is not yet available:

```rust
#[expect(clippy::indexing_slicing, reason = "Generated table bounds are validated by build-time generator checks.")]
fn generated_lookup(table: &[u8], index: usize) -> u8 {
    table[index]
}
```

## Debt ledger

Temporary exceptions belong in `policy/clippy-debt.toml`. Every debt entry must have:

- `lint`
- `path`
- `owner`
- `reason`
- `expires`

Expired debt fails the lint policy gate. Debt is allowed; silent debt is not.

## Planned Rust 1.94 / 1.95 flips

The workspace tracks planned Clippy flips before the MSRV bump. `policy/clippy-lints.toml` records planned Rust 1.94 and 1.95 lints with their target level and activation MSRV. The gate rejects activating these lints early in the root manifest before the workspace MSRV reaches the planned version.

## Allowlist policy model

Structured policy allowlists live under `policy/`:

- `policy/no-panic-allowlist.toml` uses semantic identity: `path + family + selector`, with `last_seen` as an advisory locator only.
- `policy/non-rust-allowlist.toml` records intentional non-Rust files with owner, kind, reason, surface, classification, coverage commands, and optional expiry.

The operating rule is: global deny by default, local exception by structured receipt.

## Standard verification commands

Run these before opening a PR that changes lint, suppression, or policy shape:

```bash
cargo xtask check-lint-policy
cargo xtask check-file-policy
cargo xtask check-no-panic-family
cargo xtask policy-report
```

The standard full loop is:

```bash
cargo fmt
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
cargo xtask check-lint-policy
```
