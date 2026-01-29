# testing – CLAUDE.md

**Tier:** Foundation (Layer 1)
**Publish:** No (internal, dev-dependency)
**Dependencies:** Minimal

## Purpose

Shared testing utilities for the workspace. Provides safe environment manipulation, process state isolation, and test helpers.

## Key Exports

- Safe environment variable manipulation (thread-safe)
- Process state isolation for tests
- Common test fixtures and helpers
- Test result assertions

## When to Modify

- Adding new test utilities needed by multiple crates
- Improving test isolation patterns

## When NOT to Modify

- Adding production code (this is test-only)
- Adding crate-specific test helpers (keep those local)

## Architectural Notes

- **Dev-dependency only**: Never used in production builds
- **Thread safety**: Environment manipulation is safe for parallel tests
- **Workspace-wide**: Shared across all crates' tests

## Consumers

All crates' test modules, `acceptance`

## See Also

- `crates/acceptance/` for BDD acceptance tests
- Each crate's `tests/` directory for integration tests
