---
id: EXPLANATION-TPL-CONTRACTS-001
title: "Template Contracts: Kernel Requirements and Extension Points"
doc_type: explanation
status: published
audience: fork-maintainers, platform-engineers, contributors
tags: [kernel, contracts, governance, bdd, fork]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-BDD-HARNESS
  - REQ-TPL-FORK-VISIBILITY
  - REQ-TPL-OPINIONATED-DEFAULTS
  - REQ-PLT-DOCS-CONSISTENCY
acs:
  - AC-TPL-BDD-EXIT-CODES
  - AC-TPL-KERNEL-CONTRACT-EMITTED
  - AC-TPL-OPINIONS-DOCUMENTED
adrs: [ADR-0005]
last_updated: 2025-12-22
---

# Template Contracts: Kernel Requirements and Extension Points

**Template Version:** v3.3.15
**Schema Version:** spec_ledger.yaml v1.0
**Last Updated:** 2025-12-07

> **See also:**
> - [Rust-as-Spec Overview](rust-as-spec-overview.md) – The conceptual model and four-phase pipeline
> - [Template Architecture](template-architecture.md) – The layered architecture and planes

---

## TL;DR: What You Must Preserve When Forking

If you're building a new service from this template (like the Knowledge Hub), you **MUST**:

1. **Run `cargo xtask selftest` in CI** as a merge gate on your main branch
2. **Keep these files aligned**: `spec_ledger.yaml` (your stories/REQs/ACs) + `specs/features/*.feature` (BDD tests) + test code
3. **Expose these HTTP endpoints**: `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints`, `/platform/docs/index`, `/platform/schema`, `/platform/openapi`
4. **Maintain these files**: `service_metadata.yaml`, `specs/tasks.yaml`, release evidence bundles
5. **Tag kernel ACs with** `must_have_ac: true` only if they're truly required for your service

Everything else (your domain logic, additional routes, crates, deployment) is yours to customize.

**See [docs/KERNEL_SNAPSHOT.md](../KERNEL_SNAPSHOT.md) for the baseline v3.3.9-kernel ACs.**

---

This document defines the **template kernel contracts** – the core APIs, behaviors, and structures that MUST be preserved in every service built from this template – versus the **customization surface** where you're free to extend, modify, or replace.

## Purpose and Scope

The Rust-as-Spec Platform Cell template provides:

1. **A stable kernel** – contracts that selftest, policies, and automation depend on
2. **Flexibility** – clear extension points for domain-specific features
3. **Governance** – automated checks that prevent accidental contract violations

**Golden Rule:** If `cargo xtask selftest` checks it and it's marked `must_have_ac: true` in `specs/spec_ledger.yaml`, it's a kernel contract. Everything else is customizable.

---

## How Kernel Contracts Are Defined

Kernel contracts are identified by:

1. **Requirement Flag**: `must_have_ac: true` on the requirement in `specs/spec_ledger.yaml`
2. **AC Flag**: `must_have_ac: true` on the acceptance criterion
3. **Tag**: `tags: [kernel]` on the AC
4. **Selftest Enforcement**: Validated by `cargo xtask selftest` (step 8: AC coverage check)

Non-kernel requirements and ACs (with `must_have_ac: false` or `tags: [future]`) are informational and do not block selftest.

---

## Kernel Contracts at a Glance

This table summarizes the key kernel contract areas for quick reference:

| Area | Requirement | ACs / IDs | Enforced by |
|------|-------------|-----------|-------------|
| Docs-as-Code | REQ-PLT-DOCS-CONSISTENCY | AC-PLT-009, AC-PLT-010, AC-PLT-DOC-INDEX-FRONTMATTER | `cargo xtask docs-check` |
| BDD harness | REQ-TPL-BDD-HARNESS | AC-TPL-BDD-EXIT-CODES | `cargo xtask bdd`, `selftest` |
| Bundles | REQ-TPL-BUNDLE-CONTRACT | AC-TPL-BUNDLE-MANIFEST, AC-TPL-BUNDLE-MANIFEST-LINKED, AC-TPL-BUNDLE-MINIMAL-SCOPE | `cargo xtask bundle`, `selftest` |
| Agent surfaces | REQ-TPL-AGENT-INTERFACE | AC-TPL-AGENT-HINTS-SCHEMA, AC-TPL-CLI-JSON-CORE, AC-TPL-CLI-JSON-OUTPUT | `/platform/*`, `suggest-next` |
| Example fork | REQ-TPL-EXAMPLE-FORK-CONTRACT | AC-TPL-EXAMPLE-FORK-BUILDS | `ci-example-fork.yml` workflow |
| IDP Snapshot | REQ-TPL-IDP-SNAPSHOT | AC-TPL-IDP-SNAPSHOT, AC-TPL-IDP-SNAPSHOT-VALID-JSON | `cargo xtask idp-snapshot` |

For the full contract definitions, see the sections below and `specs/spec_ledger.yaml`.

---

## AC-to-Test Traceability Pattern

The template enforces bidirectional traceability between Acceptance Criteria and their tests.

### Forward Mapping: AC → Tests

Every AC in `specs/spec_ledger.yaml` MUST include a `tests` array that specifies:

```yaml
acceptance_criteria:
  - id: AC-TPL-001
    text: "GET /health returns 200 with status 'ok' when service is healthy"
    tags: [kernel]
    must_have_ac: true
    tests:
      - type: bdd
        tag: "@AC-TPL-005"
        file: "specs/features/template_core.feature"
      - type: unit
        tag: "test_health_endpoint"
        module: "health::tests::test_health_endpoint"
        file: "crates/app-http/src/routes/health.rs"
```

**Required Fields:**
- `type`: Test type (`bdd`, `integration`, `unit`, `manual`)
- `tag`: Unique test identifier (BDD tag or unit test name)
- `file`: Path to the test file (enables `cargo xtask ac-tests <AC-ID>` to show locations)

**Optional Fields:**
- `module`: Rust module path for unit tests (e.g., `health::tests::test_health_endpoint`)

### Reverse Mapping: Test → AC

Test files MUST include AC attribution in their documentation comments:

**For Rust unit tests:**

```rust
/// AC-TPL-CONFIG-VALIDATION: Validates that the service rejects invalid
/// configuration at startup and exits with a clear error message.
#[test]
fn config_validation_rejects_invalid() {
    // Test implementation
}
```

**For BDD feature files:**

```gherkin
@AC-TPL-001
Scenario: Health endpoint returns OK when service is healthy
  Given the service is running
  When I GET /health
  Then the response status should be 200
  And the response body should contain "ok"
```

### Discovery Commands

**List all tests for an AC:**

```bash
cargo xtask ac-tests AC-TPL-001
```

Output:

```
================================================================================
Acceptance Criterion: AC-TPL-001
================================================================================

Story: US-TPL-001
Requirement: REQ-TPL-HEALTH
Text: GET /health returns 200 with status 'ok' when service is healthy

Mapped Tests:
--------------------------------------------------------------------------------

[1] Type: bdd
    Tag: @AC-TPL-005
    File: specs/features/template_core.feature

Run Tests:
--------------------------------------------------------------------------------

  BDD/Integration: cargo xtask test-ac AC-TPL-001
  Direct: CUCUMBER_TAG_EXPRESSION='@AC-TPL-005' cargo test -p acceptance
```

**Check AC coverage:**

```bash
cargo xtask ac-status --summary
```

### Governance Graph Integration

The governance graph (`/platform/graph` API and `cargo xtask graph-export`) includes:

1. **AC nodes**: Acceptance criteria from `spec_ledger.yaml`
2. **Test nodes**: Generated from the `tests` array of each AC
3. **tested_by edges**: AC → Test relationships (forward traceability)

Example graph structure:

```json
{
  "nodes": [
    {"id": "AC-TPL-001", "label": "Health endpoint", "type": "ac"},
    {"id": "AC-TPL-001:test:0", "label": "@AC-TPL-005 - specs/features/template_core.feature", "type": "test", "url": "file://specs/features/template_core.feature"}
  ],
  "edges": [
    {"source": "AC-TPL-001", "target": "AC-TPL-001:test:0", "type": "tested_by"}
  ]
}
```

