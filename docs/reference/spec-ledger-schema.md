---
id: REF-SPEC-LEDGER-SCHEMA-001
title: Spec Ledger Schema Reference
doc_type: reference
status: published
audience: developers, maintainers
tags: [reference, schema, governance, spec-ledger]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT]
acs: []
adrs: [ADR-0003, ADR-0005]
last_updated: 2026-01-30
---
<!-- doclint:disable orphan-version -->

# Reference: Spec Ledger Schema

This document defines the YAML schema for `specs/spec_ledger.yaml`, the governance ledger that maps user stories to requirements, acceptance criteria, and tests.

**Related:**
- `crates/spec-runtime/README.md` - Runtime library for loading specs
- `docs/reference/config-schema.md` - Configuration schema
- `specs/spec_ledger.yaml` - The actual spec ledger file

---

## Schema Overview

The spec ledger follows a hierarchical structure:

```
metadata
  └── schema_version, template_version, last_updated, description, adrs[]
stories[]
  └── id, title, description?, adr?, requirements[]
        └── id, title, description?, tags[], must_have_ac, adr?, ci_workflows[]?, docs[]?, acceptance_criteria[]
              └── id, text, tags[], must_have_ac, note?, adr?, tests[]
                    └── type, tag, file?, module?, workflow?, note?
```

---

## Root Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `metadata` | `Metadata` | Yes | Schema and template versioning info |
| `stories` | `Story[]` | Yes | List of user stories |

---

## Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | Yes | Schema version (e.g., `"1.0"`) |
| `template_version` | string | Yes | Template version (e.g., `"3.3.14"`) |
| `last_updated` | string | Yes | ISO date of last update (e.g., `"2025-12-22"`) |
| `description` | string | No | Human-readable description |
| `adrs` | string[] | No | Template-wide ADR references (e.g., `["ADR-0002", "ADR-0004"]`) |

**Example:**

```yaml
metadata:
  schema_version: "1.0"
  template_version: "3.3.14"
  last_updated: "2025-12-22"
  description: "Template core capabilities"
  adrs:
    - ADR-0002
    - ADR-0004
```

---

## Story

A user story represents a high-level capability or feature.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier (e.g., `US-TPL-001`, `US-TPL-PLT-001`) |
| `title` | string | Yes | Human-readable story title |
| `description` | string | No | Detailed story description |
| `adr` | string or string[] | No | Related ADR reference(s) |
| `requirements` | `Requirement[]` | Yes | List of requirements for this story |

**ID Convention:** `US-{PREFIX}-{NUMBER}` where PREFIX identifies the domain (e.g., `TPL` for template, `PLT` for platform).

**Example:**

```yaml
stories:
  - id: US-TPL-001
    title: "Service Core Capabilities"
    adr: ADR-0001
    requirements:
      - id: REQ-TPL-HEALTH
        # ...
```

---

## Requirement

A requirement represents a specific capability that must be implemented.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier (e.g., `REQ-TPL-HEALTH`) |
| `title` | string | Yes | Human-readable requirement title |
| `description` | string | No | Detailed requirement description |
| `rationale` | string | No | Why this requirement exists |
| `tags` | string[] | Yes | Classification tags (see Tags section) |
| `must_have_ac` | boolean | Yes | Whether this requirement must have tested ACs |
| `adr` | string or string[] | No | Related ADR reference(s) |
| `ci_workflows` | string[] | No | Related CI workflow identifiers |
| `docs` | string[] | No | Related documentation paths |
| `acceptance_criteria` | `AcceptanceCriterion[]` | Yes | List of ACs for this requirement |

**ID Convention:** `REQ-{PREFIX}-{NAME}` where PREFIX matches the story prefix and NAME is a descriptive slug.

**Example:**

```yaml
requirements:
  - id: REQ-TPL-HEALTH
    title: "Health Check Endpoint"
    tags: [platform, structural]
    must_have_ac: true
    adr: ADR-0003
    acceptance_criteria:
      - id: AC-TPL-001
        # ...
```

---

## AcceptanceCriterion

An acceptance criterion defines a specific, testable behavior.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier (e.g., `AC-TPL-001`) |
| `text` | string | Yes | Full acceptance criterion text (testable statement) |
| `tags` | string[] | Yes | Classification tags (see Tags section) |
| `must_have_ac` | boolean | Yes | Whether this AC must have passing tests |
| `note` | string | No | Implementation notes or caveats |
| `adr` | string or string[] | No | Related ADR reference(s) |
| `tests` | `Test[]` | No | List of tests covering this AC |

