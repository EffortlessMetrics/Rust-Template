@forks @platform @governance
Feature: Fork Metadata Registry
  As a template maintainer
  I want to track all forks of this template
  So that improvements can be shared and adoption patterns can be understood

  Background:
    Given the forks directory exists

  @AC-TPL-GOV-FORKS
  Scenario: Register a new fork via CLI
    When I run "cargo xtask fork-register my-service https://github.com/org/my-service"
    Then the command should succeed
    And the forks/fork_registry.yaml file should exist
    And the registry should contain a fork named "my-service"
    And the fork entry should have:
      | field         | value_pattern                            |
      | name          | my-service                               |
      | repo_url      | https://github.com/org/my-service        |
      | registered_at | \\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2} |

  @AC-TPL-GOV-FORKS
  Scenario: Reject duplicate fork registration
    Given a fork named "existing-service" is registered
    When I run "cargo xtask fork-register existing-service https://github.com/org/another"
    Then the command should fail
    And the output should contain "already registered"

  @AC-TPL-GOV-FORKS
  Scenario: List all registered forks via CLI
    Given 3 forks are registered:
      | name              | repo_url                                |
      | service-alpha     | https://github.com/org/service-alpha    |
      | service-beta      | https://github.com/org/service-beta     |
      | service-gamma     | https://github.com/org/service-gamma    |
    When I run "cargo xtask fork-list"
    Then the output should contain:
      | field       | value |
      | Total forks | 3     |
    And the output should list all fork names

  @AC-TPL-GOV-FORKS
  Scenario: List forks with JSON output
    Given 2 forks are registered
    When I run "cargo xtask fork-list --json"
    Then the command should succeed
    And the output should be valid JSON
    And the JSON should have a "forks" array with 2 items

  @AC-TPL-GOV-FORKS
  Scenario: HTTP endpoint returns all forks
    Given 2 forks are registered:
      | name          | repo_url                            |
      | service-one   | https://github.com/org/service-one  |
      | service-two   | https://github.com/org/service-two  |
    When I query the /platform/forks endpoint
    Then the response should be valid JSON
    And the response should include an array with 2 items
    And each item should have required fields:
      | field         |
      | name          |
      | repo_url      |
      | registered_at |

  @AC-TPL-GOV-FORKS
  Scenario: HTTP endpoint returns specific fork by name
    Given a fork named "my-fork" is registered with URL "https://github.com/org/my-fork"
    When I query the /platform/forks/my-fork endpoint
    Then the response should be valid JSON
    And the response should contain:
      | field    | value                           |
      | name     | my-fork                         |
      | repo_url | https://github.com/org/my-fork  |

  @AC-TPL-GOV-FORKS
  Scenario: Fork registry follows schema conventions
    Given I have a valid fork registry
    When the registry is saved to forks/fork_registry.yaml
    Then the file should contain YAML with required structure:
      | field            |
      | schema_version   |
      | forks            |
    And each fork should have required fields:
      | field         |
      | name          |
      | repo_url      |
      | registered_at |

  @AC-TPL-GOV-FORKS
  Scenario: Fork metadata is visible in platform status
    Given 5 forks are registered
    When I query the /platform/status endpoint
    Then the response should include:
      | field                    | value |
      | governance.forks.count   | 5     |

  @AC-TPL-GOV-FORKS
  Scenario: Fork registration supports optional metadata
    When I run "cargo xtask fork-register my-service https://github.com/org/my-service --description 'My microservice' --maintainer 'team@example.com'"
    Then the command should succeed
    And the fork entry should have:
      | field       | value            |
      | description | My microservice  |
      | maintainer  | team@example.com |

  @AC-TPL-GOV-FORKS
  Scenario: Fork registry supports template version tracking
    Given a fork named "versioned-service" is registered
    When the fork metadata includes template_version "3.3.3"
    Then the registry should track which template version was used
    And the /platform/forks/versioned-service endpoint should include:
      | field             | value |
      | template_version  | 3.3.3 |