### Validation and Enforcement

**At design time:**
- `cargo xtask ac-new` prompts for test details when creating an AC

**At commit time:**
- Pre-commit hooks run `cargo xtask ac-status` (if hooks installed)

**At CI time:**
- `cargo xtask selftest` step 2 runs all BDD tests
- `cargo xtask selftest` step 8 validates AC coverage (all kernel ACs must pass)

**At query time:**
- `cargo xtask ac-tests <AC-ID>` shows all mapped tests
- `cargo xtask test-ac <AC-ID>` runs tests for a specific AC
- `/platform/graph` API exposes test relationships

### Adding Tests to Existing ACs

If an AC exists without test mappings, add them:

```yaml
# Before
acceptance_criteria:
  - id: AC-TPL-001
    text: "Description"
    tests: []  # Empty!

# After
acceptance_criteria:
  - id: AC-TPL-001
    text: "Description"
    tests:
      - type: bdd
        tag: "@AC-TPL-001"
        file: "specs/features/my_feature.feature"
```

Then annotate the test file:

```gherkin
@AC-TPL-001
Scenario: Test scenario
  # Steps
```

Or for unit tests:

```rust
/// AC-TPL-001: Description of what this test validates
#[test]
fn test_name() {
    // Test implementation
}
```

---

## Kernel Contracts by Category

### 1. HTTP Service Core (REQ-TPL-HEALTH, REQ-TPL-VERSION, REQ-TPL-ERROR-HANDLING, REQ-TPL-METRICS)

#### AC-TPL-001: Health Check Endpoint

**Contract:**
- `GET /health` returns 200 OK with `{"status": "ok"}` when service is healthy

**Why:**
- Required for Kubernetes liveness/readiness probes
- Standard operational contract for any service

**BDD Test:** `@AC-TPL-005` in `specs/features/template-core/*.feature`

**How to Maintain:**
- Keep health endpoint handler in `crates/app-http/src/routes/health.rs`
- Do NOT remove or rename this endpoint
- May extend with additional health checks (database, dependencies)

---

#### AC-TPL-002: Version Information Endpoint

**Contract:**
- `GET /version` returns 200 OK with build information including version and git SHA

**Why:**
- Required for deployment verification
- Enables diagnostics and troubleshooting
- Links runtime state to source control

**BDD Test:** `@AC-TPL-002`

**How to Maintain:**
- Keep version endpoint handler
- Version information generated at build time via `build.rs`
- Do NOT remove version endpoint

---

#### AC-TPL-003: Error Response Envelope

**Contract:**
- All 4xx/5xx responses include an error code, message, and request ID
- Standard error format:

  ```json
  {
    "error": {
      "code": "ERROR_CODE",
      "message": "Human-readable message",
      "request_id": "uuid",
      "details": { /* optional */ }
    }
  }
  ```

**Why:**
- Provides consistent error handling across all endpoints
- Enables structured error logging and monitoring
- Allows clients to parse and handle errors reliably

**BDD Test:** `@AC-TPL-003`

**How to Maintain:**
- Use `ErrorResponse` type from `crates/model/`
- All HTTP error responses must use this envelope
- May add new error codes for domain-specific errors

---

#### AC-TPL-004: Request ID Propagation

**Contract:**
- Handlers propagate or generate `X-Request-ID` and expose it in responses
- Every request has a unique request ID
- Request ID appears in logs and error responses

**Why:**
- Enables distributed tracing
- Critical for debugging production issues
- Standard observability practice

**BDD Test:** `@AC-TPL-004`

**How to Maintain:**
- Keep request ID middleware in `crates/app-http/src/middleware/`
- All logging must include request ID
- Response must echo the `X-Request-ID` header

---

#### AC-TPL-007: Prometheus Metrics Endpoint

**Contract:**
- `GET /metrics` returns Prometheus-formatted metrics including `http_requests_total`

**Why:**
- Required for monitoring and alerting
- Standard observability contract
- Enables SLO tracking

**BDD Test:** `@AC-TPL-007`

**How to Maintain:**
- Keep metrics endpoint handler
- May add domain-specific metrics
- Do NOT remove the `/metrics` endpoint

---

### 2. Platform Introspection APIs (REQ-TPL-PLATFORM-INTROSPECTION)

#### AC-TPL-PLATFORM-GRAPH: Governance Graph API

**Contract:**
- `GET /platform/graph` returns the full governance graph in JSON format
- Response includes `nodes` (array) and `edges` (array)

**Why:**
- Enables agents and dashboards to understand the platform
- Provides runtime view of stories → requirements → ACs → tests
- Supports autonomous agent workflows

**BDD Test:** `@AC-TPL-PLATFORM-GRAPH` in `specs/features/platform_introspection.feature`

**How to Maintain:**
- Graph is built from `specs/spec_ledger.yaml` at startup
- Endpoint implemented in platform router
- Do NOT remove this endpoint

---

#### AC-TPL-PLATFORM-DEVEX: DevEx Flows API

**Contract:**
- `GET /platform/devex/flows` returns canonical flows definition from `specs/devex_flows.yaml`
- Response includes `commands` (object) and `flows` (object)

**Why:**
- Exposes available xtask commands and workflows
- Enables agents to discover and execute workflows
- Provides machine-readable developer experience contract

**BDD Test:** `@AC-TPL-PLATFORM-DEVEX`

**How to Maintain:**
- Flows loaded from `specs/devex_flows.yaml`
- Endpoint reflects current command set
- Keep endpoint synchronized with xtask commands

---

#### AC-TPL-PLATFORM-DOCS: Documentation Index API

**Contract:**
- `GET /platform/docs/index` returns documentation index
- Response includes `docs` (array) with available documentation

**Why:**
- Makes documentation discoverable via API
- Supports agent-driven documentation lookup
- Provides inventory of available docs

**BDD Test:** `@AC-TPL-PLATFORM-DOCS`

**How to Maintain:**
- Index built from documentation directory scan
- Endpoint returns current doc inventory
- Keep synchronized with actual documentation

---

#### AC-TPL-POLICY-STATUS-OVERVIEW: Policy Status in Platform Status

**Contract:**
- `GET /platform/status` includes `governance.policies.status` field
- Status derived from last policy-test run (pass/fail/unknown)
- Read from `target/policy_status.json`

**Why:**
- Surfaces governance health at runtime
- Enables monitoring of policy compliance
- Provides operational visibility into governance state

**BDD Test:** `@AC-TPL-PLATFORM-STATUS`

**How to Maintain:**
- Policy status updated by `cargo xtask policy-test`
- Status file written to `target/policy_status.json`
- Platform status endpoint reads and exposes this state

---

### 3. Platform Schema and Metadata (REQ-TPL-PLATFORM-SCHEMA, REQ-TPL-METADATA-CONSISTENT)

#### AC-TPL-PLATFORM-SCHEMA: Machine-Readable Schema

**Contract:**
- `GET /platform/schema` returns the schema index (JSON Schema + endpoint list)
- `GET /platform/openapi` returns the OpenAPI document
- Includes schemas for `/platform/status`, `/platform/graph`, `/platform/tasks`, `/platform/agent/hints`

**Why:**
- Enables IDP dashboards and tooling to consume platform APIs
- Provides contract documentation without manual scraping
- Supports automated client generation

**BDD Test:** `@AC-TPL-PLATFORM-SCHEMA` in `specs/features/platform_schema.feature`

**How to Maintain:**
- Schema generated from platform route definitions
- Keep synchronized with actual endpoints
- Update when adding new platform APIs

---

#### AC-TPL-METADATA-COMPLETE: Service Metadata Consistency

**Contract:**
- `specs/service_metadata.yaml` includes `service_id`, `template_version`, URLs, and tags
- `/platform/status` returns the same identifiers
- UI links to runbook, roadmap, agent guide, feature status, platform support docs