**ID Convention:** `AC-{PREFIX}-{NUMBER}` or `AC-{PREFIX}-{NAME}` for descriptive IDs.

**Example:**

```yaml
acceptance_criteria:
  - id: AC-TPL-001
    text: "GET /health returns 200 with status 'ok' when service is healthy"
    tags: [kernel]
    must_have_ac: true
    tests:
      - { type: bdd, tag: "@AC-TPL-001", file: "specs/features/template_core.feature" }
      - { type: unit, tag: "test_health_returns_ok", module: "app_http::tests", file: "crates/app-http/src/lib.rs" }
    adr: ADR-0005
```

---

## Test

A test reference links an AC to its test evidence.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Test type (see Test Types below) |
| `tag` | string | Yes | Test identifier or BDD tag |
| `file` | string | Conditional | File path (required for unit/bdd/integration) |
| `module` | string | No | Rust module path (for unit tests) |
| `workflow` | string | No | CI workflow file (for ci type) |
| `note` | string | No | Additional notes |

**Test Types:**

| Type | Description | Required Fields |
|------|-------------|-----------------|
| `unit` | Rust unit test | `tag`, `file`, optionally `module` |
| `bdd` | Cucumber/Gherkin scenario | `tag` (with @prefix), `file` |
| `integration` | Integration test | `tag`, `file` |
| `ci` | CI workflow verification | `tag`, optionally `workflow`, `note` |

**Examples:**

```yaml
# Unit test
- { type: unit, tag: "test_health_returns_ok", module: "app_http::tests", file: "crates/app-http/src/lib.rs" }

# BDD scenario
- { type: bdd, tag: "@AC-TPL-001", file: "specs/features/template_core.feature" }

# Integration test
- { type: integration, tag: "@AC-PLT-001", file: "specs/features/xtask_devex.feature" }

# CI workflow
- { type: ci, tag: "ac_demotion_governed", note: "Governance policy" }
```

---

## Tags

Tags classify stories, requirements, and ACs for filtering and reporting.

### Common Tags

| Tag | Level | Description |
|-----|-------|-------------|
| `kernel` | AC | Part of frozen kernel baseline (must pass) |
| `template` | AC | Template-specific (not required for forks) |
| `platform` | Req | Platform infrastructure |
| `devex` | Req | Developer experience |
| `security` | Req | Security-related |
| `release` | Req/AC | Release process |
| `governance` | Req/AC | Governance infrastructure |
| `structural` | Req | Structural/architectural |
| `docs` | Req | Documentation |
| `ci-only` | AC | Only verified in CI (not local selftest) |
| `ai` | AC | Agent/LLM integration |
| `idp` | AC | Developer portal integration |

### Tag Inheritance

Tags on requirements apply context to their ACs, but AC tags are evaluated independently for kernel/template classification.

---

## Validation Rules

The spec ledger is validated by `cargo xtask selftest` and `spec-runtime`:

1. **Unique IDs** - All story, requirement, and AC IDs must be unique
2. **Required fields** - All required fields must be present
3. **Test coverage** - ACs with `must_have_ac: true` should have tests
4. **Tag validity** - Tags must be from the recognized set
5. **ADR references** - Referenced ADRs should exist
6. **File references** - Test file paths should exist

**Validation command:**

```bash
cargo xtask selftest  # Full validation including spec ledger
```

---

## Loading the Spec Ledger

Use `spec-runtime` to load and query the spec ledger:

```rust
use spec_runtime::{load_spec_ledger, build_ac_id_index};

let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;

// Build index for fast AC lookup
let ac_index = build_ac_id_index(&ledger);

if let Some(ac_ref) = ac_index.get("AC-TPL-001") {
    println!("AC: {}", ac_ref.ac.text);
    println!("Story: {}", ac_ref.story_id);
    println!("Requirement: {}", ac_ref.req_id);
}
```

---

## Schema Evolution

The spec ledger schema is versioned via `metadata.schema_version`. Breaking changes require:

1. Major version bump in `schema_version`
2. ADR documenting the change
3. Migration guidance in release notes

Shape-lock tests in `spec-runtime` prevent accidental schema drift.

---

## See Also

- `crates/spec-runtime/README.md` - Full API documentation
- `docs/reference/config-schema.md` - Configuration schema
- `docs/AGENT_GUIDE.md` - Agent workflows using specs
- `specs/spec_ledger.yaml` - The actual spec ledger
