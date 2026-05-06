# Effortless Metrics Clippy Policy

This workspace treats Clippy as a governed engineering surface, not a local taste
file. The standard is one platform-wide baseline: MSRV 1.93, panic-free
production and test code, AST/string/indexing-safe defaults, explicit
suppression receipts, and planned Rust 1.94/1.95 lint flips tracked before the
compiler bump.

## Active baseline

The active lint baseline lives in the root `Cargo.toml` under
`[workspace.lints.rust]` and `[workspace.lints.clippy]`. Every workspace member
must inherit it with:

```toml
[lints]
workspace = true
```

The baseline covers:

- panic-free code in production and tests (`panic`, `unwrap_used`,
  `expect_used`, `todo`, `unimplemented`, `unreachable`, and related lints);
- AST/parser/string/indexing safety (`string_slice`, `indexing_slicing`,
  `char_indices_as_byte_indices`, and related lints);
- silent-failure prevention (`let_underscore_*`, `unused_result_ok`,
  `map_err_ignore`, `lines_filter_map_ok`);
- async/concurrency footguns;
- unsafe/memory footguns;
- numeric correctness and arithmetic reviewability;
- filesystem/process/path hazards;
- API and trait correctness; and
- good-taste reviewability lints that reduce allocation, formatting, and
  control-flow noise.

## No test carveouts

Tests inherit the same panic-free policy as production code. Do not add Clippy
configuration such as `allow-unwrap-in-tests`, `allow-expect-in-tests`,
`allow-panic-in-tests`, `allow-indexing-slicing-in-tests`, or
`allow-dbg-in-tests`.

Prefer tests that return `Result` and propagate setup or assertion failures with
reviewable context:

```rust
#[test]
fn parses_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = std::fs::read_to_string("tests/fixtures/input.rs")?;
    let parsed = parse(&fixture)?;

    ensure_eq(parsed.items.len(), 3, "fixture should expose three items")?;

    Ok(())
}
```

## Suppression style

Suppressions must be narrow and intentional. Use `#[expect(..., reason =
"...")]` instead of `#[allow(...)]` for local suppressions. If a suppression is
not expected to be permanent, add a matching `policy/clippy-debt.toml` entry with
`lint`, `path`, `owner`, `reason`, and `expires`.

Broad suppression configuration and blanket category enables/weakens are not
allowed. `clippy.toml` exists only for repository-local disallowed methods,
types, macros, names, or similar architecture additions; it must not weaken the
shared baseline.

## Machine-readable policy files

- `policy/clippy-lints.toml` is the source-of-truth ledger for active lints,
  policy posture, and planned Rust 1.94/1.95 flips.
- `policy/clippy-debt.toml` tracks temporary lint exceptions with owners,
  reasons, paths, lints, and expiry dates.
- `policy/no-panic-allowlist.toml` reserves the semantic TOML receipt shape for
  rare panic-family exceptions using `path + family + selector` identity and
  advisory `last_seen` location.
- `policy/non-rust-allowlist.toml` documents non-Rust surfaces with owner,
  reason, surface, classification, and CI coverage.

## Upgrade tracking

The ledger tracks planned Rust 1.94 lints (`same_length_and_capacity`,
`manual_ilog2`, `decimal_bitwise_operands`, and `needless_type_cast`) and Rust
1.95 lints (`disallowed_fields`, `manual_checked_ops`, `manual_take`,
`manual_pop_if`, `duration_suboptimal_units`, and
`unnecessary_trailing_comma`). Planned entries must remain out of the active
Cargo lint block until the workspace MSRV reaches their activation version.

## Xtask gate

Run the policy gate with:

```bash
cargo xtask check-lint-policy
```

The gate verifies MSRV alignment, workspace lint inheritance, active/planned lint
consistency, no test carveouts, no silent `#[allow]` suppressions, reasoned
`#[expect]` suppressions, and well-formed non-expired debt entries.
