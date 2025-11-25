# Roadmap: Self-Healing Platform Cell

## Vision

This template is a **self-healing platform cell**: a governed Rust service that enforces its own integrity through executable contracts and surfaces that state via multiple interfaces (CLI, HTTP API, Web UI, and agent skills).

Unlike conventional templates that drift from their documented behavior over time, this repository maintains a living contract between **what the code promises** (specs) and **what it delivers** (runtime), enforced automatically by CI.

It is deliberately **LLM-native and governance-bounded**: agents handle the mechanical work through named flows, deterministic gates (`cargo xtask selftest`, policy tests, graph invariants) keep changes honest, and humans still merge with evidence.

---

   - Concrete work items with recommended flows
   - Context-aware `suggest-next` showing satisfied vs pending steps
   - Agent-friendly guidance for LLM-native, governance-bounded workflows

5. **Self-Governing Infrastructure**
   - **7-Step Selftest** as mandatory CI gate:
     1. Core checks (fmt, clippy, tests)
     2. BDD acceptance tests (specs → runtime behavior)
     3. AC status mapping & ADR references (traceability)
     4. LLM context bundler (governance-bounded agent work)
     5. Policy tests (compliance via OPA/Rego)
     6. DevEx contract (required commands exist)
     7. Graph invariants (structural integrity)
   - **Graph invariants** prevent orphaned requirements, missing ACs, unreachable commands
   - **Real-time policy status** exposed via `/platform/status`

