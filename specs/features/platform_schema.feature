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
    And the response body should contain "schemas"
    And the response body should contain "endpoints"
    And the response body should contain "spec_ledger"
    And the response body should contain "tasks"
    And the response body should contain "questions"
    And the response body should contain "devex_flows"
    And the response body should contain "config"
    And the response body should contain "doc_index"

  @AC-TPL-PLATFORM-SCHEMA
  Scenario: Platform exposes OpenAPI contract at /platform/openapi
    When I send a GET request to "/platform/openapi"
    Then the response status code should be 200
    And the response includes "content-type" header with value "application/yaml"
    And the response body should match the file "specs/openapi/openapi.yaml"

  @AC-TPL-PLATFORM-SCHEMA
  Scenario: Platform exposes individual schemas by name
    When I send a GET request to "/platform/schema/tasks"
    Then the response status code should be 200
    And the response body should contain "json_schema"
    And the response body should contain "source_file"
    And the response body should contain "specs/tasks.yaml"
    When I send a GET request to "/platform/schema/nonexistent"
    Then the response status code should be 404

  @AC-TPL-METADATA-COMPLETE
  Scenario: Platform metadata is exposed via status and UI
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    # Validate structural metadata fields (values come from test fixtures, not production)
    And the response body should contain "service_id"
    And the response body should contain "template_version"
    And the response body should contain "display_name"
    And the response body should contain "description"
    When I send a GET request to "/ui"
    Then the response status code should be 200
    And the response body should contain "docs/runbooks/platform-kernel.md"
    And the response body should contain "docs/AGENT_GUIDE.md"
    And the response body should contain "docs/feature_status.md"

  @AC-TPL-FORKS-STATUS-SUMMARY
  Scenario: Status endpoint includes fork visibility
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    And the response body should contain "governance"
    And the response body should contain "forks"
