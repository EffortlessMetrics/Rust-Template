Feature: Platform schema and metadata surfaces

  Background:
    Given the platform is running

  @AC-TPL-PLATFORM-SCHEMA
  Scenario: Platform exposes machine-readable schema
    When I send a GET request to "/platform/schema"
    Then the response status code should be 200
    And the response body should contain "/platform/status"
    And the response body should contain "/platform/tasks"
    And the response body should contain "/platform/agent/hints"

  @AC-TPL-METADATA-COMPLETE
  Scenario: Platform metadata is exposed via status and UI
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    And the response body should contain "rust-template"
    And the response body should contain "3.3.0"
    When I send a GET request to "/ui"
    Then the response status code should be 200
    And the response body should contain "docs/runbooks/platform-kernel.md"
    And the response body should contain "docs/AGENT_GUIDE.md"
    And the response body should contain "docs/feature_status.md"
