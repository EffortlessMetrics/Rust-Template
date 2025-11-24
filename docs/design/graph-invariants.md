---
doc_type: design_doc
id: DESIGN-TPL-GRAPH-INVARIANTS-001
title: "Governance Graph Structural Invariants"
stories: ["US-TPL-PLT-001"]
requirements: ["REQ-TPL-GRAPH-INVARIANTS"]
acs:
  - AC-TPL-GRAPH-REQ-HAS-AC
  - AC-TPL-GRAPH-AC-HAS-TEST
  - AC-TPL-GRAPH-COMMAND-REACHABLE
  - AC-TPL-GRAPH-SELFTEST
adrs: ["ADR-0001", "ADR-0005"]
status: approved
last_reviewed: 2025-11-22
owner: "platform"
---

# Governance Graph Structural Invariants

## 1. Context

The Rust-as-Spec platform builds a governance graph from spec_ledger.yaml, devex_flows.yaml, and doc_index.yaml that represents relationships between stories, requirements, acceptance criteria, tests, commands, and documentation. Without structural validation, this graph can accumulate drift: requirements without ACs, ACs without tests, orphaned commands, and broken documentation links. This drift undermines the "specs as contracts" principle and makes governance enforcement unreliable.

The requirement REQ-TPL-GRAPH-INVARIANTS addresses this by defining baseline structural invariants that must hold for the graph to be considered valid, with validation enforced automatically during `cargo xtask selftest`.

## 2. High-Level Design

The graph invariant system validates three core structural properties:

**Invariant 1: Requirements must have ACs**
Every requirement tagged with platform, structural, security, devex, docs, or release must have at least one acceptance criterion. This ensures that high-impact requirements are testable and not just prose.

**Invariant 2: ACs with test mappings must have test nodes**
Every AC that declares a tests mapping in spec_ledger.yaml must have at least one corresponding test node in the governance graph. This catches broken test references and missing BDD scenarios.

**Invariant 3: Commands must be reachable**
Every command declared in specs/devex_flows.yaml must either be referenced by at least one flow or explicitly marked as internal. This prevents accumulation of orphaned commands that were once used but are no longer part of any workflow.

The validation is implemented in `crates/spec-runtime/src/graph.rs` as a `validate_invariants()` function that returns a structured result with violation details. The `cargo xtask graph-export --check` command runs validation and reports violations, while `cargo xtask selftest` includes graph invariant validation as step 7 of its contract.

## 3. Edge Cases & Failure Modes

**Violation reporting**: When an invariant fails, the system reports which specific requirement, AC, or command is in violation, along with actionable guidance (e.g., "Add at least one AC to REQ-TPL-EXAMPLE" or "Reference command 'orphan-cmd' in a flow or mark it internal").

**Partial failures**: Graph invariant validation is all-or-nothing; any single violation fails the entire check. This prevents "mostly correct" graphs from passing governance gates.

**Performance**: Graph validation runs in memory after loading specs, adding negligible overhead (< 100ms for typical template-sized graphs).

**False positives**: Requirements tagged as `future` or with `must_have_ac: false` are exempt from Invariant 1, allowing intentional spec-before-implementation documentation.

## 4. Tests & Invariants

The design is validated by:

- **AC-TPL-GRAPH-REQ-HAS-AC**: BDD scenario verifies that requirements with must_have_ac=true and relevant tags fail validation if they lack ACs (test tag: `graph_invariants_req_has_ac`).
- **AC-TPL-GRAPH-AC-HAS-TEST**: Unit test confirms that ACs with tests[] mappings produce corresponding test nodes and edges in the graph (test tag: `ac_with_tests_produces_graph_node_and_edge`).
- **AC-TPL-GRAPH-COMMAND-REACHABLE**: BDD scenario validates that orphaned commands cause validation to fail (test tag: `graph_invariants_command_reachable`).
- **AC-TPL-GRAPH-SELFTEST**: Integration test ensures `cargo xtask selftest` includes graph invariant validation and fails when violations exist (test tag: `graph_export_mermaid`).

Critical invariants enforced by this design:
1. No requirement bypass: Platform/structural requirements cannot skip ACs
2. Test traceability: AC -> test mappings are structurally sound
3. Workflow completeness: All declared commands serve a purpose

## 5. Open Questions / Future Work

**Future extensions (explicit non-goals for v1):**
- Validate ADR references (ensure ADR IDs in spec_ledger.yaml correspond to actual ADR files)
- Enforce documentation coverage (ensure every requirement has at least one design doc or impl plan, beyond just policy checks)
- Detect circular dependencies in flows (e.g., flow A references flow B which references flow A)
- Performance optimization for large graphs (100+ stories)

**Deferred:**
- Graph visualization of violations (currently text-based reporting only)
- Auto-remediation suggestions (e.g., "Run `cargo xtask ac-new REQ-TPL-EXAMPLE ...`")
