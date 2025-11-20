# Feature Status

Auto-generated AC status from acceptance tests.

## AC Status Summary

| AC ID | Story | Requirement | Status | Scenarios |
|-------|-------|-------------|--------|----------|
| AC-PLT-001 | US-TPL-PLT-001 | REQ-PLT-ONBOARDING | ❓ unknown | 0 |
| AC-PLT-002 | US-TPL-PLT-001 | REQ-PLT-ONBOARDING | ❓ unknown | 0 |
| AC-PLT-003 | US-TPL-PLT-001 | REQ-PLT-ONBOARDING | ❓ unknown | 0 |
| AC-PLT-004 | US-TPL-PLT-001 | REQ-PLT-DESIGN-SCAFFOLDING | ❓ unknown | 0 |
| AC-PLT-005 | US-TPL-PLT-001 | REQ-PLT-DESIGN-SCAFFOLDING | ❓ unknown | 0 |
| AC-PLT-006 | US-TPL-PLT-001 | REQ-PLT-SECURITY-GOVERNANCE | ❓ unknown | 0 |
| AC-PLT-007 | US-TPL-PLT-001 | REQ-PLT-SECURITY-GOVERNANCE | ❓ unknown | 0 |
| AC-PLT-008 | US-TPL-PLT-001 | REQ-PLT-SECURITY-GOVERNANCE | ❓ unknown | 0 |
| AC-PLT-009 | US-TPL-PLT-001 | REQ-PLT-DOCS-CONSISTENCY | ❓ unknown | 0 |
| AC-PLT-010 | US-TPL-PLT-001 | REQ-PLT-DOCS-CONSISTENCY | ❓ unknown | 0 |
| AC-PLT-011 | US-TPL-PLT-001 | REQ-PLT-RELEASE-SAFETY | ❓ unknown | 0 |
| AC-PLT-012 | US-TPL-PLT-001 | REQ-PLT-RELEASE-SAFETY | ❓ unknown | 0 |
| AC-PLT-013 | US-TPL-PLT-001 | REQ-PLT-RELEASE-SAFETY | ❓ unknown | 0 |
| AC-PLT-014 | US-TPL-PLT-001 | REQ-PLT-DEVEX-CONTRACT | ❓ unknown | 0 |
| AC-PLT-015 | US-TPL-PLT-001 | REQ-PLT-DEVEX-CONTRACT | ❓ unknown | 0 |
| AC-PLT-016 | US-TPL-PLT-001 | REQ-PLT-DEVEX-CONTRACT | ❓ unknown | 0 |
| AC-TPL-001 | US-TPL-001 | REQ-TPL-HEALTH | ✅ pass | 1 |
| AC-TPL-002 | US-TPL-001 | REQ-TPL-VERSION | ✅ pass | 1 |
| AC-TPL-003 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 1 |
| AC-TPL-004 | US-TPL-001 | REQ-TPL-ERROR-HANDLING | ✅ pass | 3 |
| AC-TPL-007 | US-TPL-001 | REQ-TPL-METRICS | ✅ pass | 1 |
| AC-TPL-GRAPH-AC-HAS-TEST | US-TPL-PLT-001 | REQ-TPL-GRAPH-INVARIANTS | ❓ unknown | 0 |
| AC-TPL-GRAPH-COMMAND-REACHABLE | US-TPL-PLT-001 | REQ-TPL-GRAPH-INVARIANTS | ❌ fail | 1 |
| AC-TPL-GRAPH-MERMAID | US-TPL-PLT-001 | REQ-TPL-GRAPH-VISUALIZATION | ✅ pass | 1 |
| AC-TPL-GRAPH-REQ-HAS-AC | US-TPL-PLT-001 | REQ-TPL-GRAPH-INVARIANTS | ❌ fail | 1 |
| AC-TPL-GRAPH-SELFTEST | US-TPL-PLT-001 | REQ-TPL-GRAPH-INVARIANTS | ❌ fail | 1 |
| AC-TPL-PLATFORM-DEVEX | US-TPL-PLT-001 | REQ-TPL-PLATFORM-INTROSPECTION | ✅ pass | 1 |
| AC-TPL-PLATFORM-DOCS | US-TPL-PLT-001 | REQ-TPL-PLATFORM-INTROSPECTION | ✅ pass | 1 |
| AC-TPL-PLATFORM-GRAPH | US-TPL-PLT-001 | REQ-TPL-PLATFORM-INTROSPECTION | ✅ pass | 1 |
| AC-TPL-PLATFORM-UI-DASHBOARD | US-TPL-PLT-001 | REQ-TPL-PLATFORM-UI | ✅ pass | 1 |
| AC-TPL-PLATFORM-UI-FLOWS | US-TPL-PLT-001 | REQ-TPL-PLATFORM-UI | ✅ pass | 1 |
| AC-TPL-PLATFORM-UI-GRAPH | US-TPL-PLT-001 | REQ-TPL-PLATFORM-UI | ✅ pass | 1 |
| AC-TPL-POLICY-STATUS-OVERVIEW | US-TPL-PLT-001 | REQ-TPL-PLATFORM-INTROSPECTION | ✅ pass | 1 |
| AC-TPL-SUGGEST-NEXT-CLI | US-TPL-PLT-001 | REQ-TPL-SUGGEST-NEXT | ✅ pass | 1 |
| AC-TPL-SUGGEST-NEXT-HTTP | US-TPL-PLT-001 | REQ-TPL-SUGGEST-NEXT | ✅ pass | 1 |
| AC-TPL-TASKS-CLI | US-TPL-PLT-001 | REQ-TPL-PLATFORM-TASKS | ✅ pass | 1 |
| AC-TPL-TASKS-HTTP | US-TPL-PLT-001 | REQ-TPL-PLATFORM-TASKS | ✅ pass | 1 |

