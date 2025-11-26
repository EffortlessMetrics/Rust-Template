@forks @platform @governance
Feature: Fork Metadata Registry
  As a template maintainer
  I want to track all forks of this template
  So that improvements can be shared and adoption patterns can be understood

  @AC-TPL-GOV-FORKS @wip
  Scenario: List registered forks via CLI
    # Note: Works from real workspace; test isolation needs fork_registry.yaml
    Given a clean development environment
    When I run "cargo xtask fork-list"
    Then the command should succeed

  @AC-TPL-GOV-FORKS @wip
  Scenario: List forks with JSON output
    # Note: Works from real workspace; test isolation needs fork_registry.yaml
    Given a clean development environment
    When I run "cargo xtask fork-list --json"
    Then the command should succeed
    And the output should be valid JSON
