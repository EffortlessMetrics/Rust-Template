Implement the Effortless Metrics strict Clippy policy for this Rust workspace
without changing product behavior. Add the workspace-level active lint baseline
for panic-free production and test code, silent-failure prevention, suppression
governance, AST/string/indexing safety, and good-taste reviewability lints. Add
`clippy.toml` only for repo-specific disallowed methods/types/macros/names, not
test carveouts. Add `policy/clippy-lints.toml`, `policy/clippy-debt.toml`,
`policy/no-panic-allowlist.toml`, `policy/non-rust-allowlist.toml`, and
`docs/CLIPPY_POLICY.md`. Wire `cargo xtask check-lint-policy` so CI can verify
lint inheritance, active/planned lint consistency, panic-free test posture, and
planned upgrade flips. Keep existing debt explicit, counted, and temporary.
