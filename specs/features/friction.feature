@friction @platform @governance
Feature: Friction Log Artifacts
  As a developer or agent
  I want to capture and track friction encountered during development
  So that process improvements can be identified and prioritized

  @AC-TPL-GOV-FRICTION
  Scenario: List friction entries via CLI
    Given I am in the actual workspace
    When I run "cargo xtask friction-list"
    Then the command should succeed

  @AC-TPL-GOV-FRICTION
  Scenario: List friction entries with JSON output
    Given I am in the actual workspace
    When I run "cargo xtask friction-list --json"
    Then the command should succeed
    And the output should be valid JSON
