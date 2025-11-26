@friction @platform @governance
Feature: Friction Log Artifacts
  As a developer or agent
  I want to capture and track friction encountered during development
  So that process improvements can be identified and prioritized

  Background:
    Given the friction directory exists

  @AC-TPL-GOV-FRICTION
  Scenario: Create friction entry via CLI
    When I run "cargo xtask friction-new 'Confusing error message in config validation'"
    Then the command should succeed
    And a friction file should exist matching "friction/F-*.yaml"
    And the friction file should contain:
      | field   | value_pattern              |
      | id      | F-\\d{4}                   |
      | status  | open                       |
    And the friction file should have a timestamp

  @AC-TPL-GOV-FRICTION
  Scenario: List friction entries via CLI
    Given 3 friction files exist with statuses:
      | status   | count |
      | open     | 2     |
      | resolved | 1     |
    When I run "cargo xtask friction-list"
    Then the output should contain:
      | field         | value |
      | Total         | 3     |
      | Open          | 2     |
      | Resolved      | 1     |
    And the output should list all friction IDs

  @AC-TPL-GOV-FRICTION
  Scenario: List friction entries with JSON output
    Given 2 friction files exist
    When I run "cargo xtask friction-list --json"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should have a "friction" array with 2 items

  @AC-TPL-GOV-FRICTION
  Scenario: HTTP endpoint returns all friction entries
    Given 2 friction files exist with titles:
      | title                                    |
      | Confusing error message                  |
      | Slow test suite                          |
    When I query the /platform/friction endpoint
    Then the response should be valid JSON
    And the response should include an array with 2 items
    And each item should have required fields:
      | field        |
      | id           |
      | title        |
      | status       |
      | created_at   |

  @AC-TPL-GOV-FRICTION
  Scenario: HTTP endpoint returns specific friction entry by ID
    Given a friction file exists with id "F-0001"
    When I query the /platform/friction/F-0001 endpoint
    Then the response should be valid JSON
    And the response should contain:
      | field   | value  |
      | id      | F-0001 |
      | status  | open   |

  @AC-TPL-GOV-FRICTION
  Scenario: Friction entry follows schema conventions
    Given I have a valid friction artifact
    When the friction entry is saved to friction/F-0042.yaml
    Then the file should contain YAML with required fields:
      | field          |
      | id             |
      | title          |
      | description    |
      | created_by     |
      | created_at     |
      | status         |
      | severity       |
    And the id should match pattern "F-\\d{4}"

  @AC-TPL-GOV-FRICTION
  Scenario: Friction entries are visible in platform status
    Given 2 friction files exist with status "open"
    When I query the /platform/status endpoint
    Then the response should include:
      | field                         | value |
      | governance.friction.open      | 2     |
      | governance.friction.total     | 2     |

  @AC-TPL-GOV-FRICTION
  Scenario: Friction entries support severity levels
    When I run "cargo xtask friction-new 'Missing documentation' --severity low"
    Then the command should succeed
    And the friction file should contain:
      | field    | value |
      | severity | low   |
