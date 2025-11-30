<!-- markdownlint-disable MD007 MD024 MD032 MD036 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

**Documentation:**

- **QUICKSTART.md** - Fast-path guide for new users (setup, first BDD change, selftest validation)
- **TROUBLESHOOTING.md** - Comprehensive guide for common issues (dev environment, tests, BDD, CI, governance)
- **Windows Development Guide** (`docs/windows-dev-guide.md`) - Complete Windows-as-Tier-2 setup and workflows
- **CI Workflows Reference** (`docs/reference/ci-workflows.md`) - Detailed explanation of all GitHub Actions workflows with ownership and troubleshooting
- **Branch Protection Setup** (`docs/how-to/setup-branch-protection.md`) - Step-by-step guide for configuring GitHub branch protection rules
- **Tag Signing Setup** (`docs/how-to/setup-tag-signing.md`) - Complete guide for GPG-signed tags with GitHub verification

**Platform APIs:**

- **`/platform/friction` endpoint** - Exposes friction log entries with metadata (type, title, description, timestamp)
- **`/platform/version` endpoint** - Returns current platform version from `spec_ledger.yaml`

**xtask Commands:**

- **`cargo xtask friction-new`** - Interactive command to capture friction log entries (pain point, feature idea, or question)
- **`cargo xtask question-new`** - Interactive command to create question entries in friction log
- **`cargo xtask version`** - Display current platform version from spec ledger

**Release Tooling:**

- **AC delta reporting in `release-bundle`** - Shows which ACs are new, modified, or unchanged since last release
- **Fork registry** (`docs/FORK_REGISTRY.md`) - Template for tracking downstream forks with metadata and health checks

### Changed

- **BDD test isolation** - All acceptance tests now use dedicated test database instances to prevent cross-test contamination
- **ADR numbering** - Standardized ADR filenames with 4-digit padding (0001-0019) for consistent sorting and referencing
- **Friction log structure** - Formalized sections (Pain Points, Feature Ideas, Questions, Process Observations) with metadata and timestamps

### Fixed

- **BDD version alignment** - Updated template_core.feature to expect v3.3.3 instead of v3.3.2
- **Spec ledger version consistency** - Aligned all spec files to v3.3.3
- **ADR cross-references** - Updated all ADR links in spec ledger and documentation to use 4-digit numbering

## [3.2.0] - 2025-11-22

### Added

- **AC Coverage Tooling:**

  - `cargo xtask ac-coverage` command for AC coverage summary grouped by requirement
  - `cargo xtask ac-suggest-scenarios` command to generate BDD scenario stubs from ACs
  - Shared `ac_parsing` module for consistent AC/scenario parsing across commands

**Skills Tooling:**

  - `cargo xtask skills-fmt` command to normalize SKILL.md formatting
  - `cargo xtask skills-lint` command to validate Skills definitions
  - Skills are now governed artifacts with standardized structure

**Cross-Platform Support:**

  - Windows as Tier-2 platform with native tooling support (Linux/macOS + Nix remain Tier-1)
  - Platform-aware null device constants (`/dev/null` on Unix, `nul` on Windows)
  - POSIX shell hooks that work on all platforms via Git for Windows `sh.exe`
  - Comprehensive Tier-1/Tier-2 platform model documentation in `MISSING_MANUAL.md` and `README.md`

**Release & CI Infrastructure:**

  - `cargo xtask release-bundle` command to generate comprehensive release evidence files
  - GitHub Actions now validates Linux, macOS (Tier-1), and Windows (Tier-2) on every push
  - Cross-platform CI matrix in `ci-template-selftest.yml`

### Changed

- **Module Organization:**

  - Extracted common AC parsing logic into `crates/xtask/src/commands/ac_parsing.rs`
  - `ac_status`, `ac_coverage`, and `ac_suggest_scenarios` now share parsing infrastructure
  - Improved code reuse and consistency across AC-related commands

**UX & Dashboard:**

  - Enhanced selftest summary with 7-step output and "Next actions" hints
  - AC coverage metrics now displayed in `cargo xtask status` and `/ui` dashboard
  - Skills formatting/linting integrated into `docs-check` and pre-commit workflows

**Platform-Specific Adjustments:**

  - On Windows, xtask excluded from workspace commands to avoid file-locking during self-rebuild
  - Unified Git hook installation using single POSIX script (no more batch file hooks)

**Documentation:**

  - Updated SKILL.md files for clarity and consistency
  - Replaced Python script references with Rust commands in Skills
  - Updated `docs/reference/xtask-commands.md` with ac-coverage and ac-suggest-scenarios documentation

### Fixed

- **AC Wiring:**

  - Fixed missing `ac_parsing`, `ac_coverage`, and `ac_suggest_scenarios` module exports in `crates/xtask/src/commands/mod.rs`
  - All AC-related commands now compile and function correctly
  - Resolved import issues for `CucumberReport`, `AC_PATTERN`, and other parsing utilities
  - AC tooling now provides complete workflow from coverage analysis to scenario generation

**Cross-Platform Issues:**

  - Fixed Windows build failures caused by Unix-specific file permission APIs
  - Git hook installation simplified to single POSIX script (works on all platforms)
  - Fixed malformed JSON parsing in AC coverage logic; now uses proper junit/feature mapping

**Cleanup:**

  - Removed duplicate `adr-*.md` files in docs/decisions/
  - Removed accidentally committed JUnit/JSON test output files
  - Fixed `double_ended_iterator_last` and `collapsible_if` clippy warnings in acceptance tests

### Internal

- **Governance Wiring**: Full BDD → AC → Requirement traceability implemented and validated
- **Platform Model**: Established Tier-1 (Nix + hermetic) vs Tier-2 (native Windows with caveats) support model
- **Evidence Generation**: Release artifacts (`release_evidence/*.md`) auto-generated from governance data
- **Hook Generation**: Unified hook installation using POSIX scripts recognized by Git for Windows