## Unmapped ACs

ACs with no mapped scenarios:

- AC-PLT-001: `cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance
- AC-PLT-002: `cargo xtask help-flows` renders categorized command map from specs/devex_flows.yaml
- AC-PLT-003: `cargo xtask check` runs fmt + clippy + tests as fast dev loop
- AC-PLT-004: `cargo xtask adr-new <title>` creates numbered ADR from template with metadata
- AC-PLT-005: `cargo xtask ac-new <ID> <desc>` rejects duplicate IDs and generates YAML snippet
- AC-PLT-006: `cargo xtask audit` runs cargo-audit + cargo-deny with repo policy (deny.toml)
- AC-PLT-007: `cargo xtask audit` provides 4-step recovery guidance on failure
- AC-PLT-008: `cargo xtask sbom-local` generates SPDX JSON to target/sbom.spdx.json
- AC-PLT-009: `cargo xtask docs-check` validates version alignment across spec_ledger, README, CLAUDE
- AC-PLT-010: `cargo xtask docs-check` regenerates feature_status and fails on dirty git tree
- AC-PLT-011: `cargo xtask release-prepare X.Y.Z` updates spec_ledger, README, CLAUDE, CHANGELOG
- AC-PLT-012: `cargo xtask release-verify` runs selftest + audit + docs-check + clean tree
- AC-PLT-013: `cargo xtask release-verify` provides git command sequence on success
- AC-PLT-014: Canonical flows and commands are defined in specs/devex_flows.yaml
- AC-PLT-015: `cargo xtask selftest` enforces devex contract (required commands exist)
- AC-PLT-016: `cargo xtask ci-local` orchestrates doctor + selftest + audit + docs-check
- AC-TPL-GRAPH-AC-HAS-TEST: Every AC with a tests mapping in spec_ledger.yaml has at least one test node linked in the graph.


## Unmapped Scenarios

Scenarios referencing non-existent ACs:

- Scenario 'Platform status returns governance summary' references AC-TPL-PLATFORM-STATUS (in home/steven/code/Rust/Rust-Template/specs/features/platform_rounding.feature)