**Why:**
- Enables fleet tooling to identify and categorize services
- Provides operational metadata at runtime
- Links to key documentation from UI

**BDD Test:** `@AC-TPL-METADATA-COMPLETE`

**How to Maintain:**
- Keep `specs/service_metadata.yaml` complete and accurate
- Ensure `/platform/status` reflects this metadata
- Update UI links when documentation moves

---

### 4. Platform UI (REQ-TPL-PLATFORM-UI)

#### AC-TPL-PLATFORM-UI-DASHBOARD: Dashboard Homepage

**Contract:**
- `GET /` or `GET /ui` serves HTML dashboard
- Shows platform status and governance health metrics from `/platform/status`

**Why:**
- Provides human-friendly view of governance state
- Enables quick health checks
- Surfaces key metrics and status

**BDD Test:** `@AC-TPL-PLATFORM-UI-DASHBOARD`

**How to Maintain:**
- UI templates in `crates/app-http/src/templates/` or static assets
- Dashboard consumes `/platform/status` API
- Keep synchronized with status endpoint

---

#### AC-TPL-PLATFORM-UI-GRAPH: Graph Visualization

**Contract:**
- UI provides graph visualization rendering governance graph
- Uses Mermaid.js to render stories, requirements, ACs, docs, commands

**Why:**
- Makes governance graph human-comprehensible
- Enables visual exploration of relationships
- Supports understanding of system structure

**BDD Test:** `@AC-TPL-PLATFORM-UI-GRAPH`

**How to Maintain:**
- Graph data from `/platform/graph` API
- Visualization uses Mermaid.js
- Keep rendering logic synchronized with graph structure

---

#### AC-TPL-PLATFORM-UI-FLOWS: Flows and Tasks View

**Contract:**
- UI provides flows and tasks view
- Displays DevEx flows and available tasks from platform APIs

**Why:**
- Makes workflows discoverable via UI
- Enables task tracking and progress visibility
- Supports human and agent workflows

**BDD Test:** `@AC-TPL-PLATFORM-UI-FLOWS`

**How to Maintain:**
- Flows data from `/platform/devex/flows` API
- Tasks data from `/platform/tasks` API
- Keep view synchronized with data sources

---

### 5. Agent Interface (REQ-TPL-AGENT-INTERFACE, REQ-TPL-SKILLS-GUIDE, REQ-TPL-SKILLS-TOOLING)

#### AC-TPL-AGENT-SKILLS: Skill Definitions

**Contract:**
- `.claude/skills/` contains executable skill definitions
- Skills for feature development, release, maintenance workflows
- Each skill references appropriate xtask commands and platform APIs

**Why:**
- Enables AI agents to execute governance workflows autonomously
- Provides structured guidance for agent-driven development
- Reduces guesswork and improves agent reliability

**BDD Test:** `@AC-TPL-AGENT-SKILLS` in `specs/features/xtask_devex.feature`

**How to Maintain:**
- Keep core skills: `governed-feature-dev`, `governed-maintenance`, `governed-release`, `governed-governance-debug`, `bootstrap-dev-env`
- Each skill must reference xtask commands and platform endpoints
- Skills must align with workflows in `specs/devex_flows.yaml`

---

#### AC-TPL-AGENT-HINTS-SCHEMA: Hint Schema Definition

**Contract:**
- `GET /platform/agent/hints` returns hints with a well-defined schema
- Each hint contains these canonical fields:
  - `id`: Unique hint identifier (e.g., `HINT-TASK-001`)
  - `kind`: Category - `task`, `governance`, `policy`, or `flow`
  - `priority`: `low`, `medium`, or `high`
  - `status`: `open`, `in_progress`, or `done`
  - `reason`: Object with `code` (machine-readable) and `details` (human-readable)
  - `target`: Tagged union with `type` and `id` (e.g., `{"type": "task", "id": "TASK-001"}`)
  - `tags`: Array of labels for filtering
  - `links`: Object with `spec`, `task`, `docs`, `adrs`, `extra` for resource references
- CLI `suggest-next --format json` is a projection of this schema (not a separate contract)

**Why:**
- Provides a stable, machine-readable interface for agent consumption
- Enables consistent tooling across HTTP API and CLI
- Supports filtering, prioritization, and automated workflows

**BDD Test:** `@AC-TPL-AGENT-HINTS-SCHEMA` in `specs/features/agent_hints.feature`

**How to Maintain:**
- Canonical types defined in `crates/spec-runtime/src/hints.rs`
- HTTP endpoint in `crates/app-http/src/agent.rs` reuses these types directly
- CLI projection mirrors schema fields for convenience

**Example Response:** See [`docs/examples/agent_hints_response.json`](../examples/agent_hints_response.json) for a representative response.

---

#### AC-TPL-QUESTIONS-LOGGED: Question Artifacts

**Contract:**
- Ambiguity during automated flows or suggest-next emits a structured question
- Questions logged as file/PR comment/status entry
- Can be surfaced to humans or agents without halting progress

**Why:**
- Enables autonomous agent workflows to continue despite ambiguity
- Creates audit trail of decision points
- Supports asynchronous human review

**BDD Test:** `@AC-TPL-QUESTIONS-LOGGED` in `specs/features/questions.feature`

**How to Maintain:**
- Emit structured questions when encountering ambiguous input
- Do not block or fail workflows when questions are logged
- Include context and options in question artifacts

---

#### AC-TPL-FLOW-IDEMPOTENT: Flow Idempotency

**Contract:**
- Running `cargo xtask selftest` or `cargo xtask suggest-next` multiple times without changes produces stable outputs
- No duplicate artifacts created on repeated runs

**Why:**
- Enables safe agent retries and self-healing flows
- Prevents state corruption from repeated executions
- Supports reliable automated workflows

**BDD Test:** `@AC-TPL-FLOW-IDEMPOTENT` in `specs/features/flow_idempotency.feature`

**How to Maintain:**
- Design flows to check existing state before creating artifacts
- Use idempotent operations (upsert vs insert)
- Test flows with repeated execution

---

#### AC-TPL-XTASK-SPEC-ROOT: Isolated Testing Support

**Contract:**
- xtask spec-reading commands respect `SPEC_ROOT` environment variable when set
- BDD harness uses `SPEC_ROOT` to work on isolated temp workspaces
- Commands reading `spec_ledger.yaml`, `tasks.yaml`, etc. resolve paths relative to `SPEC_ROOT`

**Why:**
- Enables BDD scenarios to test xtask commands without modifying the real workspace
- Supports hermetic testing with isolated fixtures
- Prevents test pollution of production spec files

**BDD Test:** `@AC-TPL-XTASK-SPEC-ROOT` in `specs/features/agent_hints.feature`

**How to Maintain:**
- All xtask commands that read spec files should check `SPEC_ROOT` first
- Use `spec_runtime::workspace::resolve_spec_path()` helper for consistent resolution
- BDD steps set `SPEC_ROOT` to a temp directory before invoking commands

---

#### AC-TPL-SKILLS-GUIDE-001: Skills Documentation

**Contract:**
- `docs/AGENT_SKILLS.md` exists and documents recommended skill set
- Includes SKILL.md templates and best practices

**Why:**
- Provides guidance for creating and maintaining skills
- Ensures consistency across skill definitions
- Supports skill authoring

**BDD Test:** `@AC-TPL-SKILLS-GUIDE-001`

**How to Maintain:**
- Keep skills documentation up to date
- Document skill authoring conventions
- Include examples and templates

---

#### AC-TPL-SKILLS-ALIGN-001: Skills Alignment

**Contract:**
- Existing `.claude/skills/*` aligned with documented workflows
- Skills: `bootstrap-dev-env`, `governed-feature-dev`, `governed-maintenance`, `governed-release`, `governed-governance-debug`

**Why:**
- Ensures skills match actual workflows
- Prevents drift between skills and platform
- Maintains skill reliability

**BDD Test:** `@AC-TPL-SKILLS-ALIGN-001`

**How to Maintain:**
- Review skills when workflows change
- Update skill references to commands and APIs
- Keep skills synchronized with devex_flows.yaml

