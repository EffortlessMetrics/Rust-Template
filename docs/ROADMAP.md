# Roadmap: Self-Healing Platform Cell

## Vision

This template is a **self-healing platform cell**: a governed Rust service that enforces its own integrity through executable contracts and surfaces that state via multiple interfaces (CLI, HTTP API, Web UI, and agent skills).

Unlike conventional templates that drift from their documented behavior over time, this repository maintains a living contract between **what the code promises** (specs) and **what it delivers** (runtime), enforced automatically by CI.

---

## Current State: v2.4.0 – "The Governing Kernel"

**Release Date:** 2025-Q1

### What We Built

Phase 1-4 established the **governance foundation**:

1. **Specs as Code** ([`specs/spec_ledger.yaml`](file:///home/steven/code/Rust/Rust-Template/specs/spec_ledger.yaml))
   - Stories → Requirements → Acceptance Criteria as structured YAML
   - Linked to ADRs, BDD scenarios, design docs
   - 17 requirements with mandatory AC coverage (`must_have_ac: true`)

2. **DevEx as Spec** ([`specs/devex_flows.yaml`](file:///home/steven/code/Rust/Rust-Template/specs/devex_flows.yaml))
   - Developer workflows (onboarding, AC-first, release) as executable flows
   - All `xtask` commands mapped to flows and use cases
   - DevEx contract enforced by selftest

3. **Docs as Spec** ([`specs/doc_index.yaml`](file:///home/steven/code/Rust/Rust-Template/specs/doc_index.yaml))
   - Documentation inventory with front-matter validation
   - Requirement ↔ design doc linkage tracked
   - Policy enforcement via OPA/Rego

4. **Tasks as Spec** ([`specs/tasks.yaml`](file:///home/steven/code/Rust/Rust-Template/specs/tasks.yaml))
   - Concrete work items with recommended flows
   - Context-aware `suggest-next` showing satisfied vs pending steps
   - Agent-friendly guidance for LLM-driven workflows

5. **Self-Governing Infrastructure**
   - **7-Step Selftest** as mandatory CI gate:
     1. Core checks (fmt, clippy, tests)
     2. BDD acceptance tests (specs → runtime behavior)
     3. AC status mapping & ADR references (traceability)
     4. LLM context bundler (safe AI augmentation)
     5. Policy tests (compliance via OPA/Rego)
     6. DevEx contract (required commands exist)
     7. Graph invariants (structural integrity)
   - **Graph invariants** prevent orphaned requirements, missing ACs, unreachable commands
   - **Real-time policy status** exposed via `/platform/status`

6. **Platform Introspection APIs** ([`/platform/*`](http://localhost:8080/platform/status))
   - `/platform/status` – Governance health metrics
   - `/platform/graph` – Full governance graph as JSON
   - `/platform/tasks` – Work queue for agents and humans
   - `/platform/tasks/suggest-next?task=X` – Context-aware guidance
   - `/platform/devex/flows` – Developer workflows
   - `/platform/docs/index` – Documentation inventory

7. **Web UI** ([`/ui`](http://localhost:8080/ui))
   - Rust-native governance console (maud + htmx, zero build step)
   - Dashboard: Platform health + policy status + governance contracts
   - Graph: Interactive Mermaid visualization
   - Flows: DevEx workflows + task guidance

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

## Pilot Phase: v2.4.x – "Learning from Reality"

**Timeline:** 2025-Q1 to Q2  
**Goal:** Validate that the governance kernel reduces friction in real service development

### Pilot Workflow

We're intentionally **limiting the scope** to a small, controlled pilot:

1. **Select 2-3 pilot services** (internal teams only)
2. **Clone this template** and implement 3-5 features using the AC-first workflow
3. **Maintain a friction log** ([`FRICTION_LOG.md`](file:///home/steven/code/Rust/Rust-Template/docs/templates/FRICTION_LOG.md))
4. **Weekly sync** to surface blockers
5. **After 4-6 weeks**: Retrospective to decide future direction

### Success Criteria

**Hard Requirements:**
- All 7 selftest steps pass on every PR
- Friction log is actively maintained (not abandoned)
- At least 10 ACs implemented end-to-end

**Qualitative Goals:**
- Developers report that `suggest-next` is helpful, not annoying
- Time-to-onboard for new team members decreases
- Governance becomes "background" rather than "blocker"

### Known Risks

> [!WARNING]
> **This is experimental.** The governance model may be too heavy for certain workflows.

**Risk 1: Governance Overhead**
- **Symptom**: Developers bypass specs/BDD to move faster
- **Mitigation**: Friction log captures this; we adjust or simplify

**Risk 2: Tool Complexity**
- **Symptom**: `xtask` commands feel opaque or fragile
- **Mitigation**: Improve help text, error messages, or consolidate commands

**Risk 3: False Positives**
- **Symptom**: Selftest fails on valid changes (e.g., graph invariants too strict)
- **Mitigation**: Refine invariant rules based on real usage

---

## Future Direction (Post-Pilot)

### If Pilot Succeeds: v2.5.0 – "The Agent Interface"

**Phase 5: Agent-Native Skills**
- **`.claude/skills/`**: Formal LLM skills for:
  - `governed-feature-dev` (AC → BDD → code → selftest)
  - `governed-release` (changelog → tag → deploy)
  - `governed-maintenance` (audit → upgrade → verify)
- **`docs/AGENT_GUIDE.md`**: Pure agent directive (replaces `CLAUDE.md`)
- **Enhanced UI Tier 2**:
  - Kanban board (columns derived from AC/test status)
  - Code tree view showing AC ↔ module mappings
  - Inline spec editing with patch generation

**Phase 6: Multi-Tenant Governance Dashboard**
- **Backstage Plugin**: Integrate `/platform/*` APIs into Backstage
- **Fleet View**: Cross-service governance health dashboard
- **Drift Detection**: Alert when cloned services diverge from template contracts

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

**v2.4.0 is complete.** It contains:
- 7-step selftest enforcing specs/docs/policies/graph
- Platform introspection APIs + Web UI
- Context-aware task guidance

**The pilot will determine:**
- Is the governance model the right weight?
- Which features are essential vs nice-to-have?
- Whether agents can effectively use this as an interface

**The roadmap remains flexible** until pilot data validates the direction.

**Next milestone:** First pilot team completes initial 10 ACs using the template (ETA: 2025-Q1 end).