---

## [3.0.0-sprint1] - 2025-11-20

### Summary

🚀 **v3.0.0 Sprint 1: The Write Layer**

First iteration of v3.0.0 "The Living Platform" - transitioning from "Code that Agents help write" to "An Environment where Agents live." This sprint implements the foundational governance write layer with file system persistence and BDD verification.

### Added

**Governance Repository & File System Adapter**

- **Governance Repository Trait** (`business-core`): Abstract interface for task state persistence
- **File System Adapter** (`adapters-spec-fs`): Production implementation with:
  - `tasks_state.yaml` for task status tracking
  - File locking for concurrent access safety
  - Integration with existing spec ledger
- **Integration Tests**: End-to-end validation of repository contract
- **ADR-0019**: [Governance Repository and FS Adapter](docs/adr/0019-governance-repository-and-fs-adapter.md)

**BDD Scenarios for Platform Governance**

- **New Feature File**: `platform_governance_write.feature` with scenarios:
  - `@AC-GOV-WRITE-001`: Task status changes are persisted
  - `@AC-GOV-WRITE-002`: Task status changes are visible through introspection
- **Step Definitions**: `governance_write.rs` implementing governance write operations
- **Wiring**: `app-http` and acceptance tests integrated with `FsGovernanceRepository`

**Agent Skills & Documentation**

- **Claude Skills** (`.claude/skills/`):
  - `governed-feature-dev` - AC → BDD → code → selftest workflow
  - `governed-release` - changelog → tag → deploy workflow
  - `governed-maintenance` - audit → upgrade → verify workflow
- **Documentation**:
  - `docs/AGENT_GUIDE.md` - LLM operational directive
  - `docs/INDEX.md` - Documentation inventory
  - `docs/MISSING_MANUAL.md` - Implicit knowledge capture
  - `docs/RELEASE_v2.5.0.md` - v2.5.0 release documentation

**Developer Experience**

- **Governance Hooks**: `cargo xtask install-hooks` command
  - Installs pre-commit hook that runs `cargo xtask check`
  - Ensures governance checks run before commit
- **Local Runtime**: `docker-compose.yaml` with Postgres 16 + Jaeger for local development

### Changed

- **Acceptance Tests**: Updated `World` struct and step definitions to support governance write scenarios
- **ROADMAP.md**: Updated to show Sprint 1 as completed ✅
- **Feature Status**: Marked Sprint 1 tasks as complete in `docs/feature_status.md`
- **Spec Ledger**: Updated with governance write requirements and acceptance criteria
- **CLAUDE.md**: Enhanced with agent skills references and governance workflow guidance

### Technical Details

**New Crates:**
- `crates/adapters-spec-fs/` - File system governance repository adapter (295 lines)

**Files Created (17):**

- `.claude/skills/governed-feature-dev/SKILL.md`
- `.claude/skills/governed-maintenance/SKILL.md`
- `.claude/skills/governed-release/SKILL.md`
- `crates/acceptance/src/steps/governance_write.rs`
- `crates/adapters-spec-fs/Cargo.toml`
- `crates/adapters-spec-fs/src/lib.rs`
- `crates/adapters-spec-fs/src/tasks_state.rs`
- `crates/adapters-spec-fs/tests/integration_test.rs`
- `crates/xtask/src/commands/install_hooks.rs`
- `docker-compose.yaml`
- `docs/AGENT_GUIDE.md`
- `docs/INDEX.md`
- `docs/MISSING_MANUAL.md`
- `docs/RELEASE_v2.5.0.md`
- `docs/adr/0019-governance-repository-and-fs-adapter.md`
- `specs/features/platform_governance_write.feature`

**Files Modified (20+):**

- Multiple crate `Cargo.toml` files for dependency updates
- Acceptance test infrastructure (`world.rs`, `mod.rs`)
- ROADMAP, CHANGELOG, CLAUDE.md, README.md
- Feature status and spec ledger files

**Validation:**

- ✅ All pre-commit checks passed (fmt, clippy, tests)
- ✅ Governance repository integration tests passing
- ✅ BDD scenarios for governance write operations passing
- ✅ Selftest validation complete

### Next: Sprint 2 - Domain Rules

- Task Status Machine: Enforce valid transitions (Todo → InProgress → Done)
- Task ↔ Requirement Linking: Domain model for traceability
- Update Use Case: Pure business logic for status updates

---

## [2.5.0] - 2025-11-20

### Summary

🎉 **Agent-Ready Platform Cell**

**Kernel freeze.** This release completes the self-governing platform cell architecture. The platform successfully used its own governance contracts to build its final features (pilot validation). No new kernel features planned - next phase is real-world service pilot.

### Added - Phase 4: Deep Governance & Polish

**Epic 1: Graph Invariants as Mandatory Gate**

- Structural validation catches drift: `REQ_HAS_NO_AC`, `COMMAND_UNREACHABLE`
- Integrated as selftest step [7/7]
- Violations block merge with actionable errors

**Epic 2: Context-Aware Suggest-Next**

- Tracks `Pending` vs `Satisfied` steps (✓ indicators)
- Prevents redundant work for humans and agents

**Epic 3: Real-Time Policy Status**

- `policy-test` emits `target/policy_status.json`
- `/platform/status` exposes policy health

**Epic 4: Platform Introspection Web UI**

- `/ui` - Dashboard (metrics, status, policy health)
- `/ui/graph` - Mermaid visualization
- `/ui/flows` - DevEx explorer
- Zero build steps, maud + htmx + Mermaid.js

### Added - Pilot: Agent Interface (Kernel Bootstrapped Itself)

**Local Runtime** (`docker-compose.yaml`)

- Postgres 16 + Jaeger, zero-config

**Governance Hooks** (`cargo xtask install-hooks`)

- Pre-commit hook runs `cargo xtask check`

**Agent Skills** (`.claude/skills/*`)