---

#### AC-TPL-SKILLS-FMT: Skills Formatting

**Contract:**
- `cargo run -p xtask -- skills-fmt` normalizes SKILL.md files
- Enforces frontmatter, headings, links conventions

**Why:**
- Ensures consistent skill formatting
- Prevents drift and parsing errors
- Enables automated skill processing

**BDD Test:** `@AC-TPL-SKILLS-FMT`

**How to Maintain:**
- Run `skills-fmt` before committing skill changes
- Keep formatting rules in xtask implementation
- Ensure frontmatter schema is documented

---

#### AC-TPL-SKILLS-LINT: Skills Validation

**Contract:**
- `cargo run -p xtask -- skills-lint` validates skills frontmatter and content
- Checks name/description rules, references to flows and APIs

**Why:**
- Catches skill authoring errors early
- Ensures skill metadata completeness
- Validates references to platform contracts

**BDD Test:** `@AC-TPL-SKILLS-LINT`

**How to Maintain:**
- Run `skills-lint` in CI and pre-commit hooks
- Keep validation rules updated
- Extend linting for new skill patterns

---

### 6. DevEx Platform Commands (REQ-PLT-ONBOARDING, REQ-PLT-DESIGN-SCAFFOLDING, REQ-PLT-SECURITY-GOVERNANCE, REQ-PLT-DOCS-CONSISTENCY, REQ-PLT-RELEASE-SAFETY, REQ-PLT-DEVEX-CONTRACT, REQ-PLT-STATUS-CLI)

#### AC-PLT-001: Doctor Command

**Contract:**
- `cargo xtask doctor` validates Rust, Nix, conftest, git
- Provides next-steps guidance
- Checks environment health

**BDD Test:** `@AC-PLT-001` in `specs/features/xtask_devex.feature`

**How to Maintain:**
- Keep doctor checks comprehensive
- Update for new tool dependencies
- Provide actionable recovery guidance

---

#### AC-PLT-002: Help Flows Command

**Contract:**
- `cargo xtask help-flows` renders categorized command map
- Reads from `specs/devex_flows.yaml`

**BDD Test:** `@AC-PLT-002`

**How to Maintain:**
- Keep synchronized with `devex_flows.yaml`
- Update when adding new commands
- Ensure categorization is clear

---

#### AC-PLT-003: Check Command

**Contract:**
- `cargo xtask check` runs fmt + clippy + tests as fast dev loop

**BDD Test:** `@AC-PLT-003`

**How to Maintain:**
- Keep fast (< 30 seconds on warm cache)
- Run essential checks only
- Do NOT add slow checks here (use selftest instead)

---

#### AC-PLT-018: Dev-Up Command

**Contract:**
- `cargo xtask dev-up` runs doctor + install-hooks + check
- Displays next steps on success

**BDD Test:** `@AC-PLT-018`

**How to Maintain:**
- Keep one-command onboarding experience
- Ensure idempotent (safe to re-run)
- Provide clear "what next" guidance

---

#### AC-PLT-004: ADR-New Command

**Contract:**
- `cargo xtask adr-new <title>` creates numbered ADR from template
- Includes metadata (date, status, context, decision, consequences)

**BDD Test:** `@AC-PLT-004`

**How to Maintain:**
- Keep ADR template in sync with ADR format
- Auto-number ADRs sequentially
- Validate ADR structure on creation

---

#### AC-PLT-005: AC-New Command

**Contract:**
- `cargo xtask ac-new <ID> <desc>` rejects duplicate IDs
- Generates YAML snippet for `spec_ledger.yaml`

**BDD Test:** `@AC-PLT-005`

**How to Maintain:**
- Check for duplicate AC IDs before creation
- Generate valid YAML structure
- Provide clear instructions for insertion

---

#### AC-PLT-006: Audit Command

**Contract:**
- `cargo xtask audit` runs cargo-audit + cargo-deny
- Uses repo policy from `deny.toml`

**BDD Test:** `@AC-PLT-006`

**How to Maintain:**
- Run both audit tools
- Report vulnerabilities clearly
- Exit non-zero on failure

---

#### AC-PLT-007: Audit Recovery Guidance

**Contract:**
- `cargo xtask audit` provides 4-step recovery guidance on failure

**BDD Test:** `@AC-PLT-007`

**How to Maintain:**
- Include actionable recovery steps
- Link to policy configuration
- Provide examples of common fixes

---

#### AC-PLT-008: SBOM Generation

**Contract:**
- `cargo xtask sbom-local` generates SPDX JSON to `target/sbom.spdx.json`

**BDD Test:** `@AC-PLT-008`

**How to Maintain:**
- Use `cargo-sbom` or equivalent
- Generate valid SPDX 2.3+ format
- Include all workspace dependencies

---

#### AC-PLT-009: Docs-Check Version Alignment

**Contract:**
- `cargo xtask docs-check` validates version alignment across `spec_ledger.yaml` (canonical) and 8 consumer files

**Consumer Files Validated:**
1. `README.md` – H1 `(vX.Y.Z)` + Template Version badge
2. `CLAUDE.md` – H1 `(vX.Y.Z)` + Template Version line
3. `docs/ROADMAP.md` – H1 `(vX.Y.Z)`
4. `docs/KERNEL_SNAPSHOT.md` – H1 `vX.Y.Z`
5. `docs/explanation/TEMPLATE-CONTRACTS.md` – Template Version badge
6. `specs/service_metadata.yaml` – `template_version` field
7. `specs/doc_index.yaml` – `template_version` field
8. `CHANGELOG.md` – First `## [X.Y.Z]` section after `[Unreleased]`

**BDD Test:** `@AC-PLT-009`

**How to Maintain:**
- All versions must match `spec_ledger.yaml.metadata.template_version`
- Fail with detailed mismatch report showing file, expected, found, pattern
- Provide guidance to run `cargo xtask release-prepare X.Y.Z` for auto-update

---

#### AC-PLT-010: Docs-Check Feature Status

**Contract:**
- `cargo xtask docs-check` regenerates `feature_status.md`
- Fails on dirty git tree (uncommitted changes to generated doc)

**BDD Test:** `@AC-PLT-010`

**How to Maintain:**
- Regenerate feature_status.md
- Check git diff for changes
- Fail if file changed (not committed)

---

#### Docs Alignment Invariants (AC-PLT-009 / AC-PLT-010)

**Contract:**

`cargo xtask docs-check` enforces that these sources agree on version and kernel state:

**Canonical Source:**
- `specs/spec_ledger.yaml` → `metadata.template_version` (single source of truth)

**Consumer Files (8 total):**
- `README.md` → Template Version badge + Kernel Version section
- `CLAUDE.md` → H1 title with version
- `docs/ROADMAP.md` → H1 title with version
- `docs/KERNEL_SNAPSHOT.md` → H1 with kernel version
- `docs/explanation/TEMPLATE-CONTRACTS.md` → Template Version badge
- `specs/service_metadata.yaml` → `template_version` field
- `specs/doc_index.yaml` → `template_version` field
- `CHANGELOG.md` → latest `[X.Y.Z]` entry after `[Unreleased]`

**On version bumps:**

1. Update `spec_ledger.yaml` metadata (or use `cargo xtask release-prepare X.Y.Z`).
2. All 8 consumer files will be validated by `docs-check`.
3. Add or update the `[X.Y.Z]` section in `CHANGELOG.md`.
4. Run `cargo xtask docs-check && cargo xtask selftest`.
5. Use `cargo xtask version --json` to verify the canonical version.

---

#### AC-PLT-011: Release-Prepare Command

**Contract:**
- `cargo xtask release-prepare X.Y.Z` updates `spec_ledger.yaml`, `README.md`, `CLAUDE.md`, `CHANGELOG.md`

**BDD Test:** `@AC-PLT-011`

**How to Maintain:**
- Update all version references
- Seed CHANGELOG with release section
- Do NOT commit changes (let developer review)

---

#### AC-PLT-012: Release-Verify Command

