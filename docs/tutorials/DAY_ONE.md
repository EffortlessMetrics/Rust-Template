# Day One: Onboarding to Rust-as-Spec

Welcome to the IDP Platform Cell! This guide will get you from zero to your first Acceptance Criterion (AC) in under 30 minutes.

## 1. Fast Bootstrap

If you haven't already, run:

```bash
cargo xtask dev-up
```

This installs git hooks, checks your environment, and ensures local services are running.

## 2. Core Concepts

Read the [Glossary](../GLOSSARY.md) to understand terms like **Spec Ledger**, **BDD**, and **ADR**.

## 3. Your First Change (AC-First Workflow)

The "Golden Path" for adding features is:

1. **Define the AC**: Add a new entry to `specs/spec_ledger.yaml`. Use `cargo xtask ac-new` to scaffold it.
2. **Generate Context**: Run `cargo xtask bundle implement_ac` to create a context bundle for your agent or for yourself.
3. **Write Scenarios**: Add a `.feature` file in `specs/features/` with your AC tag (e.g., `@AC-MY-001`).
4. **Implement**: Write the code to satisfy the scenarios.
5. **Verify**: Run `cargo xtask selftest` to ensure everything is correct and governed.

## 4. Getting Help

- **Available Workflows**: `cargo xtask help-flows`
- **Command Documentation**: `cargo xtask <command> --help-docs`
- **Troubleshooting**: `docs/TROUBLESHOOTING.md`

Happy hacking!
