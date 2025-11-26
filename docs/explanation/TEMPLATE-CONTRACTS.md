# Template Contracts: Kernel Requirements and Extension Points

**Template Version:** v3.3.1
**Schema Version:** spec_ledger.yaml v1.0
**Last Updated:** 2025-11-26

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
- `GET /platform/schema` returns JSON schema/OpenAPI document
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

#### AC-TPL-AGENT-HINTS: Agent Task Hints
**Contract:**
- `GET /platform/agent/hints` returns prioritized task suggestions
- Filters tasks by Todo/InProgress status
- Each hint includes: `task_id`, `status`, `requirement_ids`, `ac_ids`, `reason`, `recommended_sequence` (array of commands/edits)

**Why:**
- Provides agents with actionable next steps
- Reduces agent decision-making overhead
- Surfaces high-priority work items

**BDD Test:** `@AC-TPL-AGENT-HINTS`

**How to Maintain:**
- Hints generated from `specs/tasks.yaml` and task status
- Include REQ/AC IDs and recommended command sequences
- Keep synchronized with task state

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
- `cargo xtask docs-check` validates version alignment across `spec_ledger.yaml`, `README.md`, `CLAUDE.md`

**BDD Test:** `@AC-PLT-009`

**How to Maintain:**
- Parse version from all sources
- Fail if versions don't match
- Provide clear guidance for fixing mismatches

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

### 7. Release Evidence (REQ-TPL-REL-BUNDLE)

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

### 8. Graph Invariants (REQ-TPL-GRAPH-INVARIANTS)

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

### 9. Configuration and Validation (REQ-TPL-CONFIG-INTEGRITY)

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

### 10. Git Hooks (REQ-TPL-GOV-HOOKS)

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

### 11. Governance Write Layer (REQ-TPL-GOV-WRITE-001, REQ-TPL-TASK-LIFECYCLE)

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

The `cargo xtask selftest` command validates all kernel contracts through 8 steps:

1. **Core checks**: fmt, clippy, tests (AC-PLT-003)
2. **BDD acceptance tests**: Run all `@AC-*` scenarios
3. **AC/ADR mapping**: AC status and ADR reference validation
4. **LLM bundler**: Test context bundle generation
5. **Policy tests**: Run conftest on all Rego policies
6. **DevEx contract**: Validate required commands exist (AC-PLT-015)
7. **Graph invariants**: Validate governance graph structure (AC-TPL-GRAPH-SELFTEST)
8. **AC coverage**: Ensure all kernel ACs (must_have_ac=true) are passing (AC-PLT-019)

**CI Enforcement:**
- GitHub Actions runs selftest in Tier-1 job
- Selftest must pass for PRs to merge
- Low-resource mode available via `XTASK_LOW_RESOURCES=1`

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

## Summary

**Kernel Contracts (must preserve):**
- HTTP service core endpoints and behaviors (AC-TPL-001 through AC-TPL-007)
- Platform introspection APIs (AC-TPL-PLATFORM-*)
- Platform UI components (AC-TPL-PLATFORM-UI-*)
- Agent interface (AC-TPL-AGENT-*, AC-TPL-SKILLS-*)
- DevEx xtask commands (AC-PLT-001 through AC-PLT-020)
- Release evidence generation (AC-TPL-REL-*)
- Graph invariants (AC-TPL-GRAPH-*)
- Configuration validation (AC-TPL-CONFIG-*)
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
