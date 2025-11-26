# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance graph invariants

  @AC-TPL-GRAPH-SELFTEST
  Scenario: Selftest validates graph invariants
    When I run "cargo xtask selftest"
    Then the command succeeds
    And the output contains "Checking governance graph invariants"
    And the output contains "Graph invariants satisfied"

  @AC-TPL-GRAPH-REQ-HAS-AC @AC-TPL-GRAPH-COMMAND-REACHABLE @AC-TPL-GRAPH-AC-HAS-TEST
  Scenario: Graph invariants enforce coverage for requirements, commands, and AC tests
    When I run "cargo xtask selftest" with low-resource mode
    Then the command should succeed
    And the output should contain "Graph invariants satisfied"