**Contract:**
- `cargo xtask release-verify` runs selftest + audit + docs-check + clean tree check

**BDD Test:** `@AC-PLT-012`

**How to Maintain:**
- Run full validation suite
- Check git working tree is clean
- Exit non-zero on any failure

---

#### AC-PLT-013: Release-Verify Git Guidance

**Contract:**
- `cargo xtask release-verify` provides git command sequence on success
- Example: tag, push, CI validation steps

**BDD Test:** `@AC-PLT-013`

**How to Maintain:**
- Print actionable git commands
- Include tag creation, push steps
- Reference CI validation

---

#### AC-PLT-014: DevEx Flows Spec

**Contract:**
- Canonical flows and commands defined in `specs/devex_flows.yaml`

**BDD Test:** Unit test `devex_flows_schema_valid` in `crates/spec-runtime/src/devex.rs`

**How to Maintain:**
- Keep devex_flows.yaml as single source of truth
- Validate YAML schema on load
- Document all commands and flows

---

#### AC-PLT-015: Selftest DevEx Contract

**Contract:**
- `cargo xtask selftest` enforces devex contract (required commands exist)

**BDD Test:** `@AC-PLT-015`

**How to Maintain:**
- Check all `required: true` commands exist in xtask
- Fail if required commands missing
- Report missing commands clearly

---

#### AC-PLT-016: CI-Local Command

**Contract:**
- `cargo xtask ci-local` orchestrates doctor + selftest + audit + docs-check

**BDD Test:** `@AC-PLT-016`

**How to Maintain:**
- Run full CI validation suite locally
- Replicate CI checks exactly
- Provide clear pass/fail output

---

#### AC-PLT-019: Selftest Summary Display

**Contract:**
- `cargo xtask selftest` displays condensed summary with clear pass/fail indicators for all 8 steps

**BDD Test:** `@AC-PLT-019`

**How to Maintain:**
- Show all 8 selftest steps: core checks, BDD, AC/ADR mapping, bundler, policy, devex contract, graph invariants, AC coverage
- Use clear ✓/✗ indicators
- Provide hint for each failed step

---

#### AC-PLT-020: Low-Resource Mode

**Contract:**
- `XTASK_LOW_RESOURCES=1` environment variable skips resource-intensive steps in selftest
- Suitable for CI/constrained environments

**BDD Test:** `@AC-PLT-020`

**How to Maintain:**
- Skip policy tests when enabled
- Limit build parallelism (CARGO_BUILD_JOBS=1)
- Maintain functional validation

---

#### AC-PLT-017: Status Command

**Contract:**
- `cargo xtask status` displays version, REQ/AC/task counts, selftest status, suggested next tasks

**BDD Test:** `@AC-PLT-017`

**How to Maintain:**
- Parse spec_ledger.yaml for counts
- Check selftest status
- Surface high-priority tasks
- Provide clear dashboard output

---

#### Version Command (Canonical Version Source)

**Contract:**
- `cargo xtask version` displays human-readable kernel version information
- `cargo xtask version --json` provides stable machine-readable JSON output for IDP/agent consumption

**JSON Schema:**

```json
{
  "kernel_version": "3.3.4",       // Required: From spec_ledger.yaml metadata.template_version
  "kernel_tag": "v3.3.4-kernel",   // Required: Git tag format for this kernel
  "schema_version": "1.0",         // Required: Schema version from metadata.schema_version
  "spec_ledger_path": "specs/spec_ledger.yaml",  // Required: Path to canonical spec
  "description": "Rust-as-Spec Platform Cell",   // Required: From metadata.description
  "service_id": "rust-template",   // Optional: From service_metadata.yaml
  "last_updated": "2025-11-30"     // Optional: From metadata.last_updated
}
```

**Why:**
- Provides a stable, machine-readable version endpoint for IDP integrations, CI, and agents
- Single source of truth: always reads from `spec_ledger.yaml` (canonical version authority)
- Part of Docs-as-Code v2: enables tooling to derive version from centralized tracking

**How to Maintain:**
- JSON shape is covered by unit tests in `crates/xtask/src/commands/version.rs`
- Any changes to required fields require test updates and documentation
- Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]` for clean output

---

### 7. LLM Bundles (REQ-TPL-BUNDLE-CONTRACT)

#### AC-TPL-BUNDLE-LAYOUT: Bundle Directory Structure

**Contract:**
- `cargo xtask bundle <TASK>` creates `bundle/<TASK>/` directory with:
  1. `bundle.yaml` – manifest with task_id, requirement_ids, ac_ids, spec sections, docs, and tests
  2. `context.md` – markdown-formatted bundled file contents
  3. Manifest includes `bundle_version`, `git_sha`, and `timestamp` for reproducibility

**Why:**
- Agents and humans need predictable structure to understand task scope
- Manifest enables programmatic bundle discovery and validation
- Reproducibility (git_sha + timestamp) enables traceability

**Bundle Usage:**
- **For agents:** Use manifest to understand task scope, dependencies, and test coverage before taking action
- **For humans:** Use context.md for code review; use manifest for scope validation
- **For CI:** Validate bundle structure and scope against manifest contract

**Important:** Bundle artifacts (`bundle/` directory) are **ephemeral** and ignored by git. They are generated on-demand during development or CI with timestamps and git SHAs baked in. Only the **contract** (specs, ACs, BDD scenarios, tests, docs) is versioned. Bundles should be validated via tests and selftest, not by checking in artifacts.

**BDD Test:** `@AC-TPL-BUNDLE-LAYOUT` in `specs/features/bundles.feature`

**How to Maintain:**
- Ensure `bundle.yaml` is always generated and valid YAML
- Include all referenced specs, docs, and tests in manifest
- Keep git_sha and timestamp accurate for reproducibility

---

#### AC-TPL-BUNDLE-MANIFEST: Manifest Structure and Fields

**Contract:**
- `bundle.yaml` contains required fields:
  - `bundle_version`: Current version (e.g., `1`)
  - `task_id`: Task identifier from `specs/tasks.yaml`
  - `requirement_ids`: List of REQ-* IDs relevant to this task
  - `ac_ids`: List of AC-* IDs this task implements
  - `specs`: Array of spec sections with `file` and optional `line_anchor` or `range`
  - `docs`: Array of referenced documentation with `file` paths
  - `tests`: Array of test handles with `type` (bdd/unit/integration), `tag`, and `file`

**Example manifest structure:**

```yaml
bundle_version: 1
task_id: TASK-TPL-BUNDLE-001
requirement_ids:
  - REQ-TPL-BUNDLE-CONTRACT
ac_ids:
  - AC-TPL-BUNDLE-LAYOUT
  - AC-TPL-BUNDLE-MANIFEST
git_sha: "abc123def456"
timestamp: "2025-11-30T12:34:56Z"
specs:
  - file: "specs/spec_ledger.yaml"
    section: "REQ-TPL-BUNDLE-CONTRACT"
    lines: "1307-1354"
docs:
  - file: "docs/explanation/TEMPLATE-CONTRACTS.md"
    section: "LLM Bundles"
tests:
  - type: bdd
    tag: "@AC-TPL-BUNDLE-LAYOUT"
    file: "specs/features/bundles.feature"
  - type: bdd
    tag: "@AC-TPL-BUNDLE-MANIFEST"
    file: "specs/features/bundles.feature"
