@forks @platform @governance
Feature: Fork Metadata Registry
  As a template maintainer
  I want to track all forks of this template
  So that improvements can be shared and adoption patterns can be understood

  @AC-TPL-GOV-FORKS
  Scenario: List registered forks via CLI
    Given I am in the actual workspace
    When I run "cargo xtask fork-list"
    Then the command should succeed

  @AC-TPL-GOV-FORKS
  Scenario: List forks with JSON output
    Given I am in the actual workspace
    When I run "cargo xtask fork-list --json"
    Then the command should succeed
    And the output should be valid JSON