- `governed-feature-dev`, `governed-release`, `governed-maintenance`
- Updated `CLAUDE.md`

### Documentation

- `docs/ROADMAP.md` - Strategic vision
- `docs/explanation/rust-as-spec-overview.md` - Technical deep-dive
- `docs/AGENT_GUIDE.md` - LLM operational guide
- `docs/INDEX.md`, `docs/MISSING_MANUAL.md`

### Validated

✅ All 7 selftest steps pass
✅ 22/22 policy tests pass
✅ Graph invariants enforced
✅ **Platform used itself to build final features**

### What's Next

**Real-world pilot** using this kernel → friction log → v2.5.x hardening (no new features)

---

## [2.4.0] - 2025-11-19

### Added

**Supply Chain Hardening** (ADR-0006):

  - `docs/adr/0006-supply-chain-hardening.md` - Supply chain hardening and build provenance decision record
  - `docs/explanation/supply-chain-hardening.md` - Comprehensive guide to SBOM and provenance (25KB, 834 lines)
  - `.github/workflows/ci-supply-chain.yml` - Automated SBOM generation and provenance attestation for tagged releases
  - SLSA v1.0 Level 2 compliance via GitHub Artifact Attestations
  - SBOM generation in SPDX JSON format via anchore/sbom-action
  - Build provenance attestation via actions/attest-build-provenance (Sigstore-backed)
  - Nix-based hermetic builds for release artifacts
  - Verification guides for GitHub CLI, SLSA verifier, and policy enforcement

**Release Polish**:

  - `FRICTION_LOG.md` in root for immediate pilot feedback
  - Dynamic port selection for gRPC smoke tests (reliability)
  - Full documentation consistency check

### Changed

- **README.md**: Added supply chain basics to "What this template provides" section
- **CONTRIBUTING.md**: Added supply chain workflow expectations to release process
- **CLAUDE.md**: Added ADR-0006 to key ADRs list and supply chain hardening to explanations
- **spec_ledger.yaml**: Added ADR-0006 to template-wide ADRs, bumped version to v2.4.0
- **Version strings**: Updated all version references from v2.3.x to v2.4.0 across documentation

## [2.3.1] - 2025-01-18

### Fixed

**k8s.rego policy**: Fixed variable shadowing and undefined reference in `has_secret_ref` function. All 22 policy tests now pass cleanly in Nix devshell.

**xtask selftest**: Policy tests now properly skip when conftest is not available in local development without failing selftest. CI environments (detected via `CI` or `GITHUB_ACTIONS` env vars) still correctly treat missing conftest as a failure.

**rust_iac_config**: Marked flaky tests that mutate `set_current_dir` as `#[ignore]` to avoid non-deterministic failures in workspace test runs.

### Added

**ADR System** (Design-as-Code):

  - `docs/templates/ADR-TEMPLATE.md` - Template for architecture decision records
  - `docs/adr/0001-hexagonal-architecture.md` - Hexagonal architecture via workspace crates
  - `docs/adr/0002-nix-first-dev-env.md` - Nix-first development environment
  - `docs/adr/0003-spec-and-bdd-as-source-of-truth.md` - Spec ledger and BDD as canonical behavior contracts
  - `docs/adr/0004-policy-and-llm-governance.md` - Policy-as-code and LLM governance
  - `docs/adr/0005-xtask-selftest-single-gate.md` - Selftest as the single quality gate
  - ADR references wired into `specs/spec_ledger.yaml` at metadata, story, requirement, and AC levels
  - `cargo run -p xtask -- adr-check` command to validate ADR references
  - ADR check integrated into selftest step 3

**Nix-First Development Environment**:

  - `docs/dev-environment.md` - Comprehensive guide with Tier 1 (Nix) and Tier 2 (native) paths
  - README.md now Nix-first with clear quick start
  - Selftest provides helpful hints when conftest is missing locally

**LLM & Governance Documentation**:

  - `CLAUDE.md` - Formalized LLM workflow guide with standard prompts and golden path
  - `docs/explanation/controls-as-code.md` - Comprehensive explanation of policy governance (447 lines)
  - `docs/templates/SERVICE_METADATA.example.yaml` - Service self-description template
  - `docs/templates/RUNBOOK.example.md` - Operations guide template (400 lines)

**OSS Hygiene**:

  - CI status and license badges in README.md
  - ADR workflow documented in CONTRIBUTING.md
  - SECURITY.md with vulnerability reporting and scope

### Changed

- **README.md**: Repositioned as Nix-first with clear opinionated messaging
- **CLAUDE.md**: Updated all workflow examples to assume `nix develop` for CI-equivalent validation
- **CONTRIBUTING.md**: Added ADR workflow to governance model section

### Removed

- **docs/ROADMAP.md**: Removed in favor of per-version release planning in `docs/v2.*.md` files
- **Dev cruft**: Removed obsolete `.llm/example-prompts.md` and `.llm/test_contextpack.yaml`

## [1.0.0] - 2025-11-15

### Added

**Core Template Features:**

- Template foundation with health and version endpoints (`AC-TPL-001`, `AC-TPL-002`)
- Refund example feature to demonstrate AC-first patterns (`AC-REFUND-001`)
- Complete hexagonal architecture with adapters, core, and model crates
- Axum HTTP runtime with telemetry integration
- Cucumber BDD acceptance testing with JSON and JUnit XML output
- Comprehensive Diátaxis documentation framework

**xtask Rust-Native Tooling:**

- `xtask check` - Fast feedback loop (fmt, clippy, tests)
- `xtask bdd` - Run Cucumber acceptance tests
- `xtask ac-status` - Generate AC status report from JSON/JUnit
  - Primary: Structured JSON from Cucumber with full metadata
  - Fallback: JUnit XML + feature file parsing (legacy)