```

**BDD Test:** `@AC-TPL-BUNDLE-MANIFEST` in `specs/features/bundles.feature`

**How to Maintain:**
- Manifest is machine-generated from task definition + spec_ledger + task dependencies
- Validate manifest YAML at bundle creation time
- Include accurate line numbers/anchors for spec references

---

#### AC-TPL-BUNDLE-MINIMAL-SCOPE: Bundle Scope Guard

**Contract:**
- Bundle scope audit warns if a bundle exceeds soft thresholds:
  - `~64 files` (default, overridable via `BUNDLE_MAX_FILES`)
  - `~300 KiB` total size (default, overridable via `BUNDLE_MAX_BYTES`)
- Warnings are advisory; bundle generation completes regardless
- Hard failures only occur for manifest/structure issues, not size

**Why:**
- Prevents unbounded context from overwhelming agents/LLMs
- Encourages focused task scoping during bundle definition
- Provides early feedback when task scope may be too broad

**Environment Variables:**
- `BUNDLE_MAX_FILES`: Override default file count threshold (e.g., `128`)
- `BUNDLE_MAX_BYTES`: Override default byte size threshold (e.g., `512000`)

**BDD Test:** `@AC-TPL-BUNDLE-MINIMAL-SCOPE` in `specs/features/bundles.feature`

**How to Maintain:**
- Scope audit runs during `cargo xtask bundle` after manifest generation
- Thresholds are configurable via environment variables for special cases
- Consider splitting large tasks if scope warnings appear frequently

---

### 8. Release Evidence (REQ-TPL-REL-BUNDLE)

#### AC-TPL-REL-EVIDENCE: Release Bundle Generation

**Contract:**
- `cargo xtask release-bundle X.Y.Z` writes `release_evidence/vX.Y.Z.md`
- Contains: completed tasks, linked REQs/ACs/ADRs, git log since last tag, selftest summary, policy status, resolved friction entries

**BDD Test:** `@release_bundle_generation` (integration test)

**How to Maintain:**
- Parse tasks.yaml for completed tasks in this version
- Collect linked REQs/ACs/ADRs
- Run git log since previous tag
- Include selftest and policy status
- Extract resolved friction from FRICTION_LOG.md

---

#### AC-TPL-REL-CHANGELOG: Evidence Structure

**Contract:**
- Evidence file includes distinct sections (Tasks, Specs/ACs, ADRs, Git log, Governance signals)
- Adequate for LLM formatting into Keep a Changelog format

**BDD Test:** `@release_bundle_structure` (integration test)

**How to Maintain:**
- Use clear section headers
- Provide machine-parsable structure
- Include all data needed for changelog generation

---

### 9. Graph Invariants (REQ-TPL-GRAPH-INVARIANTS)

#### AC-TPL-GRAPH-REQ-HAS-AC: Requirements Have ACs

**Contract:**
- Every requirement with `tags` including `platform`, `structural`, `security`, `devex`, `docs`, or `release` has at least one AC node in the graph

**BDD Test:** Unit test `graph_invariants_req_has_ac` in `crates/spec-runtime/src/graph.rs`

**How to Maintain:**
- Validate graph structure at startup
- Check all tagged requirements have ACs
- Report violations clearly

---

#### AC-TPL-GRAPH-AC-HAS-TEST: ACs Have Tests

**Contract:**
- Every AC with a `tests` mapping in `spec_ledger.yaml` has at least one test node linked in the graph

**BDD Test:** Unit test `ac_with_tests_produces_graph_node_and_edge`

**How to Maintain:**
- Validate AC → test linkage
- Check test mappings exist
- Report missing tests

---

#### AC-TPL-GRAPH-COMMAND-REACHABLE: Commands Are Reachable

**Contract:**
- Every command declared in `specs/devex_flows.yaml` is either referenced by a flow or explicitly marked internal
- No orphan commands exist

**BDD Test:** Unit test `graph_invariants_command_reachable`

**How to Maintain:**
- Check all commands are referenced
- Allow explicit `internal: true` marking
- Report orphan commands

---

#### AC-TPL-GRAPH-SELFTEST: Selftest Validates Graph

**Contract:**
- `cargo xtask selftest` validates graph invariants
- Outputs "Graph invariants satisfied" when all checks pass

**BDD Test:** `@AC-TPL-GRAPH-SELFTEST` in `specs/features/graph_invariants.feature`

**How to Maintain:**
- Run graph invariant checks in selftest step 7
- Report violations with actionable details
- Exit non-zero on failure

---

#### AC-TPL-GRAPH-MERMAID: Graph Export

**Contract:**
- `cargo xtask graph-export --format mermaid` emits valid Mermaid graph (graph TD)
- Includes nodes for stories, requirements, ACs, and edges showing relationships

**BDD Test:** Unit test `graph_export_mermaid`, BDD test `@AC-TPL-GRAPH-MERMAID` in `specs/features/graph_visualization.feature`

**How to Maintain:**
- Generate valid Mermaid syntax
- Include all relevant nodes and edges
- Ensure graph is renderable

---

### 10. Configuration and Validation (REQ-TPL-CONFIG-INTEGRITY)

#### AC-TPL-CONFIG-VALIDATION: Startup Configuration Validation

**Contract:**
- On startup the service validates configuration against `specs/config_schema.yaml`
- Exits non-zero with clear validation error when config is invalid

**BDD Test:** `@AC-TPL-CONFIG-VALIDATION`, unit test `config_validation_rejects_invalid` in `crates/spec-runtime/src/config.rs`

**How to Maintain:**
- Load and validate config at startup
- Fail fast on invalid config
- Provide clear error messages with field names and expected values

---

### 11. Infrastructure Alignment (REQ-TPL-IAC-ALIGNMENT)

#### AC-TPL-IAC-K8S-ALIGN: Kubernetes Manifest Alignment

**Contract:**
- Kubernetes manifests under `infra/k8s` (Deployment/Service) use ports and env vars consistent with `specs/config_schema.yaml` and default environment
- Sample IaC aligns with application configuration contract

**Why:**
- Prevents config drift between application and infrastructure
- Ensures example manifests are trustworthy and deployable
- Reduces deployment errors from mismatched configuration

**BDD Test:** Unit test `iac_k8s_aligns_with_config` in `crates/spec-runtime/src/k8s_iac.rs`

**How to Maintain:**
- When adding config keys, update K8s manifests
- Validate manifest ports/env against config_schema.yaml
- Keep environment variables synchronized
- Test manifests in local or staging environment

---

### 12. Git Hooks (REQ-TPL-GOV-HOOKS)

#### AC-TPL-HOOKS-INSTALL: Git Pre-Commit Hooks

**Contract:**
- `cargo xtask install-hooks` creates pre-commit hook
- Hook runs `cargo run -p xtask -- precommit` inside Nix devshell when available
- Failures are advisory and do not block commits

**BDD Test:** `@AC-TPL-HOOKS-INSTALL` in `specs/features/git_hooks.feature`, manual test `git_commit_verify`

**How to Maintain:**
- Install hooks to `.git/hooks/pre-commit`
- Use Nix devshell when available
- Make failures soft (warn but don't block)
- Provide clear output on hook execution

---

### 13. Governance Write Layer (REQ-TPL-GOV-WRITE-001, REQ-TPL-TASK-LIFECYCLE)

#### AC-TPL-GOV-WRITE-TASK-STATUS-200: Task Status Persistence

**Contract:**
- `set_task_status` writes durable state reflected in governance graph

**BDD Test:** `@AC-TPL-GOV-WRITE-TASK-STATUS-200` (integration test)

**How to Maintain:**
- Update task status in `specs/tasks.yaml`
- Preserve human comments in YAML
- Reflect changes in governance graph

---

#### AC-TPL-TASK-TRANSITIONS: Task Status Transitions

**Contract:**
- Task status transitions validated against domain model
- Allowed transitions enforced (e.g., Todo → InProgress)

**BDD Test:** Unit tests `test_allowed_transitions`, `test_forbidden_transitions` in `crates/business-core/src/lib.rs`

**How to Maintain:**
- Define valid state machine in domain model
- Reject invalid transitions
- Provide clear error messages

---

## Selftest Enforcement

The `cargo xtask selftest` command validates all kernel contracts through 12 steps:

1. **Core checks**: fmt, clippy, tests (AC-PLT-003)
2. **Skills governance**: Validate SKILL.md structure and policies
3. **Agents governance**: Validate agent definitions and policies
4. **BDD acceptance tests**: Run all `@AC-*` scenarios
5. **AC/ADR mapping**: AC status and ADR reference validation
6. **LLM bundler**: Test context bundle generation
7. **Policy tests**: Run conftest on all Rego policies
8. **DevEx contract**: Validate required commands exist (AC-PLT-015)
9. **Graph invariants**: Validate governance graph structure (AC-TPL-GRAPH-SELFTEST)
10. **AC coverage**: Ensure all kernel ACs (must_have_ac=true) are passing (AC-PLT-019)
11. **Test coverage**: Advisory check for test coverage floor

### BDD Harness Behaviour (AC-TPL-BDD-EXIT-CODES)

The cucumber harness may return exit code 101 due to async cleanup issues even when all scenarios pass. To shield callers from this flakiness, xtask uses **semantic success detection** rather than raw exit codes.

**xtask treats acceptance as passing if ANY of these conditions are met:**
1. Exit code is 0 (explicit success)
2. Output contains `[BDD-PASS]` marker (harness completed normally)
3. JUnit XML (`target/junit/acceptance.xml`) contains zero failures and zero errors
4. Output contains passing markers (`✔`) with no failure markers (`✗` or `FAILED`)

This behaviour is consistent across:
- `cargo xtask bdd` — direct BDD execution
- `cargo xtask check` — change-aware BDD (routes through `bdd::run_with_options`)
- `cargo xtask selftest` — step 4 (uses `bdd::run()`)
- `cargo xtask test-ac` — AC-specific execution
- `cargo xtask test-changed` — change-aware test execution

**Implementation:** See `crates/xtask/src/commands/bdd.rs` for the canonical `is_bdd_success()` function and helpers.

**CI Enforcement:**
- GitHub Actions runs selftest in Tier-1 job
- Selftest must pass for PRs to merge
- Low-resource mode available via `XTASK_LOW_RESOURCES=1`

---

## CI & Tier-1 Requirements

**All changes to `main` MUST pass Tier-1 selftest before merge.**

The template enforces a **Tier-1 selftest gate** as the canonical quality check for merging to the main branch. This decision is documented in **[ADR-0017: Tier-1 Selftest as Required Gate on Main Branch](../adr/0017-tier1-selftest-gate.md)**.

### What is Tier-1?

**Tier-1** is the hermetic, reproducible development environment powered by Nix:
- **Environment:** Linux (Ubuntu-latest) + Nix devshell
- **Hermetic:** No system dependency leakage
- **Pinned versions:** Exact Rust, conftest, cargo-binstall versions from `flake.nix`
- **Validation:** `nix develop --command cargo xtask selftest`
- **Result:** All 8 selftest phases must pass

### Why Tier-1 is Canonical

1. **Reproducible:** Same environment on every developer's machine and in CI
2. **Platform-independent:** Works identically on Linux, macOS, WSL2
3. **No false positives:** Avoids Windows file locking and other platform-specific issues
4. **Hermetic:** Nix ensures exact tool versions, eliminating "works on my machine" failures
5. **Fast feedback:** Developers can run Tier-1 locally before push (~5-10 minutes)

### Branch Protection

GitHub branch protection enforces this requirement:
- **Required status check:** `tier1-selftest / selftest` must pass ✅
- **No bypass:** PRs cannot merge with red selftest (unless admin override)
- **Branches must be up-to-date:** Ensures cumulative validation

### Developer Workflow

**Before creating a PR:**

```bash
# Enter Tier-1 environment
nix develop