6. **Platform Introspection APIs** ([`/platform/*`](http://localhost:8080/platform/status))
   - `/platform/status` – Governance health metrics
   - `/platform/graph` – Full governance graph as JSON
   - `/platform/tasks` – Work queue for agents and humans
   - `/platform/agent/hints` – Suggested next tasks for agents
   - `/platform/devex/flows` – Developer workflows
   - `/platform/docs/index` – Documentation inventory

7. **Web UI** ([`/ui`](http://localhost:8080/ui))
   - Rust-native governance console (maud + htmx, zero build step)
   - Dashboard: Platform health + policy status + governance contracts
   - Graph: Interactive Mermaid visualization
   - Flows: DevEx workflows + task guidance
   - Tasks: Kanban board with task lifecycle management

### Key Architectural Decisions

- **Single source of truth**: YAML specs → Rust runtime → HTTP API → UI (no drift possible)
- **Type-safe everything**: Specs deserialized via `serde`, validated at compile time
- **Self-healing**: UI/API data derived from same loaders/validators that power CI
- **Zero external dependencies**: UI compiles into binary, no npm/webpack/SPA complexity

### What This Enables

**For Platform Teams:**
- Clone → `nix develop` → `cargo run` → working governed service in 5 minutes
- Governance enforced automatically; impossible to merge broken specs/policies
- Real-time visibility into compliance via dashboard

**For Developers:**
- `cargo xtask suggest-next --task implement_ac` tells you exactly what to do
- Specs stay current because CI fails if they drift
- Documentation validated as rigorously as code

**For Agents (LLMs):**
- Structured `/platform/*` APIs provide bounded context windows
- `suggest-next` provides deterministic "next step" guidance
- Selftest provides instant validation of agent-generated changes

---

## Pilot Status: v3.0.0 – Completed

**Timeline:** 2025-Q1
**Status:** ✅ Pilot phase complete; governance model validated

### What Was Built
- **7-step selftest**: Core checks, BDD, AC mapping, bundler, policies, DevEx contract, graph invariants
- **Platform APIs**: Read-only governance introspection + write-path task management
- **Web UI**: Dashboard, graph visualization, Kanban board, flow guidance
- **Agent Loop**: Task hints, status updates, low-resource mode
- **DevEx Polish**: `dev-up`, `status`, selftest summary, standardized ports

### Lessons Learned
- **Friction log was critical**: Captured real pain points and drove polish
- **Selftest summary matters**: Condensed output reduced cognitive load
- **Tasks as first-class entities**: Game-changer for agent workflows
- **Port consistency**: Small details (3000 vs 8080) cause disproportionate friction
- **One-command setup**: `dev-up` dramatically improved first-run experience

### Success Metrics Met
- ✅ All 7 selftest steps pass consistently
- ✅ Friction log actively maintained with 12+ resolved entries
- ✅ 15+ ACs implemented end-to-end across platform features
- ✅ Agent integration validated (task hints, status API, bundler)
- ✅ Onboarding time reduced from ~30min to ~5min (via `dev-up`)

### Risks That Materialized
- **Tool complexity**: Addressed via help text improvements and `suggest-next`
- **Cognitive overhead**: Mitigated via selftest summary and status dashboard
- **Port confusion**: Resolved via standardization to 3000

---

---

## Current State: v3.0.0 "Living Platform" (Complete ✅)

**Goal:** Transform from "Code that Agents help write" to "An Environment where Agents live."

### Sprint 1: The Write Layer (Completed ✅)
- [x] **Governance Repository**: Abstract trait for state persistence
- [x] **FS Adapter**: `adapters-spec-fs` with file locking and `tasks_state.yaml`
- [x] **Wiring**: `app-http` and acceptance tests wired to use the repository
- [x] **Verification**: BDD scenario for task persistence

### Sprint 2: Domain Rules (Completed ✅)
- [x] **Task Status Machine**: Enforce valid transitions (Todo -> InProgress -> Review -> Done)
- [x] **Task <-> Requirement Linking**: Domain model for traceability
- [x] **Update Use Case**: Pure business logic for status updates via `TaskService`

### Sprint 3: API & UI (Completed ✅)
- [x] **Write API**: `POST /platform/tasks/{id}/status` for agents and humans
- [x] **Kanban Board**: Interactive UI at `/ui/tasks` with drag-and-drop
- [x] **Agent Hints**: `GET /platform/agent/hints` returns prioritized task suggestions

### v3.0.0 Polish (Completed ✅)
- [x] **Dev-up Command**: `cargo xtask dev-up` for one-command environment setup
- [x] **Status Dashboard**: `cargo xtask status` shows governance health at a glance
- [x] **Selftest Summary**: Condensed output with clear pass/fail indicators
- [x] **Low-Resource Mode**: `XTASK_LOW_RESOURCES=1` for CI/constrained environments
- [x] **Port Standardization**: All services on `localhost:8080` (was 3000)
- [x] **Friction Log Integration**: Resolved entries tracked in governance

---

## Upcoming: v3.1.x "Release & Versioning"

**Goal:** Make versions first-class in governance and enable evidence-based changelog generation.

### Motivation
Currently, versions exist in `Cargo.toml` and metadata, but:
- No bounded evidence file per release
- Changelog generation is manual
- Hard to answer "what's in v3.1.0?" without git archaeology

### Planned Features
- **Release Evidence Bundle**: `cargo xtask release-bundle X.Y.Z` generates a bounded markdown file:
  - All tasks completed in this version
  - Related REQs/ACs and their status
  - ADRs introduced or updated
  - Git log of commits since last tag
  - Selftest summary at release time
  - Policy compliance status
  - Resolved friction log entries
- **LLM-Native Changelogs**: Feed evidence bundle to LLM to generate Keep a Changelog format entries
- **Version-Aware Specs**: Tasks and ACs can reference target version

### Acceptance Criteria
- `REQ-TPL-REL-BUNDLE` – Release evidence bundle requirement
- `AC-TPL-REL-EVIDENCE` – `cargo xtask release-bundle X.Y.Z` writes to `release_evidence/vX.Y.Z.md`
- `AC-TPL-REL-CHANGELOG` – Generated evidence includes all required sections (tasks, specs, git, selftest)

### Benefits
- Traceable: Every release has a provable evidence file
- Auditable: Compliance teams can verify what changed
- Agent-friendly: Structured data for LLM changelog generation
- Historical: Evidence files are committed, not regenerated

---

## Completed: v3.2.0 "Skills & AC Tooling" ✅

**Goal:** Explicit mapping between DevEx flows and Agent Skills, with repo-specific authoring guidance, plus enhanced AC coverage tooling.

### What Was Delivered
- **Skills Documentation**: `docs/AGENT_SKILLS.md` with repo-specific guidance
  - Recommended Skill set aligned with `specs/devex_flows.yaml`
  - SKILL.md templates for consistent authoring
  - Governance for Skill creation (REQ/AC/Task, not ad-hoc)
- **Skills Tooling**:
  - `cargo xtask skills-fmt` to normalize SKILL.md formatting
  - `cargo xtask skills-lint` to validate Skills definitions
  - Skills are now governed artifacts with standardized structure
- **AC Coverage Tooling**:
  - `cargo xtask ac-coverage` for AC coverage summary grouped by requirement
  - `cargo xtask ac-suggest-scenarios` to generate BDD scenario stubs
  - Shared `ac_parsing` module for consistent parsing across commands
- **Module Wiring**:
  - Fixed all AC-related module exports (`ac_parsing`, `ac_coverage`, `ac_suggest_scenarios`)
  - Complete workflow from coverage analysis to scenario generation

### Skills Implemented
1. **bootstrap-dev-env**: Uses `dev-up`, `status`, `/platform/status` for first-time setup ✅
2. **governed-feature-dev**: AC-first workflow using `ac-new`, `bundle`, `bdd`, `selftest`, task API ✅
3. **governed-maintenance**: Policy/dependency/docs updates using `policy-test`, `audit`, `check` ✅
4. **governed-release**: Release workflow using `release-prepare`, `release-bundle`, `release-verify` ✅
5. **governed-governance-debug**: Diagnose selftest failures using summary, graph, policies ✅

### Acceptance Criteria Met
- ✅ `REQ-TPL-SKILLS-GUIDE` – Documented pattern for Skills in this repo
- ✅ `AC-TPL-SKILLS-GUIDE-001` – `docs/AGENT_SKILLS.md` exists with templates and flow mappings
- ✅ `AC-TPL-SKILLS-ALIGN-001` – Existing `.claude/skills/*` align with documented patterns
- ✅ `REQ-TPL-SKILLS-TOOLING` – Skills formatting and linting commands exist
- ✅ `AC-TPL-SKILLS-FMT` – Skills can be normalized with `skills-fmt`
- ✅ `AC-TPL-SKILLS-LINT` – Skills validated with `skills-lint`

### Benefits Realized
- Clarity: Agents and humans know which Skills exist and when to use them
- Governance: Skills follow same AC-first rigor as code
- Alignment: Skills match actual workflows from `devex_flows.yaml`
- Maintainability: Skills are documented, tested, and traceable
- Efficiency: AC coverage workflow streamlined from analysis to scenario generation

---

## v3.0.0 - Production-Ready Rust IDP Template

**Status:** Target for completion

### Kernel AC Coverage

Every kernel AC (requirements marked `must_have_ac: true` and `kernel: true` tag) has:
- ✅ Passing BDD scenario coverage OR unit test coverage
- ✅ Traceability from REQ → AC → Test → Code
- ✅ Validated by `cargo xtask ac-coverage`

**Current kernel coverage:** 0/50 ACs passing (run `cargo xtask ac-coverage` for live status)

### What "Green Out of the Box" Means

1. **Onboarding**: `doctor`, `help-flows`, `check`, `dev-up` guide new users
2. **Development**: `ac-new`, `adr-new`, skills-based workflows
3. **Validation**: `selftest` enforces governance contracts
4. **Release**: `release-prepare`, `release-verify`, `release-bundle` with evidence
5. **Security**: `audit`, `sbom-local` supply-chain visibility
6. **Observability**: `status`, `/platform/ui` real-time governance health

Non-kernel ACs (future features, nice-to-haves) are tracked but don't block release.

### Success Criteria

- [ ] All kernel ACs green in `cargo xtask ac-coverage`
- [ ] `cargo xtask selftest` passes on Linux, macOS, Windows
- [ ] Documentation complete (README, AGENT_GUIDE, this ROADMAP)
- [ ] Release evidence bundle generated

---

## Future: v3.3+ "Adoption & Fleet View"

**Goal:** Enable brownfield adoption and multi-cell observability.

### Brownfield Kernel Extraction
- Extract core governance crates for reuse in existing services:
  - `spec-runtime` (standalone spec loading/validation)
  - `governance-core` (AC/REQ domain models)
  - `policy-runtime` (OPA/Rego integration)
- Let teams adopt incrementally (specs only → BDD → graph → full stack)
- Provide migration guides for non-template services

### Multi-Cell Observatory
- **Backstage Plugin**: Integrate `/platform/*` APIs into Backstage
- **Fleet Dashboard**: Cross-service governance health view
- **Drift Detection**: Alert when cloned services diverge from template contracts
- **Aggregated Metrics**: Track governance adoption across multiple cells

### Enhanced UI (Optional)
- Code tree view showing AC ↔ module mappings
- Inline spec editing with patch generation
- Cross-cell navigation and comparison

### If Pilot Struggles: Simplify or Pivot

**Option A: Lighter Governance** (if overhead is too high)
- Remove graph invariants or `must_have_ac` constraints
- Make BDD optional
- Keep only core selftest steps (fmt, clippy, tests, policies)

**Option B: Different Target** (if unsuitable for general services)
- Position as "regulated services only" (FinTech, HealthTech)
- Build vertical-specific policy packs (PCI-DSS, HIPAA, SOC2)
- Partner with compliance teams, not general platform teams

**Option C: Extract Components** (if full stack is too coupled)
- Publish `spec-runtime` as standalone crate
- Publish policy packs separately
- Let teams adopt incrementally (specs only, then BDD, then graph)

---

## Principles (Invariant Across All Futures)

Regardless of pilot outcome, we maintain these constraints:

1. **Specs as Source of Truth**
   - Code that doesn't match specs fails CI
   - Specs that don't match code fail CI
   - There is no third state

2. **Self-Healing First**
   - UI never maintains state; it projects specs/runtime
   - Policies auto-update from file changes
   - Graph invariants catch drift, not humans

3. **Agent-Compatible by Default**
   - All workflows exposed as `/platform/*` APIs
   - All guidance structured (not prose)
   - All validation deterministic (no "vibes-based" checks)

4. **Zero Vendor Lock**
   - Pure Rust; no proprietary runtimes
   - No SaaS dependencies
   - Cloneable, forkable, offline-capable

---

## How to Contribute to the Roadmap

During the pilot, this roadmap is **living**. We update it based on friction log entries.

**Process:**
1. Friction log entry created
2. Weekly triage: Is this a blocker, nice-to-have, or won't-fix?
3. Blockers → immediate patch release (v2.4.x)
4. Nice-to-haves → roadmap for v2.5.0
5. Won't-fix → document as "anti-pattern" in runbooks

**Decision Framework:**
- **Does it reduce cognitive load?** → Consider
- **Does it increase spec/code drift risk?** → Reject
- **Does it make agents smarter?** → Prioritize
- **Does it require new state/config?** → Scrutinize

---

## Appendix: Technology Choices

### Why Rust?
- **Type safety**: Specs are validated at compile time
- **Performance**: CLI tools are instant; no Node.js startup tax
- **Single binary**: `app-http` bundles UI + API + business logic

### Why YAML (not JSON/TOML/custom DSL)?
- **Readable**: Non-developers can edit specs
- **Comments**: Inline documentation stays with the spec
- **Ecosystem**: `serde_yaml` is mature and well-tested

### Why OPA/Rego for Policies?
- **Declarative**: Policy logic separated from enforcement
- **Testable**: `conftest` enables policy unit tests
- **Industry standard**: Used by Kubernetes, Envoy, Terraform

### Why Maud (not Askama/Tera)?
- **Compile-time HTML checking**: Typos caught by compiler
- **Zero runtime cost**: Templates are Rust macros
- **Type-safe**: Data models are enforced

### Why No GraphQL/gRPC for `/platform/*`?
- **Simplicity**: REST + JSON is universal
- **Introspectable**: `curl` works; no special clients needed
- **Agent-friendly**: LLMs understand JSON natively

---

## Summary

**Current state:**
- ✅ v3.2.0 complete: Skills, AC tooling, release bundle
- ✅ Platform infrastructure: 7-step selftest, APIs, Web UI, agent loop
- ✅ DevEx foundation: `dev-up`, `status`, selftest summary, port 3000

**Production readiness (v3.0.0 milestone):**
- 🎯 **Kernel AC Coverage**: 0/50 ACs passing (target: 100%)
- 🎯 Every kernel AC has passing BDD scenario or unit test coverage
- 🎯 Full traceability: REQ → AC → Test → Code
- 🎯 Cross-platform selftest pass (Linux, macOS, Windows)

**Completed milestones:**
- ✅ **v3.1.x**: Release evidence bundles and LLM-native changelog generation
- ✅ **v3.2.x**: Agent Skills documentation and flow-to-Skill alignment

**Next milestone:** Complete kernel AC coverage for v3.0.0 production-ready release.

**Pilot validated:**
- ✅ Governance model is the right weight (with polish)
- ✅ Agents can effectively use the platform APIs
- ✅ Selftest + friction log drives continuous improvement