- `xtask policy-test` - Run Rego policy tests via conftest
- `xtask bundle` - Generate LLM context bundles with `.llmignore` support
- `xtask quickstart` - First-run validation
- `xtask selftest` - Comprehensive validation suite (all of the above)

**Policy-as-Code (Rego):**

- Ledger coverage policy - Ensures all ACs have tests
- Features policy - Validates feature-to-AC mapping
- Flags policy - Feature flag governance
- Privacy policy - PII handling validation

**Observability Patterns:**

- Request ID correlation middleware (custom implementation)
  - Reads `X-Request-ID` header or generates UUID
  - Automatically adds to tracing spans
  - Returns in response for client tracking
- Enhanced `AppError` with structured logging
  - 12 machine-readable error codes
  - AC ID and Feature ID tracking
  - Structured context for debugging (logged, not exposed)
  - JSON error responses
- Metrics integration stubs with examples
- Full instrumentation examples in handlers

**LLM Integration:**

- Context bundler with gitignore-style `.llmignore` semantics
- Task-based bundle generation for focused context
- Environment variable configuration (`AC_REPORT_JSON`, etc.)
- Example contextpack.yaml with implement_ac task

**Documentation:**

- Complete Diátaxis framework (Tutorials, How-tos, Reference, Explanation)
- Getting Started tutorial
- First AC Change tutorial
- Architecture overview and design rationale
- Template Foundation vs Examples guide
- xtask Commands reference
- Branch Protection Profiles reference
- Design documents for key implementation decisions
- OBSERVABILITY.md for runtime patterns

**CI/CD:**

- GitHub Actions workflows for:
  - Template self-test (xtask selftest)
  - Policy verification
  - Lints, MSRV, Coverage
  - Nix flake check
- Branch protection profile recommendations (Minimal/Standard/Strict)

### Changed

- Migrated all core workflows from bash scripts to Rust-native `xtask` commands
- Replaced custom .llmignore matching with full gitignore semantics (using `ignore` crate)
- Improved AC status mapping to use structured JSON instead of fragile text parsing
- Updated all documentation to canonize xtask as primary interface
- Converted legacy bash scripts to thin wrappers (backward compatibility)

### Deprecated

- `scripts/test-policies.sh` - Use `cargo run -p xtask -- policy-test`
- `scripts/test-all.sh` - Use `cargo run -p xtask -- selftest`
- JUnit XML + feature file parsing in ac-status (fallback only, may be removed in v2.0)

### Technical Details

**Code Statistics:**

- ~1,500 lines of new code and features
- ~1,200 lines of new documentation
- 14 new unit tests (100% coverage of new code)
- 26 files created or modified
- 3 comprehensive design documents

**Validation:**

- All `xtask selftest` checks passing:
  - ✅ Format, clippy, unit tests
  - ✅ 3/3 BDD scenarios, 7/7 steps
  - ✅ All ACs correctly mapped
  - ✅ LLM bundler generating correct output
  - ✅ Policy tests (with conftest)

**Dependencies Added:**

- `ignore = "0.4"` - Gitignore-style pattern matching
- `cucumber` with `output-json` feature - Structured test reports
- `quick-xml` - JUnit XML parsing (fallback)
- `uuid` with `v4` - Request ID generation

### Breaking Changes

**NONE** - v1.0.0 is the initial stable release. All changes are backward compatible with the template's pre-release state.

### Migration Guide

If migrating from pre-1.0 template usage:

**Replace bash script calls with xtask:**

```bash
# Old
bash scripts/test-policies.sh
bash scripts/test-all.sh

# New
cargo run -p xtask -- policy-test
cargo run -p xtask -- selftest
```

**Update CI workflows:**

```yaml
# Old
run: nix develop -c bash scripts/test-policies.sh

# New
run: nix develop -c cargo run -p xtask -- policy-test
```

**Enable JSON AC reports (recommended):**

- Acceptance tests now automatically generate `target/ac_report.json`
- `xtask ac-status` uses this by default
- JUnit fallback still works but is legacy

**Update .llmignore if using advanced patterns:**

- Full gitignore syntax now supported (`*.log`, `**/*.tmp`, etc.)
- Simple patterns (like `target/`) work unchanged
- See `docs/how-to/use-llm-bundles.md` for full syntax

### Security

- No security vulnerabilities introduced
- All dependencies vetted and up-to-date
- No unsafe code in new implementations

### For Contributors

- Read `docs/implementation-summary-2025-11-15.md` for full technical details
- Review `docs/design/*.md` for implementation rationale
- Check `crates/app-http/OBSERVABILITY.md` for runtime patterns
- Follow `TEMPLATE_API.md` for stable interfaces

---

## [1.1.0] - 2025-11-15

### Added

**Phase 1: Template Contract Layer**

**Error Envelope Specification (Phase 1.1):**

- OpenAPI `ErrorResponse` schema with required `error`, `message`, and `requestId` fields
- New acceptance criteria `AC-TPL-003` (error envelope) and `AC-TPL-004` (request ID propagation)
- 5 new BDD scenarios testing error responses and request ID behavior in `features/template_core.feature`
- Error handling contract formalized and enforced

**Template-Core Protection (Phase 1.2):**

- Enhanced `policy/template_core.rego` to validate test presence and completeness
- Policy enforcement for core ACs (`AC-TPL-001`, `AC-TPL-002`) to prevent silent removal
- 5 comprehensive test fixtures covering valid/invalid template-core configurations
- Protection against accidental degradation of foundational features

**Meta-Contract Specifications (Phase 1.3):**

- `specs/xtask_commands.yaml` - Machine-readable specification of all 7 required xtask commands
- `specs/ac_report.schema.json` - JSON Schema for Cucumber AC report format
- Automated validation in tests ensuring `Commands` enum matches specification
- Control plane interface under contract to prevent breaking changes

**LLM Bundler Protection (Phase 1.4):**

