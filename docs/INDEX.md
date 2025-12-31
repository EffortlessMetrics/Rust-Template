---
id: GUIDE-TPL-DOC-INDEX-001
title: Documentation Index
doc_type: guide
status: published
audience: developers, maintainers
tags: [navigation, index, documentation, reference]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: [ADR-0005]
last_updated: 2025-12-22
---

# Documentation Index

**Navigation hub for the Rust-as-Spec Platform Cell (v3.3.14)**

---

## Quick Start

New to this repository? Start here:

1. **[QUICKSTART.md](QUICKSTART.md)** - Get productive in under 15 minutes
2. **[README.md](../README.md)** - Overview, quick start, key features
3. **[Why This Template Exists](why-this-exists.md)** - Problem statement and philosophy
4. **[Getting Started](#getting-started)** - Step-by-step onboarding

---

## Strategic Documents

### Status & Vision
- **[ROADMAP.md](ROADMAP.md)** - Current state (v3.3.14), pilot plan, future direction
- **[KERNEL_SNAPSHOT.md](KERNEL_SNAPSHOT.md)** - v3.3.9-kernel frozen baseline, AC status, capabilities
- **[IDP_CELL_CONTRACT.md](IDP_CELL_CONTRACT.md)** - IDP integration contract (surfaces, trust model, versioning)
- **[BACKLOG.md](BACKLOG.md)** - Future feature ideas and enhancements
- **[PILOT-PROJECT-PLAN.md](PILOT-PROJECT-PLAN.md)** - Pilot project planning and validation

### Constitution & Governance
- **[CLAUDE.md](../CLAUDE.md)** - Repository constitution and operating manual for autonomous agents
- **[CONSTITUTION.md](CONSTITUTION.md)** - Core principles and governance model

### For Agents (LLMs)
- **[AGENT_GUIDE.md](AGENT_GUIDE.md)** - Operational guide for LLMs driving workflows
- **[AGENT_SKILLS.md](AGENT_SKILLS.md)** - Available skills and when to use them
- **[MISSING_MANUAL.md](MISSING_MANUAL.md)** - Unwritten knowledge and edge cases

---

## Getting Started

### Installation & Setup
- **[QUICKSTART.md](QUICKSTART.md)** - 15-minute fast path to productivity
- **[dev-environment.md](dev-environment.md)** - Development environment setup and configuration
- **[Windows Development Guide](how-to/windows-development.md)** - Complete Windows setup (WSL2 + native, Tier-2 support)

### First Steps
- **[Getting Started Tutorial](tutorials/getting-started.md)** - Complete walkthrough for new users
- **[First AC Change](tutorials/first-ac-change.md)** - Implement your first acceptance criterion
- **[Day 1: First Change](tutorials/day-1-first-change.md)** - Your first day with the template
- **[Day 7: First Real Feature](tutorials/day-7-first-real-feature.md)** - Building your first complete feature
- **[First Service](tutorials/first-service.md)** - Creating a new service from scratch

---

## How-To Guides

### Adoption & Setup
- **[Adopt Kernel](how-to/adopt-kernel.md)** - **Kernel adoption guide for forks (v3.3.9-kernel+)** ⭐
- **[Fork and Build Your First Feature](how-to/fork-and-build-first-feature.md)** - Golden path: fork to feature in 2-3 hours
- **[Add Governance to Existing Repo](how-to/add-governance-to-existing-repo.md)** - Brownfield adoption (complete guide)
- **[Create New Service from Template](how-to/new-service-from-template.md)** - Greenfield instantiation
- **[Your First Hour](how-to/first-hour.md)** - Hands-on exploration tour (30-60 min)
- **[Pre-Fork Checklist](how-to/pre-fork-checklist.md)** - Before forking this template
- **[Second Service LLM Sanity Check](how-to/second-service-llm-sanity.md)** - Validating second service with LLM assistance

### Fork Customization (Complete Guide)
- **[First Fork Guide](how-to/FIRST_FORK.md)** - Complete first-time fork setup (20 min)
- **[Change Template Opinions](how-to/change-template-opinion.md)** - Override or relax template defaults safely (with worked examples)
- **[Reconcile Kernel Updates](how-to/reconcile-kernel-updates.md)** - Pull upstream fixes without losing customizations
- **[Report Fork Feedback](how-to/report-fork-feedback.md)** - How to report issues and improvements

### Development Workflows
- **[Add Acceptance Criterion](how-to/add-acceptance-criterion.md)** - Creating new ACs in the spec ledger
- **[Add HTTP Endpoint](how-to/add-http-endpoint.md)** - Adding new API endpoints
- **[Change OpenAPI Safely](how-to/change-openapi-safely.md)** - Modifying API contracts without breaking changes
- **[Using LLM Bundles](how-to/use-llm-bundles.md)** - Bounded context for AI assistance

### CI/CD & Branch Protection
- **[Setup Branch Protection](how-to/setup-branch-protection.md)** - Configuring branch protection rules
- **[Setup Tag Signing](how-to/setup-tag-signing.md)** - GPG signing for releases

### Operations & Deployment
- **[Deploy to Dev Environment](how-to/deploy-dev.md)** - Development deployment guide
- **[Test OTLP Tracing](how-to/test-otlp-tracing.md)** - OpenTelemetry tracing setup and validation

---

## Reference Documentation

### Platform & Environment
- **[Platform Support Reference](reference/platform-support.md)** - Complete platform support matrix, Tier 1/Tier 2, Windows guidance
- **[xtask Command Reference](reference/xtask-commands.md)** - All CLI commands and options

### CI/CD & Branch Protection
- **[CI Workflows Reference](reference/ci-workflows.md)** - Comprehensive CI workflow documentation
- **[CI Coverage](reference/ci-coverage.md)** - CI coverage matrix and validation
- **[Required Checks](reference/required-checks.md)** - Required CI checks for merge
- **[Branch Protection Profiles](reference/branch-protection-profiles.md)** - Branch protection configuration options

### Architecture & Design
- **[ADR Index](adr/README.md)** - All architecture decision records
  - **[ADR-0001: Hexagonal Architecture](adr/0001-hexagonal-architecture.md)** - Ports & adapters pattern
  - **[ADR-0002: Nix-First Dev Environment](adr/0002-nix-first-dev-env.md)** - Tier-1 reproducible environments
  - **[ADR-0003: Spec and BDD as Source of Truth](adr/0003-spec-and-bdd-as-source-of-truth.md)** - Spec-driven development
  - **[ADR-0004: Policy and LLM Governance](adr/0004-policy-and-llm-governance.md)** - OPA policies for governance
  - **[ADR-0005: xtask selftest Single Gate](adr/0005-xtask-selftest-single-gate.md)** - Unified validation
  - **[ADR-0006: Supply Chain Hardening](adr/0006-supply-chain-hardening.md)** - Security and dependency management
  - **[ADR-0007: Dependency Security Health](adr/0007-dependency-security-health.md)** - Dependency monitoring
  - **[ADR-0017: Tier-1 Selftest Gate](adr/0017-tier1-selftest-gate.md)** - CI enforcement model
  - **[ADR-0019: Governance Repository and FS Adapter](adr/0019-governance-repository-and-fs-adapter.md)** - Governance data layer

---

## Examples & Reference Implementations

Working code examples demonstrating platform integration patterns:

### IDP Integration
- **[Backstage Plugin Example](../examples/backstage-plugin/README.md)** - React TypeScript plugin for governance tiles
- **[Port.io Integration Guide](how-to/implement-port-integration.md)** - Python-based IDP ingestion

### Agent & Automation
- **[Agent Pilot Example](../examples/agent-pilot/README.md)** - Python agent pilot for autonomous workflows
- **[Agent Guide](AGENT_GUIDE.md)** - Operational guide for LLM agents

### Adoption Patterns
- **[Brownfield Demo](../examples/brownfield-demo/README.md)** - Adding governance to existing services
- **[Fork Customization](../examples/fork-customization/README.md)** - Template fork and customization patterns

### Fork Examples
- **[Example MyService Fork](../forks/example-myservice/README.md)** - Minimal fork demonstrating domain customization
- **[Fork Registry](../forks/README.md)** - Fork management and tracking

> **Note:** Examples are illustrative quality, not production-ready. See the example README files for quality statements and production considerations.

---

## Explanation Documents

Deeper understanding of concepts:

### Core Concepts
- **[Rust-as-Spec Overview](explanation/rust-as-spec-overview.md)** - Complete technical architecture
- **[Architecture](explanation/architecture.md)** - System architecture overview
- **[Template Architecture](explanation/template-architecture.md)** - Template-specific architecture
- **[LLM-Native DevEx](explanation/llm-native-devex.md)** - Building for LLM-driven workflows
- **[Controls as Code](explanation/controls-as-code.md)** - Governance automation philosophy
- **[IDP Positioning](explanation/idp-positioning.md)** - Internal Developer Platform positioning

### Patterns & Design
- **[Adoption Patterns](explanation/adoption-patterns.md)** - Template vs upstream vs generator
- **[Adapters](explanation/adapters.md)** - Adapter pattern in hexagonal architecture
- **[Infra Modules](explanation/infra-modules.md)** - Infrastructure module design
- **[Supply Chain Hardening](explanation/supply-chain-hardening.md)** - Security and supply chain

### Template Management
- **[Template Contracts](explanation/TEMPLATE-CONTRACTS.md)** - Template versioning and contracts
- **[Template Foundation vs Examples](explanation/template-foundation-vs-examples.md)** - Kernel vs examples
- **[Template Versioning](explanation/template-versioning.md)** - Version management strategy

---

## Tutorials

Step-by-step learning paths:

1. **[Getting Started](tutorials/getting-started.md)** - Complete onboarding tutorial
2. **[First AC Change](tutorials/first-ac-change.md)** - Hello World for this template
3. **[Day 1: First Change](tutorials/day-1-first-change.md)** - Your first day with the template
4. **[Day 7: First Real Feature](tutorials/day-7-first-real-feature.md)** - Building your first complete feature
5. **[First Service](tutorials/first-service.md)** - Creating a new service from scratch

---

## Design Documents

Active design documents for current and future work:

### Platform & Runtime
- **[Platform DevEx](design/platform-devex.md)** - Developer experience design
- **[Platform Introspection](design/platform-introspection.md)** - Runtime introspection APIs
- **[Platform Runtime Contract](design/platform-runtime-contract.md)** - Platform runtime guarantees
- **[Platform UI](design/platform-ui.md)** - Web UI design
- **[Local Runtime](design/local-runtime.md)** - Local development runtime
- **[IDP Tile Specifications](design/DESIGN-IDP-TILES.md)** - IDP tile specifications for platform integration

### Governance & Validation
- **[Governance Hooks](design/governance-hooks.md)** - Pre-commit and CI governance hooks
- **[Graph Invariants](design/graph-invariants.md)** - Governance graph structural rules
- **[GOV-WRITE-001](design/gov-write-001.md)** - Write operations in governance

### Developer Experience
- **[Agent Interface](design/agent-interface.md)** - LLM agent interface design
- **[Skills Guide](design/skills-guide.md)** - Skills system design
- **[Skills Tooling](design/skills-tooling.md)** - Skills implementation tooling
- **[Suggest Next](design/suggest-next.md)** - Task suggestion algorithm
- **[Status CLI](design/status-cli.md)** - Status command design
- **[LLMignore Semantics](design/llmignore-semantics.md)** - LLM context filtering

### Endpoints & APIs
- **[Health Endpoint](design/health-endpoint.md)** - Service health endpoint design
- **[Version Endpoint](design/version-endpoint.md)** - Version information endpoint
- **[Error Handling](design/error-handling.md)** - Error handling strategy
- **[Metrics](design/metrics.md)** - Metrics and observability

### Release & Testing
- **[Release Bundling](design/release-bundling.md)** - Release artifact generation
- **[AC Structured Report](design/ac-structured-report.md)** - Acceptance criteria reporting
- **[Task Lifecycle](design/task-lifecycle.md)** - Task state management
- **[Single Versioning Engine](design/DESIGN-VERSIONING-ENGINE.md)** - Single versioning engine for release-prepare (deferred tech debt)

---

## Runbooks & Operational Guides

- **[Platform Kernel Runbook](runbooks/platform-kernel.md)** - Operational guide for platform kernel

---

## Plans & Trackers

Historical and current planning documents:

- **[v2.1.0 Plan](v2.1.0-plan.md)** - Historical release plan
- **[v2.2.0 Plan](v2.2.0-plan.md)** - Historical release plan
- **[v2.2.0 Execution Roadmap](v2.2.0-execution-roadmap.md)** - Historical execution plan
- **[v2.2.0 Tracker](v2.2.0-tracker.md)** - Historical progress tracker
- **[v2.3.0 Plan](v2.3.0-plan.md)** - Historical release plan
- **[Health Endpoint v1 Plan](plans/health-endpoint-v1.md)** - Feature-specific plan

---

## Requirements & Specifications

- **[Service Core Requirements](requirements/service-core.md)** - Core service requirements

---

## Release Documentation

- **[RELEASE_PLAYBOOK.md](RELEASE_PLAYBOOK.md)** - Release process and checklist
- **[RELEASE_v2.5.0.md](RELEASE_v2.5.0.md)** - v2.5.0 release notes
- **[READY-FOR-PRODUCTION-CHECKLIST.md](READY-FOR-PRODUCTION-CHECKLIST.md)** - Production readiness criteria

---

## Process & Quality

- **[testing-strategy.md](testing-strategy.md)** - Testing philosophy and approach
- **[SELECTIVE_TESTING.md](SELECTIVE_TESTING.md)** - Selective test execution strategy
- **[feature_status.md](feature_status.md)** - Auto-generated AC status (generated by `cargo xtask ac-status`)
- **[feature_status_notes.md](feature_status_notes.md)** - Manual AC status notes

---

## Docs Governance

Documentation is governed by contracts enforced through `cargo xtask docs-check`.

### Docs Governance Quickstart

When updating documentation:

1. **Update `specs/doc_index.yaml`** if adding/moving/renaming docs
2. **Sync front-matter** with doc_index entries:
   ```bash
   cargo xtask docs-frontmatter-sync
   ```
3. **Validate governance**:
   ```bash
   cargo xtask docs-check
   ```

### Key Invariants

| Invariant | Enforced By | AC |
|-----------|-------------|-----|
| **Version alignment** | `docs-check` | AC-PLT-009 |
| **Feature status header** | `docs-check` | AC-PLT-010 |
| **Doc index ↔ front-matter sync** | `docs-check` | AC-PLT-DOC-INDEX-FRONTMATTER |
| **ADR format** | `adr-check` | — |
| **Skills format** | `skills-lint` | AC-TPL-SKILLS-LINT |
| **TypeScript config** | `validate-ts-config.sh` (CI) | — |

### Version Authority

The canonical version is `specs/spec_ledger.yaml → metadata.template_version`.

These files are validated against it:
- README.md, CLAUDE.md, ROADMAP.md
- KERNEL_SNAPSHOT.md, TEMPLATE-CONTRACTS.md
- service_metadata.yaml, doc_index.yaml
- CHANGELOG.md

### Cutting a New Version

```bash
cargo xtask release-prepare X.Y.Z    # Update all version-bearing files
cargo xtask ac-status                 # Regenerate feature_status.md
cargo xtask docs-check                # Validate governance
cargo xtask selftest                  # Full validation
```

### Reference

- **[doc-sources.md](reference/doc-sources.md)** - Complete governance specification
- **[TEMPLATE-CONTRACTS.md](explanation/TEMPLATE-CONTRACTS.md)** - Template versioning contracts

---

## Troubleshooting

- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Comprehensive troubleshooting guide (FAQ format)
- **[BRANCH-PROTECTION-SETUP.md](BRANCH-PROTECTION-SETUP.md)** - Branch protection troubleshooting

---

## Guides (Legacy)

- **[Brownfield Adoption Guide](guides/brownfield-adoption.md)** - Legacy brownfield adoption guide

---

## Templates & Examples

Scaffolds you can copy:

- **[ADR Template](templates/ADR-TEMPLATE.md)** - Architecture decision record template
- **[Design Doc Template](templates/DESIGN_DOC.example.md)** - Design document template
- **[Requirements Doc Template](templates/REQUIREMENTS_DOC.example.md)** - Requirements document template
- **[Plan Doc Template](templates/PLAN_DOC.example.md)** - Planning document template
- **[Friction Log Template](templates/FRICTION_LOG.md)** - Track pain points during development
- **[Release Plan Template](templates/RELEASE_PLAN.md)** - Release planning template
- **[Runbook Template](templates/RUNBOOK.example.md)** - Operational runbook template
- **[Pilot Feature Ideas](templates/PILOT_FEATURE_IDEAS.md)** - Feature ideation template

---

## Historical & Meta Documents

- **[implementation-summary-2025-11-15.md](implementation-summary-2025-11-15.md)** - Historical implementation summary
- **[TECHNICAL-FREEZE-COMPLETE.md](TECHNICAL-FREEZE-COMPLETE.md)** - Technical freeze milestone
- **[meta_contract_phase1.3.md](meta_contract_phase1.3.md)** - Meta-contract development phase
- **[ci-examples.md](ci-examples.md)** - CI configuration examples

---

## API Documentation

- **[API Test Documentation](api/test.md)** - API testing documentation

---

## Status Legend

- ✅ **Complete** - Document exists and is comprehensive
- 🔄 **In Progress** - Document exists but needs expansion
- 📝 **Stub** - Placeholder for future development
- ❌ **Missing** - Not yet created
- 🤖 **Auto-generated** - Generated by `cargo xtask` commands

Current status (v3.3.14):
- ✅ All core documentation (ROADMAP, QUICKSTART, AGENT_GUIDE, platform references, CI workflows)
- ✅ Complete ADR set (9 ADRs)
- ✅ All design documents
- ✅ All how-to guides
- ✅ All tutorials
- 🤖 feature_status.md (auto-generated, do not edit)

---

## Document Conventions

### File Naming
- How-tos: `how-to/{verb}-{noun}.md` (e.g., `how-to/add-governance-to-existing-repo.md`)
- Tutorials: `tutorials/{task-name}.md` (e.g., `tutorials/first-ac-change.md`)
- Explanations: `explanation/{concept}.md` (e.g., `explanation/rust-as-spec-overview.md`)
- Reference: `reference/{topic}.md` (e.g., `reference/platform-support.md`)
- Design: `design/{feature-name}.md` (e.g., `design/platform-ui.md`)

### Front Matter
All docs should have YAML front-matter:

```yaml
---
id: unique-doc-id
type: how_to | tutorial | explanation | reference | design_doc
status: draft | active | deprecated
last_reviewed: YYYY-MM-DD
owner: team-name
related_requirements: [REQ-ID-1, REQ-ID-2]
---
```

### Cross-Linking
- Use relative paths: `[Text](../how-to/file.md)`
- Use file:// URIs for code: `[spec_ledger.yaml](file:///path/to/spec_ledger.yaml)`
- Link to specific lines: `[graph.rs:L123](file:///path/to/graph.rs#L123)`

---

## Need Help?

1. **Start with QUICKSTART.md** for a 15-minute introduction
2. **Check TROUBLESHOOTING.md** for common issues (use Ctrl+F to search)
3. **Search this index** for relevant docs
4. **Try the CLI:** `cargo xtask help` or `cargo xtask help-flows`
5. **Open the UI:** `http://localhost:8080/ui` for visual navigation
6. **Check platform status:** `curl http://localhost:8080/platform/status`
7. **Get task suggestions:** `curl http://localhost:8080/platform/agent/hints`
8. **Open an issue** if documentation is missing or unclear

---

## Documentation Statistics

**Total documents:** 120+ markdown files
**Categories:**
- Getting Started: 5 docs
- How-To Guides: 13 docs
- Tutorials: 5 docs
- Reference: 7 docs
- Explanations: 11 docs
- Design Docs: 22 docs
- ADRs: 9 docs
- Templates: 8 docs

**Last updated:** 2025-12-31 (v3.3.14)
