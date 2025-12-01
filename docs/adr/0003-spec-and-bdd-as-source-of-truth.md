<!-- doclint:disable orphan-version -->
<!-- ADR: This document contains historical version references as part of the decision record. -->
# ADR-0003: Spec Ledger and BDD as Source of Truth for Behavior

**Status**: Accepted
**Date**: 2025-01-18
**Authors**: Steven Zimmerman
**Related ACs**: AC-TPL-001, AC-TPL-002

---

## Context

Service behavior often becomes tribal knowledge:

- Features exist but aren't documented
- Acceptance criteria drift from implementation
- Tests pass but no one knows what they validate
- Product and engineering lose shared vocabulary

Traditional documentation approaches fail:

1. **JIRA/Linear tickets**: archived after sprint, forgotten
2. **Confluence/Wiki pages**: stale within weeks, never updated
3. **Code comments**: scattered, not validated, ignored in reviews
4. **Unit tests**: test implementation, not behavior contract

We need:

- Single source of truth for "what does this service do?"
- Traceability: user story → requirement → acceptance criterion → test → code
- Validation: ensure specs stay in sync with reality
- LLM-friendly: structured files agents can read and enforce

---

## Decision

We adopt **spec-as-code** via a YAML ledger + Gherkin BDD features:

### 1. Spec Ledger (`specs/spec_ledger.yaml`)

Canonical registry of all user stories, requirements, and acceptance criteria:

```yaml
metadata:
  schema_version: "1.0"
  template_version: "2.3.0"

stories:
  - id: US-TPL-001
    title: "Template Core Endpoints"
    status: implemented
    requirements:
      - id: REQ-TPL-001
        title: "Service provides health check"
        status: implemented
        acceptance_criteria:
          - id: AC-TPL-001
            title: "Health endpoint returns 200 OK"
            status: implemented
            feature_file: specs/features/template_core.feature
            scenario: "Health check returns 200 OK"
```

**Rules:**

- Every feature must trace to a user story (US-XXX-YYY)
- Every AC must reference a BDD scenario via `@AC-XXX-YYY` tag
- Status values: `proposed`, `in_progress`, `implemented`, `deprecated`

### 2. BDD Features (`specs/features/*.feature`)

Cucumber/Gherkin scenarios tagged with AC IDs:

```gherkin
# Template Version: v2.3.0
# Schema: spec_ledger.yaml v1.0

Feature: Template Core Endpoints

  @AC-TPL-001 @template-core
  Scenario: Health check returns 200 OK
    Given the service is running
    When I send a GET request to "/health"
    Then the response status should be 200
    And the response body should contain "status"
```

**Rules:**

- Every scenario tagged with `@AC-XXX-YYY`
- Feature files live under `specs/features/`
- Step definitions in `acceptance/tests/steps/`

### 3. Enforcement (`xtask ac-status`)

Automated validation that ledger ↔ features ↔ tests are in sync:

```bash
cargo run -p xtask -- ac-status
```

Checks:

- Every AC in ledger has a matching `@AC-XXX-YYY` tag in a feature file
- Every `@AC-XXX-YYY` tag in features exists in the ledger
- Feature file paths in ledger match reality
- Scenario names match between ledger and `.feature` files

**Enforcement point:**

- `xtask selftest` runs `ac-status` in step 3
- CI fails if mapping is broken

---

## Consequences

### Positive

- **Single source of truth**: Product and engineering share the ledger
- **Traceability**: From US → REQ → AC → scenario → test → code
- **Validation**: Broken links detected in CI, can't drift silently
- **LLM-friendly**: Structured YAML + Gherkin that agents can parse and modify
- **Living documentation**: Features are validated every build; if they pass, they're current

### Negative

- **Upfront ceremony**: Every AC requires ledger entry + feature + scenario + step def + code
- **Refactoring friction**: Renaming an AC means updating ledger, feature tag, and possibly tests
- **Learning curve**: Teams unfamiliar with BDD need onboarding
- **Noise for small changes**: Even trivial fixes require AC mapping (though this is also a feature)

### Neutral

- **Not a replacement for API docs**: OpenAPI/Swagger still needed for contract details
- **Not a replacement for runbooks**: Operational knowledge lives elsewhere

---

## Compliance

**Automated:**

- `cargo run -p xtask -- ac-status` validates ledger ↔ feature mapping
- `cargo run -p acceptance` runs Cucumber scenarios
- `xtask selftest` includes both checks (steps 3 and 4)
- CI fails if either check fails

**Manual:**

- Code review should reject PRs that:
  - Add features without updating the ledger
  - Change behavior without updating scenarios
  - Break AC mapping (detected by `ac-status`)

**Schema enforcement:**

- Future: JSON Schema for `spec_ledger.yaml` to catch structural errors early
- Future: Rego policy to validate ledger structure

---

## Notes

**Why YAML ledger instead of JSON?**

- YAML is human-friendly for manual edits
- Comments allowed (JSON doesn't support them)
- Less noise than JSON for nested structures

**Why Gherkin/BDD instead of just integration tests?**

- Gherkin is readable by non-engineers (product, QA)
- Tags (`@AC-XXX-YYY`) create explicit traceability
- Scenarios are **behavior contracts**, not implementation details

**What if a feature doesn't need BDD?**

If a requirement is non-functional (e.g., "service must start in <2s"), you can:

- Still create an AC in the ledger
- Point to a unit/integration test instead of a scenario
- Or write a simple scenario that validates the constraint

**Migration path:**

If you're adding this to an existing service:

1. Audit current features: list what the service actually does
2. Create initial `spec_ledger.yaml` with one US per major feature
3. Write one BDD scenario per AC (start with happy paths)
4. Wire step definitions to existing test helpers
5. Run `xtask ac-status` and fix broken mappings
6. From now on: AC-first workflow (ledger → scenario → code)

**AC-first workflow:**

For new features:

1. Update ledger: add US → REQ → AC
2. Write scenario: create `.feature` file with `@AC-XXX-YYY` tag
3. Run `xtask ac-status`: verify mapping is valid
4. Implement step definitions (they fail at first)
5. Implement business logic + adapters
6. Run `cargo run -p acceptance`: scenarios turn green
7. Commit when `xtask selftest` passes

**References:**

- [Cucumber documentation](https://cucumber.io/docs/gherkin/)
- [Specification by Example](https://www.manning.com/books/specification-by-example) (Gojko Adzic)
- [BDD in Action](https://www.manning.com/books/bdd-in-action) (John Ferguson Smart)