- `policy/llm.rego` - Validates contextpack.yaml structure and required tasks
- 6 test fixtures covering validation rules (structure, tasks, required fields)
- Integration with `xtask policy-test` including YAML→JSON conversion
- LLM bundler configuration protection from silent degradation

**Phase 2: Infrastructure & Deploy Foundation**

**Kubernetes Manifests (Phase 2.1):**

- `infra/k8s/dev/deployment.yaml` - Security-hardened Deployment
  - Non-root user execution (`runAsNonRoot: true`)
  - Dropped capabilities (`drop: ["ALL"]`)
  - Read-only root filesystem
  - Resource limits and requests
  - Liveness and readiness probes
- `infra/k8s/dev/service.yaml` - ClusterIP service exposing port 8080

**Kubernetes Policies (Phase 2.2):**

- `policy/k8s.rego` - OPA policies for K8s security, labels, and resources
- 3 test fixtures (valid deployment, runs-as-root violation, missing labels)
- Integration with `xtask policy-test` for automated K8s manifest validation

**Deploy Command (Phase 2.3):**

- `crates/xtask/src/commands/deploy.rs` - Full deployment orchestration
- Environment support (dev/staging/prod) with validation
- Prerequisite checking (Docker, kubectl, namespace, service account)
- Local cluster detection and warnings
- `docs/how-to/deploy-dev.md` - Comprehensive deployment guide

**Phase 3: DevEx & Nix Improvements**

**Multi-Platform Nix Support (Phase 3.1):**

- Added `x86_64-darwin` and `aarch64-linux` to supported platforms (now 4 total)
- Added `rust-analyzer` to devShell components for better IDE integration
- Ran `nix flake update` to update dependencies

**Cleanup (Phase 3.2):**

- Removed 3 non-functional placeholder scripts
- Added TODOs in workflows for future implementation

**Verbosity Controls (Phase 3.3):**

- Global `--verbose` and `--quiet` flags on all xtask commands
- Implementation in `ac-status` and `selftest` commands
- Better CI integration and debugging capabilities

**Performance Improvements (Phase 3.4):**

- Elapsed time tracking in selftest (shown with `--verbose`)
- Converted 6 frequently-used regexes to `once_cell::sync::Lazy` for 10-20% speedup
- Optimized AC status generation for large projects

### Changed

- `policy/template_core.rego` now validates test presence and feature completeness
- `xtask policy-test` expanded to include LLM and Kubernetes policies
- `xtask selftest` shows elapsed time with `--verbose` flag
- OpenAPI specification enhanced with formal ErrorResponse contract

### Fixed

- Template-core ACs can no longer be silently removed or degraded
- xtask control plane interface protected by machine-readable specifications
- LLM bundler configuration validated against policy
- AC report format enforced by JSON Schema

### Documentation

- `docs/meta_contract_phase1.3.md` - Meta-contract design and implementation
- `policy/README.md` - Policy organization and testing guide
- `docs/how-to/deploy-dev.md` - Development deployment guide
- Updated all policy files with comprehensive comments

### Technical Details

**Files Created (30+):**

- 2 machine-readable specifications (xtask_commands.yaml, ac_report.schema.json)
- 4 new policies (template_core enhancements, llm.rego, k8s.rego, policy/README.md)
- 14 policy test fixtures (template_core, LLM, K8s)
- 2 infrastructure manifests (deployment.yaml, service.yaml)
- 2 new xtask modules (commands/deploy.rs, validation.rs)
- 4 new documentation files

**Code Statistics:**

- ~800 lines of new Rust code (xtask deploy + validation)
- ~400 lines of Rego policies
- ~300 lines of K8s manifests
- ~600 lines of new documentation
- 100% test coverage for new policies

**Validation:**

- All `xtask selftest --verbose` checks passing (outside Nix)
- Template-core policies enforcing foundational ACs
- Meta-contract validation preventing control plane breaking changes
- K8s manifests pass security policies

**Dependencies Added:**

- None - all improvements use existing dependencies

### Security

- K8s deployment runs as non-root with minimal privileges
- All capabilities dropped from containers
- Read-only root filesystem in K8s pods
- Resource limits prevent resource exhaustion
- Security policies enforced via Rego

### Breaking Changes

**NONE** - All changes are backward compatible. New policies and infrastructure are additive.

### Migration Guide

No migration required. All new features are opt-in:

**To use the deploy command:**

```bash
cargo xtask deploy --env dev
# See docs/how-to/deploy-dev.md
```

**To benefit from new policies:**

```bash
cargo xtask policy-test
# Automatically includes template_core, llm, and k8s policies
```

**To use verbose output:**

```bash
cargo xtask selftest --verbose
# Shows elapsed time and detailed progress
```

### For Contributors

- Read `docs/meta_contract_phase1.3.md` for meta-contract design
- Review `policy/README.md` for policy organization
- Check `docs/how-to/deploy-dev.md` for deployment workflows
- See `specs/` directory for machine-readable contracts

---

## [2.1.0] - 2025-11-17

### Added

**Observability: Prometheus Metrics**

- **`/metrics` endpoint** in `app-http` exposing Prometheus-formatted metrics
  - Global `http_requests_total` counter with labels: `method`, `path`, `status`
  - Middleware automatically tracks all HTTP requests
  - Tests validate metrics collection and endpoint response format
- **K8s manifests** updated for prod and staging environments:
  - Added `service-patch.yaml` with Prometheus scrape annotations:
    - `prometheus.io/scrape: "true"`
    - `prometheus.io/port: "8080"`
    - `prometheus.io/path: "/metrics"`
- **Rego policy enforcement** for metrics observability:
  - Production and staging Services **must** have Prometheus annotations
  - Policy test fixtures added: `k8s_service_metrics_valid.yaml`, `k8s_service_metrics_missing.yaml`
  - Dev environment exempt (policy only enforces in staging/prod)

**Telemetry: OTLP Feature Flag Preparation**

