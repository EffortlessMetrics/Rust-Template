# Reference: Coverage Workflow

Documents how the `Coverage` workflow works, where the coverage floor is
configured, and how to change it safely.

- tooling: `cargo llvm-cov` (via Nix)
- enforcement: `jq` on the JSON output

## Default threshold

By default, this template enforces a **60% line coverage floor** for the
workspace. The exact threshold is configured in
`.github/workflows/ci-coverage.yml` via an environment variable or inline
constant. Teams may lower this during brownfield adoption and raise it
over time as tests improve.
