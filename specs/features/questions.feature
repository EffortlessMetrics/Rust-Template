@questions @platform
Feature: Question Artifacts for Ambiguity Handling
  As a developer or agent
  I want flows to emit structured question artifacts when encountering ambiguity
  So that work can continue without stalling while decisions are captured

  @AC-TPL-ARTIFACTS-HAVE-REFS
  Scenario: Question artifacts support refs field for traceability
    # Light BDD scenario validating that question-new accepts --refs
    # Full functionality is verified by unit tests in questions.rs::tests::artifacts_have_refs
    Given I am in the actual workspace
    When I run "cargo xtask question-new --category TEST --summary 'Ref test' --flow bundle --phase selection --description 'Test description' --refs REQ-TPL-HEALTH"
    Then the command should succeed
    And I clean up created question test artifacts

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
