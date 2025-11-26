@questions @platform
Feature: Question Artifacts for Ambiguity Handling
  As a developer or agent
  I want flows to emit structured question artifacts when encountering ambiguity
  So that work can continue without stalling while decisions are captured

  Background:
    Given the questions directory exists

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Bundle flow emits question when task not found
    When I run "cargo xtask bundle nonexistent_task"
    Then the command should fail
    And a question file should exist matching "questions/Q-BUNDLE-*.yaml"
    And the question file should contain:
      | field   | value                           |
      | flow    | bundle                          |
      | phase   | task_lookup                     |
      | status  | open                            |
    And the question should have at least 2 options

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Questions are visible in cargo xtask status
    Given a question file exists with status "open"
    When I run "cargo xtask status"
    Then the output should contain "Questions:"
    And the output should contain "Open:        1"

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Questions are visible in /platform/status
    Given a question file exists with status "open"
    When I query the /platform/status endpoint
    Then the response should include:
      | field                        | value |
      | governance.questions.open    | 1     |
      | governance.questions.total   | 1     |

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Question file follows schema conventions
    Given I have a valid question artifact
    When the question is saved to questions/Q-TEST-001.yaml
    Then the file should contain YAML with required fields:
      | field          |
      | id             |
      | summary        |
      | context.flow   |
      | context.phase  |
      | created_by     |
      | created_at     |
      | status         |
    And the id should match pattern "Q-[A-Z0-9]+-\\d{3}"

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Multiple questions are counted correctly
    Given 3 question files exist with statuses:
      | status   | count |
      | open     | 2     |
      | resolved | 1     |
    When I run "cargo xtask status"
    Then the output should contain "Open:        2"
    And the output should contain "Resolved:    1"
    And the output should contain "Total:       3"

  @AC-TPL-QUESTIONS-LOGGED
  Scenario: Question includes recommendation and options
    When a flow encounters ambiguity
    And emits a question with 3 options
    And includes a recommendation with rationale
    Then the question file should contain an "options" array
    And the question file should contain a "recommendation" object
    And the recommendation should reference one of the options
