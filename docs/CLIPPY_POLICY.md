# Effortless Metrics Clippy Policy

This workspace treats Clippy as a governed engineering surface, not as an
informal style preference. The root `Cargo.toml` owns the active lint block,
`policy/clippy-lints.toml` records the machine-readable source of truth, and
`cargo xtask check-lint-policy` verifies that manifests, debt, planned flips,
and suppression posture stay coherent.

## Goals

The common Rust platform baseline is:

- MSRV `1.93` for Rust workspaces.
- Panic-free production **and tests**.
- AST, UTF-8, string, and indexing safety by default.
- Silent-failure prevention for ignored futures, ignored `Result`s, and ignored
  lock guards.
- Explicit suppression governance: use `#[expect(..., reason = "...")]`, not
  broad `#[allow]` blocks.
- Planned Rust `1.94` and `1.95` lint flips tracked before the MSRV bump.

## Active lint surface

The active baseline lives in `[workspace.lints.rust]` and
`[workspace.lints.clippy]` in the root manifest. Workspace member crates must
inherit it with:

```toml
[lints]
workspace = true
```

The ledger in `policy/clippy-lints.toml` mirrors every active lint with a level,
status, class, and reason. It also records planned Rust `1.94` and `1.95` flips
so upgrades are reviewed as policy changes instead of surprise compiler churn.

## No test carveouts

Do not add any of these Clippy configuration carveouts:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
```

Tests should return `Result` and use domain assertions that report structured
errors rather than relying on `unwrap`, `expect`, or panic-driven setup.

## Suppression style

Prefer fixing the lint. When a local exception is genuinely needed, use a narrow
`#[expect]` with a reason:

```rust
#[expect(clippy::indexing_slicing, reason = "Generated lookup table; covered by parser fixture tests.")]
fn generated_table_lookup(table: &[u8], index: usize) -> u8 {
    table[index]
}
```

Broad `#[allow]` usage is intentionally blocked by policy checks unless a future
explicit allowlist records the exception.

## Debt ledger

Temporary weakening belongs in `policy/clippy-debt.toml`. Each debt entry must
have a lint, path, owner, reason, and expiry date. Expired debt fails the policy
gate.

## Allowlist model

The template also standardizes policy receipts for exceptions outside Clippy:

- `policy/no-panic-allowlist.toml` uses semantic panic-family identities:
  `path + family + selector`; `last_seen` is advisory only.
- `policy/non-rust-allowlist.toml` records why non-Rust programming or policy
  files exist, who owns them, what surface they affect, and what CI covers them.

This keeps the global default strict while making every local exception
structured, reviewable, and temporary.

## Required check

Run:

```bash
cargo xtask check-lint-policy
```

The gate verifies MSRV alignment, workspace lint inheritance, active/planned lint
ledger consistency, absence of Clippy test carveouts, suppression posture, and
non-expired debt metadata.
