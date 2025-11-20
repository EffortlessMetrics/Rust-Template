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
