# Template Version: v3.3.8
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance graph invariants

  @AC-TPL-GRAPH-SELFTEST @ci-only
  Scenario: Selftest validates graph invariants
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest"
    Then the command succeeds
    And the output contains "Checking governance graph invariants"
    And the output contains "Graph invariants satisfied"

  @AC-TPL-GRAPH-INVARIANTS @ci-only
  Scenario: Graph invariants enforce coverage for requirements, commands, and AC tests
    # Note: Marked @ci-only to avoid recursive selftest within selftest BDD step
    When I run "cargo xtask selftest" with low-resource mode
    Then the command should succeed
    And the output should contain "Graph invariants satisfied"
