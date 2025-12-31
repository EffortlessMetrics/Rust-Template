# How-to: Write BDD Tests

This guide explains how to write Behavior-Driven Development (BDD) tests using Gherkin feature files in this template.

**What you'll learn:**

- How to structure `.feature` files in `specs/features/`
- How to tag scenarios with `@AC-xxx` for AC coverage
- How to run and debug BDD tests
- Best practices for scenario design

**Prerequisites:**

- Development environment set up (`cargo xtask dev-up`)
- Basic understanding of Gherkin syntax
- Familiarity with the spec ledger (see `docs/explanation/TEMPLATE-CONTRACTS.md`)

---

## Overview

BDD tests in this template use Gherkin `.feature` files located in `specs/features/`. Each scenario should be tagged with an `@AC-xxx` tag linking it to an Acceptance Criterion in `specs/spec_ledger.yaml`.

```gherkin
@AC-TPL-EXAMPLE-001
Scenario: Example scenario linked to AC
  Given the system is running
  When I perform an action
  Then the expected result occurs
```

## Running BDD Tests

```bash
# Run all BDD tests
cargo xtask bdd

# Run BDD tests for a specific AC
cargo xtask test-ac AC-TPL-EXAMPLE-001

# Check AC coverage
cargo xtask ac-status
```

## Best Practices

1. **One AC per scenario** - Keep scenarios focused on a single acceptance criterion
2. **Use descriptive Given/When/Then** - Make steps self-documenting
3. **Tag appropriately** - Use `@AC-xxx` for AC coverage, `@wip` for work in progress
4. **Avoid implementation details** - Test behavior, not implementation

## See Also

- [How-to: Add an Acceptance Criterion](./add-acceptance-criterion.md)
- [Template Contracts](../explanation/TEMPLATE-CONTRACTS.md)
- [Selective Testing](../SELECTIVE_TESTING.md)