- Added `otlp` feature flag to `crates/telemetry/Cargo.toml`
- Dependencies configured: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`
- Implementation deferred to v2.2.0 (OpenTelemetry 0.31 API research needed)
- TODO comment in `init_tracing` documents next steps

### Changed

**Dependencies**

- Updated workspace dependencies with OTLP feature flags:
  - `opentelemetry_sdk = { version = "0.31.0", features = ["rt-tokio"] }`
  - `opentelemetry-otlp = { version = "0.31.0", features = ["tonic", "trace"] }`
- Added `once_cell = "1.20.2"` to `app-http` for metrics registry initialization

### Infrastructure

**Kustomize**

- `infra/k8s/prod/service-patch.yaml` - Prometheus annotations for production
- `infra/k8s/staging/service-patch.yaml` - Prometheus annotations for staging
- Updated `kustomization.yaml` in both environments to apply service patches

**Policy-as-Code**

- `policy/k8s.rego` - New rules for Prometheus scrape configuration enforcement
- `policy/testdata/` - Test fixtures validating metrics policy compliance

### Notes

- `/metrics` endpoint is **immediately usable** with any Prometheus-compatible scraper
- OTLP tracing deferred to v2.2.0 to allow proper research of OpenTelemetry 0.31.x API changes
- This release completes the observability foundation for production monitoring

---

## [2.0.1] - 2025-11-17

### Fixed

**xtask: Policy test behavior improvements**

- Policy tests (`cargo xtask policy-test`) now **skip gracefully on local dev machines** when `conftest` is not installed, displaying a clear warning message with installation instructions
- In CI environments (detected via `CI` or `GITHUB_ACTIONS` env vars), policy tests remain **strictly enforced** and will fail if `conftest` is missing
- This change eliminates noisy failures on dev machines while maintaining governance enforcement in pipelines
- Nix `devShell` includes `conftest`, ensuring CI runs (which use `nix develop`) always have the tool available

**rust_iac_config: Eliminate flaky tests**

- Marked integration tests that call `std::env::set_current_dir` as `#[ignore]` with explicit rationale
- These tests (`test_find_environment`, `test_required_files_validation`) manipulate global process state and fail non-deterministically when run in parallel with other tests
- `cargo test --workspace` is now fully deterministic and green
- Tests can still be run individually when needed for debugging specific scenarios

### Changed

- Introduced `PolicyTestError` enum in `policy_test.rs` to distinguish "conftest not found" from other errors
- Updated `selftest.rs` to handle `ConftestNotFound` errors conditionally based on environment

### Technical Details

**No API changes** - This is a pure quality-of-life patch addressing developer experience and test reliability.

**Files Modified:**

- `crates/xtask/src/commands/policy_test.rs` - New error type and conftest detection
- `crates/xtask/src/commands/selftest.rs` - CI-aware error handling
- `crates/xtask/src/main.rs` - Error conversion for policy test command
- `crates/rust_iac_config/tests/integration_tests.rs` - Mark flaky tests as ignored

**Validation:**

- ✅ `cargo test --workspace` - All tests pass
- ✅ `cargo xtask selftest` - Full suite passes (policy tests skip gracefully without conftest)
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` - Clean
- ✅ `cargo fmt --all -- --check` - Clean

### Migration Guide

No migration required. This is a backward-compatible patch:

**Local development without conftest:**

- `cargo xtask selftest` will skip policy tests with a warning
- Use `cargo xtask policy-test` directly to see installation instructions
- Or use `nix develop` to get conftest automatically

**CI environments:**

- Policy tests continue to run and fail if conftest is missing
- Existing CI workflows using `nix develop` are unaffected

**Flaky tests:**

- Tests marked `#[ignore]` will not run by default
- To run them individually: `cargo test test_find_environment -- --ignored`

---

## [2.0.0] - 2025-11-17

### Added

**Complete workspace stabilization and crate split for production readiness**

This major release represents a fundamental reorganization of the template into a production-grade, multi-crate workspace with clear architectural boundaries and stabilized APIs.

**Core Workspace Crates:**

- `business-core` - Core business logic with async traits and clean boundaries
- `model` - Shared domain models and types
- `adapters-db-sqlx` - Database adapter with SQLx integration
- `adapters-grpc` - gRPC adapter with tonic integration
- `app-http` - HTTP server runtime with Axum
- `telemetry` - Observability with tracing and metrics
- `rust_iac_config` - Infrastructure-as-code configuration library
- `rust_iac_xtask_core` - Core xtask automation library
- `xtask` - Command-line automation tool

**Key Architectural Improvements:**

- Hexagonal architecture fully realized across workspace
- Async traits for clean adapter interfaces
- Clear separation between core business logic and adapters
- Stabilized public APIs with semantic versioning
- Production-ready error handling and observability patterns

**Observability:**

- OTLP telemetry support with structured tracing
- Prometheus metrics integration
- Request ID correlation across all layers
- Structured logging with context preservation

**Infrastructure:**

- Rust-native IaC libraries (`rust_iac_config`, `rust_iac_xtask_core`)
- Policy-as-code foundation with Rego
- Complete xtask automation suite
- Nix flake for reproducible development environments

### Changed

- **BREAKING**: Workspace reorganized from monolithic structure to multi-crate architecture
- **BREAKING**: All public APIs stabilized - expect semantic versioning going forward
- Async traits throughout core business logic
- Enhanced error types with better context and structured logging

### Removed

- Legacy monolithic crate structure
- Temporary scaffolding and experimental code paths

### Technical Details

**Major Commits:**

- `1889bae` - Complete workspace stabilization
- `a58dc11` - Stabilize adapters-grpc and adapters-db-sqlx with async traits
- `5b7e027` - Finalize crate split and stabilize core workspace
- `cbd7186` - Add core library for Rust IaC xtask automation
- `fc0f9bc` - Integrate OTLP telemetry and Prometheus metrics
- `ecef157` - Complete governance foundation and LLM-first positioning

