# Codecov coverage

Codecov coverage is Rust test execution-surface evidence.

It answers:

> Did tests execute this Rust surface?

It does not answer:

- whether acceptance criteria coverage is complete,
- whether kernel contracts are correct,
- whether the governance graph is complete,
- whether BDD scenarios are adequate,
- whether policy checks are complete,
- whether `/platform/coverage` is correct,
- whether IDP readiness is proven,
- whether release readiness is proven.

Those are separate proof lanes.

## Coverage workflow

The Coverage workflow runs on:

- push to `main`,
- `workflow_dispatch`,
- PRs labeled `coverage` or `full-ci`.

Codecov comments are disabled. Durable receipts are:

- `coverage.json`,
- `coverage.txt`,
- `lcov.info`,
- the GitHub Actions coverage artifact,
- the Codecov dashboard.

## Governed state

For governed state, continue to rely on:

- `cargo xtask selftest`,
- `cargo xtask ac-status`,
- `cargo xtask ac-coverage`,
- `docs/feature_status.md`,
- `/platform/coverage`.
