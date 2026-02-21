# Template Version: v3.3.8
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance graph invariants

  @AC-TPL-GRAPH-INVARIANTS
  Scenario: graph-export contains requirement, AC, test, and command-flow links
    Given low resources are disabled
    When I run "cargo xtask graph-export"
    Then the command should succeed
    And the output should contain "\"id\": \"REQ-PLT-ONBOARDING\""
    And the output should contain "\"id\": \"AC-PLT-001\""
    And the output should contain "\"id\": \"AC-PLT-001:test:0\""
    And the output should contain "\"source\": \"flow:discovery\""
    And the output should contain "\"target\": \"cmd:graph-export\""

  @AC-TPL-GRAPH-REQ-HAS-AC
  Scenario: graph-export links requirements to ACs
    Given low resources are disabled
    When I run "cargo xtask graph-export"
    Then the command should succeed
    And the output should contain "\"type\": \"contains\""

  @AC-TPL-GRAPH-AC-HAS-TEST
  Scenario: graph-export links ACs to test nodes
    Given low resources are disabled
    When I run "cargo xtask graph-export"
    Then the command should succeed
    And the output should contain "\"type\": \"tested_by\""

  @AC-TPL-GRAPH-COMMAND-REACHABLE
  Scenario: graph-export shows commands reachable from flows
    Given low resources are disabled
    When I run "cargo xtask graph-export"
    Then the command should succeed
    And the output should contain "\"type\": \"executes\""

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
