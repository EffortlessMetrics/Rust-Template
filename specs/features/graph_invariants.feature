# Template Version: v3.0.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance graph invariants

  @AC-TPL-GRAPH-REQ-HAS-AC
  Scenario: Requirements with must_have_ac have at least one AC
    When I run "cargo xtask graph-export --check"
    Then the command succeeds
    And the output contains "Graph invariants satisfied"

  @AC-TPL-GRAPH-COMMAND-REACHABLE
  Scenario: Required DevEx commands are reachable
    When I run "cargo xtask graph-export --check"
    Then the command succeeds
    And the output contains "Graph invariants satisfied"

  @AC-TPL-GRAPH-SELFTEST
  Scenario: Selftest enforces graph invariants
    When I run "cargo xtask selftest"
    Then the command succeeds
    And the output contains "Checking governance graph invariants"
    And the output contains "Graph invariants satisfied"