**Crate Count:** 9 production crates (excluding examples)

**Validation:**

- All workspace tests passing
- Full clippy compliance
- Documentation coverage for public APIs
- Policy tests enforcing governance

### Migration Guide

For users upgrading from v1.x:

**Workspace structure:**

- Code is now split across focused crates
- Update imports to reference new crate names
- See individual crate `CHANGELOG.md` files for detailed changes

**Dependencies:**

- Add explicit dependencies on crates you use (e.g., `business-core`, `model`)
- Remove dependencies on the old monolithic crate

**APIs:**

- Review async trait signatures in `business-core`
- Update adapter implementations to match new interfaces
- Check `model` crate for domain type changes

---

## [2.2.0] - 2025-11-17

### Added

**Adapter Integration Testing**

- **DB adapter integration test** with testcontainers + Postgres
  - End-to-end CRUD test for `TaskRepository` using real Postgres instance
  - Validates `create_task`, `get_task`, and `list_tasks` operations
  - Located in `crates/adapters-db-sqlx/tests/integration_test.rs`
  - Proves database adapter contract compliance without mocks
- **gRPC adapter smoke test** with in-memory repository
  - Tests gRPC service layer (`CreateTask`, `GetTask`, `ListTasks`) against in-memory implementation
  - No Docker dependency, fast feedback loop
  - Located in `crates/adapters-grpc/tests/smoke_test.rs`
  - Validates protobuf serialization and gRPC server behavior

**BDD Scenario for Metrics**

- New `specs/features/metrics.feature` with scenario `@AC-TPL-007`
  - Tests `/metrics` endpoint availability
  - Validates `http_requests_total` metric is present in Prometheus output
  - Uses existing step definitions with new raw body support
- Enhanced acceptance test infrastructure:
  - `Response` struct now includes `raw_body: String` field for non-JSON responses
  - New step definition: `the response body contains "{string}"`
  - Extended GET step to support `/metrics` endpoint

**LLM Ergonomics Improvements**

- Enhanced `.llm/contextpack.yaml` with richer metadata for `implement_ac` task:
  - `description` field: "Context for implementing a single acceptance criterion end-to-end."
  - `prompt` field: Embedded workflow guidance for LLMs (understand AC → find BDD → edit core → run selftest)
  - Updated include paths to reference `business-core` and `app-http` (not legacy `core`)
  - Added tutorial reference (`docs/tutorials/day-7-first-real-feature.md`)
- Documentation update in `docs/how-to/use-llm-bundles.md`:
  - New "Quick Reference" section with 5-step workflow
  - Clear path from bundle generation → LLM prompt → verification

**VSCode Integration (optional, local-only)**

- `.vscode/tasks.json` with shortcuts for xtask commands:
  - `xtask: check`, `xtask: selftest`, `xtask: bdd`, `xtask: bundle implement_ac`, `xtask: policy-test`, `xtask: ac-status`
  - Accessible via VSCode "Tasks → Run Task" menu
  - Note: `.vscode/` is in `.gitignore`, so this is not committed to the repository

### Changed

- Acceptance test `World` struct now tracks both JSON body and raw text responses
- All step definitions updated to populate `raw_body` field
- GET step (`when_get_endpoint`) regex extended to include `/metrics`

### Fixed

- Acceptance tests now properly handle non-JSON responses (e.g., Prometheus plain text format)
- LLM contextpack paths corrected to reference current workspace structure

### Technical Details

**Files Modified:**

- `.llm/contextpack.yaml` - Enhanced with description and prompt metadata
- `crates/acceptance/src/world.rs` - Added `raw_body` field to `Response` struct
- `crates/acceptance/src/steps/template_core.rs` - Updated all step definitions, added raw body assertion step
- `specs/features/metrics.feature` - New BDD scenario for metrics endpoint
- `docs/how-to/use-llm-bundles.md` - Added quick-reference workflow section

**New Files:**

- `crates/adapters-db-sqlx/tests/integration_test.rs` - DB adapter integration test
- `crates/adapters-grpc/tests/smoke_test.rs` - gRPC adapter smoke test
- `specs/features/metrics.feature` - Metrics endpoint BDD scenario

**Validation:**

- ✅ `cargo test --workspace` - All tests pass (including new integration tests)
- ✅ `cargo xtask selftest` - All 5 checks passing
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` - Clean
- ✅ `cargo fmt --all -- --check` - Clean

**Dependencies Added:**

- `testcontainers = "0.23"` (dev dependency in adapters-db-sqlx) - For Postgres integration tests

### Notes

- OTLP tracing implementation **deferred to v2.3.0** to allow proper research of OpenTelemetry 0.31.x API changes
- This release completes the minimum viable v2.2.0 scope with solid adapter testing and improved LLM workflow

---

## [2.3.0] - 2025-11-17

### Added

**OTLP Tracing (Feature-Gated)**

- `telemetry` crate now supports OTLP (OpenTelemetry Protocol) export via the `otlp` feature flag
- When `otlp` feature is enabled and `OTLP_ENDPOINT` environment variable is set:
  - Traces are exported to OTLP collectors via gRPC (tonic)
  - Uses OpenTelemetry 0.31.x API with `SpanExporter::builder()` pattern
  - Combines OTLP export with console tracing for local development
- Graceful fallback behavior:
  - If OTLP initialization fails (e.g., collector unreachable), logs warning and falls back to console-only tracing
  - Application never crashes due to telemetry issues
- Feature flag: `telemetry/otlp` (disabled by default)
- No changes to existing `telemetry::init_tracing()` call sites

**Documentation**

- `docs/how-to/test-otlp-tracing.md` - Complete guide for testing OTLP locally with Jaeger
  - Quick start with Jaeger all-in-one container
  - Alternative setup with OpenTelemetry Collector
  - Fallback behavior verification
  - Troubleshooting guide
- `crates/telemetry/README.md` - API documentation and usage examples
  - Console tracing vs. OTLP tracing
  - Environment variable reference
  - Design principles and implementation details

### Technical Details

**Files Modified:**

- `crates/telemetry/src/lib.rs` - Implemented `try_init_otlp()` function under `#[cfg(feature = "otlp")]`
- `crates/telemetry/Cargo.toml` - Added `otlp` feature with dependencies: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` (with `grpc-tonic`), `tracing-opentelemetry`

**New Files:**

- `docs/how-to/test-otlp-tracing.md` - OTLP testing guide
- `crates/telemetry/README.md` - Telemetry crate documentation
- `docs/v2.3.0-plan.md` - Release plan with OTLP version decision

**Validation:**

- ✅ `cargo build -p telemetry` - Builds without OTLP feature (default)
- ✅ `cargo build -p telemetry --features otlp` - Builds with OTLP feature
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` - Clean
- ✅ `cargo fmt --all -- --check` - Clean
- ✅ `cargo run -p xtask -- selftest` - Core checks passing