# Run selftest
cargo xtask selftest

# If green, push
git push origin feature-branch
```

**For Windows developers:**
- **Daily iteration:** Use native Windows (Tier-2) for fast feedback
- **Pre-PR validation:** Use WSL2 + Nix (Tier-1) for canonical check
- **Merge decision:** Only Tier-1 CI result matters

### Tier-2 (Native Windows) Status

Tier-2 is **informational, not gating**:
- Fast iteration during development
- May have platform-specific file locking issues (non-deterministic)
- Not used as merge gate

### Emergency Escape Hatch

In rare cases where Tier-1 blocks a critical production fix:
1. **Diagnose:** Is the failure related to the hotfix code?
2. **Escalate:** Admin-level decision required
3. **Document:** Add note to CHANGELOG explaining override
4. **Remediate:** Fix Tier-1 failure in follow-up PR

**Preferred path:** Fix Tier-1 failure first, even if urgent.

### References

- **ADR-0017:** [Tier-1 Selftest as Required Gate on Main Branch](../adr/0017-tier1-selftest-gate.md)
- **ADR-0002:** [Nix-First Development Environment](../adr/0002-nix-first-dev-env.md)
- **ADR-0005:** [Selftest as the Single Quality Gate](../adr/0005-xtask-selftest-single-gate.md)
- **Platform Support:** [Platform Support Reference](../reference/platform-support.md)
- **CI Workflow:** `.github/workflows/tier1-selftest.yml`

---

## Service Metadata Contract

`specs/service_metadata.yaml` must include:

- `service_id`: Unique service identifier
- `template_version`: Template version (e.g., "3.3.1")
- `ownership`: Team, email, slack
- `lifecycle`: Tier, data_class, criticality, languages, runtime
- `links`: roadmap, kernel_contract, agent_guide, feature_status, support, ui, status, repo
- `tags`: Service tags for categorization

This metadata is exposed via `/platform/status` and used by the UI.

---

## Version Manifest Contract

`specs/version_manifest.yaml` declares all version-bearing files in the repository. This manifest is the single source of truth for release automation.

**Key Properties:**
- `schema_version`: Manifest schema version
- `files`: Array of version targets with paths and patterns

**Version-Bearing Files (10+):**
- `specs/spec_ledger.yaml` - Canonical version source (priority 1)
- `specs/service_metadata.yaml` - Service metadata version
- `specs/doc_index.yaml` - Documentation index version
- `README.md` - Repository README version references
- `CLAUDE.md` - Agent instructions version
- `CHANGELOG.md` - Release history
- `docs/KERNEL_SNAPSHOT.md` - Kernel baseline version
- `docs/ROADMAP.md` - Roadmap version references
- `docs/explanation/TEMPLATE-CONTRACTS.md` - Template version

**Related ACs:**
- `AC-TPL-VERSION-MANIFEST`: Version locations declared in manifest
- `AC-TPL-VERSION-DRYRUN`: Dry-run preview before applying
- `AC-TPL-VERSION-ATOMIC`: Atomic updates with rollback

**Enforcement:**
- `cargo xtask docs-check` validates version alignment across files
- `cargo xtask release-prepare X.Y.Z` uses manifest for updates (v3.3.6+)

---

## Version Strings in Documentation

All version strings in documentation fall into one of three categories:

### 1. Governed Versions (Auto-Updated)

Template/kernel versions (`3.3.x`, `v3.3.x`, `v3.3.x-kernel`) are:
- Declared in `specs/version_manifest.yaml`
- Updated automatically by `cargo xtask release-prepare X.Y.Z`
- Validated by `cargo xtask docs-check` orphan-version lint

**Files covered:**
- `specs/spec_ledger.yaml` (canonical source)
- `README.md`, `CLAUDE.md` (primary docs)
- `docs/KERNEL_SNAPSHOT.md`, `docs/ROADMAP.md` (kernel docs)
- `docs/explanation/TEMPLATE-CONTRACTS.md`, `docs/explanation/json-contracts.md`
- `docs/testing-strategy.md`, `docs/reference/ci-workflows.md`
- `docs/SKILLS_GOVERNANCE.md`, `docs/AGENTS_GOVERNANCE.md`

### 2. Historical Versions (Suppressed)

Old versions in archived documentation are intentionally preserved:
- Marked with `<!-- doclint:disable orphan-version -->` at file top
- Listed in `version_manifest.yaml` under `historical_docs`
- Not updated by release automation

**Examples:**
- `docs/v2.1.0-plan.md`, `docs/v2.2.0-plan.md` (completed release plans)
- `docs/TECHNICAL-FREEZE-COMPLETE.md` (historical checkpoint)
- `CHANGELOG.md` (complete version history)

### 3. Pinned External Versions

External dependency versions (rustc, conftest, wasmtime) are:
- Not tied to template version
- Either governed separately or intentionally pinned
- Documented in context (e.g., "uses conftest 0.52.0 from nixpkgs")

**Handling orphan versions:**
1. **Governed version stale?** → Update with `release-prepare` or manually
2. **Historical doc?** → Add `<!-- doclint:disable orphan-version -->` at top
3. **External dep?** → Document version source, suppress if needed

---

## IDP Snapshot Contract

The IDP snapshot command provides a stable, machine-readable JSON output for Internal Developer Portal integration.

### Contract Requirements

**Requirement:** REQ-TPL-IDP-SNAPSHOT
**ACs:** AC-TPL-IDP-SNAPSHOT, AC-TPL-IDP-SNAPSHOT-VALID-JSON

### Output Schema

The `cargo xtask idp-snapshot` command outputs JSON with these guaranteed fields:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | string | ISO 8601 timestamp of snapshot generation |
| `template_version` | string | Semantic version from spec_ledger.yaml |
| `service_id` | string | Service identifier from service_metadata.yaml |
| `governance_health` | object | Status (healthy/degraded/failing), AC coverage metrics |
| `documentation` | object | Doc counts (total, valid, with_issues) |
| `task_hints` | object | Pending/in_progress counts, high-priority task hints |

### Usage

```bash
# Generate snapshot to stdout
cargo xtask idp-snapshot

