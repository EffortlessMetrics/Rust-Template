# Template Version: v3.0.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Governance Graph Visualization
  The template must provide a Mermaid visualization of the governance graph.

  Background:
    Given I am in a Rust-Template workspace
    And I am in a Nix devshell

  @AC-TPL-GRAPH-MERMAID
  Scenario: graph-export emits Mermaid with stories and requirements
    When I run "cargo xtask graph-export --format mermaid"
    Then the command succeeds
    And the output contains "graph TD"
    And the output contains "US_TPL_PLT_001"
    And the output contains "REQ_TPL_PLATFORM_INTROSPECTION"
    And the output contains "-->|\"contains\"|"