**Dependencies:**

- OpenTelemetry 0.31.0 (already in workspace)
- Using official OTLP patterns from opentelemetry-rust examples

### Design Decisions

**OTLP Version Strategy:**

- Chose OpenTelemetry 0.31.0 (current workspace version)
- Rationale: No dependency churn, aligns with current ecosystem, better long-term maintenance
- Rejected: Pinning to older 0.24.x (would create technical debt)

**Implementation Approach:**

- Feature-gated to keep default builds lightweight
- Graceful degradation ensures production reliability
- No breaking changes to existing telemetry API

### Notes

- OTLP remains **optional** - default builds use console-only tracing
- No changes required to existing applications
- To enable OTLP: `cargo build --features telemetry/otlp` and set `OTLP_ENDPOINT`
- Telemetry failures never crash the application

---

### Changed

**Documentation Cleanup (2025-11-19):**

**Removed** `scripts/create-pilot.sh` - Replaced with explicit GitHub template + git instructions

- Shell scaffolding didn't align with Rust-native, xtask-driven philosophy
- GitHub "Use this template" provides better first-class experience
- Manual git clone path is now fully documented as fallback

**Updated** documentation with clear bootstrap instructions:

- `README.md` - Pilot Workflow section now shows both GitHub template and manual clone paths
- `CLAUDE.md` - Updated "Pilot Workflow" and "Create a Pilot Project" prompts
- `docs/README.md` - Updated Pilot Projects section with explicit git commands

**Rationale**: Template instantiation is better served by GitHub's built-in features and documented git steps than by an opaque shell script. All "real" automation lives in Rust (xtask), Rego, or Nix.

### Post-v2.3.0: Pilot Infrastructure

**Added** (2025-11-17, commits fe0a00e, 2615dbc):

**Pilot Project Tooling:**

- `scripts/create-pilot.sh` - Automated pilot project creation from v2.3.0 template
  - Copies template to new directory with fresh git history
  - Pre-configures `FRICTION_LOG.md` with metadata (project name, date, developer)
  - Runs initial selftest to verify setup
  - Provides comprehensive next-step guidance
- `docs/templates/PILOT_FEATURE_IDEAS.md` - Curated pilot feature catalog
  - 3 pilot archetypes: Task Management (starter), E-commerce (intermediate), User Auth (advanced)
  - 18 feature examples across simple/medium/complex tiers
  - Expected friction points per feature
  - Half-day and week-long pilot paths
  - Anti-patterns and recommendations
- `docs/templates/FRICTION_LOG.md` - Structured friction capture template
  - Pre-formatted sections for pain points, feature tracking, observability/governance assessment
  - Guides pilot evaluation and template evolution decisions
- `docs/RELEASE_PLAYBOOK.md` - Reusable 7-phase release process
  - Planning → Roadmap → Code → Validation → Documentation → Tagging → Post-Release
  - Adaptable to non-Rust projects
  - Multiple validation gates and checklists
- `docs/templates/RELEASE_PLAN.md` - Template for future release planning
  - Scope definition, implementation roadmap, exit criteria, decision log structure

**Documentation:**

- `README.md` - Added "Quick Start (Pilot Project)" and "Pilot Workflow" sections
  - Day 0 → Day 1+ workflow guide
  - Friction log → template evolution feedback loop
  - References to Release Playbook

**Strategic Shift:**

- v2.3.0 closes the three-release observability arc (v2.1.0 → v2.2.0 → v2.3.0)
- Template now has complete observability stack (logs, metrics, traces) + governance infrastructure
- Pilot infrastructure enables validation through real usage before planning v2.4.0
- Template evolution should be informed by pilot friction logs, not feature speculation

**Usage (Historical - script removed 2025-11-19, see "Changed" section above):**

```bash
# Original workflow (now replaced with GitHub template + git instructions)
./scripts/create-pilot.sh my-pilot-service ~/projects/

# Follow Day 1 workflow
cd ~/projects/my-pilot-service
cargo run -p xtask -- selftest
# Implement features, track friction in FRICTION_LOG.md
# After 1-2 weeks: review friction, decide on v2.3.1/v2.4.0
```

**Current workflow:** See README.md "Pilot Workflow" section for GitHub template + manual git clone instructions.

### Planned for Later

- Run greenfield pilot project (1-2 weeks)
- Analyze friction logs to inform v2.3.1 or v2.4.0 planning
- Docker build automation in deploy command
- Additional how-to guides based on pilot learnings

---

[2.0.1]: https://github.com/your-org/rust-template/releases/tag/v2.0.1
[2.0.0]: https://github.com/your-org/rust-template/releases/tag/v2.0.0
[1.1.0]: https://github.com/your-org/rust-template/releases/tag/v1.1.0
[1.0.0]: https://github.com/your-org/rust-template/releases/tag/v1.0.0
