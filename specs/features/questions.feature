@questions @platform
Feature: Question Artifacts for Ambiguity Handling
  As a developer or agent
  I want flows to emit structured question artifacts when encountering ambiguity
  So that work can continue without stalling while decisions are captured

  @AC-TPL-QUESTIONS-LOGGED @wip
  Scenario: Questions list command works
    # Note: Works from real workspace; test isolation needs questions directory
    Given a clean development environment
    When I run "cargo xtask questions-list"
    Then the command should succeed

  @AC-TPL-QUESTIONS-LOGGED @wip
  Scenario: Questions list supports JSON output
    # Note: Works from real workspace; test isolation needs questions directory
    Given a clean development environment
    When I run "cargo xtask questions-list --json"
    Then the command should succeed
    And the output should be valid JSON