# Pretty-print for debugging
cargo xtask idp-snapshot --pretty

# Write to file
cargo xtask idp-snapshot --output idp-snapshot.json
```

### IDP Integration

This output is designed for:
- **Backstage**: Import via custom entity provider or catalog info
- **Port.io**: Ingest via Python/Ruby scripts using the blueprint schema
- **Custom IDPs**: Parse JSON directly for governance tiles

---

## Customization Surface (SAFE TO CHANGE)

### Domain Model and Business Logic

- Add new domain types, services, use cases
- Create domain-specific error codes
- Implement domain-specific handlers
- **Preserve:** ErrorResponse structure, Health/Version types

### HTTP Routes and Middleware

- Add new domain API endpoints
- Create custom middleware (auth, rate limiting, logging)
- **Preserve:** `/health`, `/version`, `/metrics`, `/platform/*` endpoints, request ID middleware

### Acceptance Criteria and Features

- Add new domain ACs to `spec_ledger.yaml`
- Create new `.feature` files for domain scenarios
- **Preserve:** Template-core ACs (AC-TPL-*, AC-PLT-*), AC structure from policies

### Configuration and Infrastructure

- Add new config keys to `config_schema.yaml`
- Customize K8s manifests (within policy bounds)
- Add new environment variables
- **Preserve:** Required config schema structure, multi-env K8s structure

### Dependencies

- Add new crates for domain features
- Update dependency versions
- **Preserve:** Core dependencies (axum, tower, serde, tokio), testing dependencies (cucumber)

### Tasks and Workflows

- Add domain-specific tasks to `specs/tasks.yaml`
- Create custom workflows in `specs/devex_flows.yaml`
- **Preserve:** Task schema structure, flow definitions

---

## Non-Kernel Requirements (Informational)

Requirements and ACs marked with `must_have_ac: false` or `tags: [future]` are:

- **Not enforced by selftest AC coverage gate**
- Informational only
- May be incomplete or experimental
- Safe to implement incrementally

Examples:
- AC-TPL-PLATFORM-AUTH-BASIC: Platform authentication (future)
- AC-TPL-LOG-NO-SECRETS: Log hygiene (future)
- AC-TPL-SUGGEST-NEXT-CLI: Suggest-next command (future)
- AC-TPL-TASKS-CLI: Tasks-list command (future)

These provide guidance for future work but do not block releases.

---

## FAQ

**Q: Can I rename the `/health` endpoint to `/healthz`?**
A: No. This is a kernel contract enforced by AC-TPL-001. K8s manifests and policies reference `/health`.

**Q: Can I remove the ErrorResponse type and use my own error format?**
A: No. AC-TPL-003 enforces the error envelope structure. Extend it instead.

**Q: Can I delete BDD features if I don't use BDD?**
A: No. Template-core scenarios must exist and pass. You can skip writing NEW features, but kernel ACs must have BDD tests.

**Q: Can I add a new xtask command like `cargo xtask migrate`?**
A: Yes! Adding new commands is safe. Add to `specs/devex_flows.yaml` and implement in xtask.

**Q: Can I change the UI framework or replace the UI entirely?**
A: Yes, as long as you maintain the kernel contracts: dashboard showing status (AC-TPL-PLATFORM-UI-DASHBOARD), graph visualization (AC-TPL-PLATFORM-UI-GRAPH), and flows/tasks view (AC-TPL-PLATFORM-UI-FLOWS).

**Q: What if I find a kernel contract too restrictive?**
A: Open an issue in the template repo. We can discuss relaxing it in a future version. Do not silently break it.

**Q: How do I know if I've broken a kernel contract?**
A: Run `cargo xtask selftest`. If it passes, you haven't broken kernel contracts.

---

## Test Diversity Roadmap

**Current State (v3.3.1):** Most kernel ACs have **single-test coverage** (1 test per AC). This is acceptable for initial implementation but represents a test diversity gap.

**Best Practice:** Kernel ACs should have **2-3 test scenarios** covering:
- Happy path (primary scenario)
- Error conditions or edge cases
- Different input combinations or contexts

**Priority Phases:**

### Phase 1: Critical Security & Release (v3.4.0)

- AC-PLT-006, AC-PLT-007, AC-PLT-008 (security audit)
- AC-PLT-011, AC-PLT-012, AC-PLT-013 (release management)
- Target: Add 1 unit test per BDD scenario for validation logic

### Phase 2: DevEx Foundation (v3.5.0)

- AC-PLT-001, AC-PLT-002, AC-PLT-003 (doctor, help-flows, check)
- Target: Add error-case BDD scenarios

### Phase 3: Agent Interface (v3.6.0)

- AC-TPL-SKILLS-FMT, AC-TPL-SKILLS-LINT, AC-TPL-SKILLS-ALIGN-001
- Target: Add unit tests for validation logic

**See:** `docs/feature_status_notes.md` for detailed roadmap and rationale.

---

## Summary

**Kernel Contracts (must preserve):**
- HTTP service core endpoints and behaviors (AC-TPL-001 through AC-TPL-007)
- Platform introspection APIs (AC-TPL-PLATFORM-*)
- Platform UI components (AC-TPL-PLATFORM-UI-*)
- Agent interface (AC-TPL-AGENT-SKILLS, AC-TPL-QUESTIONS-LOGGED, AC-TPL-FLOW-IDEMPOTENT, AC-TPL-SKILLS-*)
- DevEx xtask commands (AC-PLT-001 through AC-PLT-020)
- Release evidence generation (AC-TPL-REL-*)
- Graph invariants (AC-TPL-GRAPH-*)
- Configuration validation (AC-TPL-CONFIG-*)
- Infrastructure alignment (AC-TPL-IAC-K8S-ALIGN)
- Git hooks (AC-TPL-HOOKS-*)
- Governance write layer (AC-TPL-GOV-WRITE-*, AC-TPL-TASK-*)

**Enforcement:**
- `cargo xtask selftest` validates all kernel contracts
- CI runs selftest on every PR
- Step 8 (AC coverage) ensures all `must_have_ac: true` ACs are passing

**Customization:**
- Domain models, business logic, routes
- New ACs, features, tests
- Configuration, infrastructure, dependencies
- Tasks, workflows, documentation

**When in doubt:** Run `cargo xtask selftest`. If it passes, you're good.

---

## References

- **Spec Ledger**: `specs/spec_ledger.yaml`
- **DevEx Flows**: `specs/devex_flows.yaml`
- **Service Metadata**: `specs/service_metadata.yaml`
- **Tasks**: `specs/tasks.yaml`
- **Selftest Implementation**: `crates/xtask/src/commands/selftest.rs`
- **Graph Invariants**: `crates/spec-runtime/src/graph.rs`
- **Agent Guide**: `docs/AGENT_GUIDE.md`
- **Skills Guide**: `docs/AGENT_SKILLS.md`
