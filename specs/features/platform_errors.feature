# Template Version: v3.3.6
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-12-07

Feature: Platform Error Taxonomy and Surfacing
  As an operator or agent
  I want governance errors to be mapped consistently to HTTP responses
  So that I can handle errors predictably and monitor error health via /platform/status

  Background:
    Given the platform service is running on port 8080

  @AC-TPL-ERROR-MAPPING
  Scenario: Platform status exposes error summary with stats structure
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have field "errors"
    And the field "errors" should be of type "object"
    And the JSON response should have nested field "errors.has_recent_errors"
    And the JSON response should have nested field "errors.stats"
    And the JSON response should have nested field "errors.stats.total_errors"
    And the JSON response should have nested field "errors.stats.client_errors"
    And the JSON response should have nested field "errors.stats.server_errors"

  @AC-TPL-ERROR-MAPPING
  Scenario: Error responses include error code and request ID
    When I GET "http://localhost:8080/platform/questions/Q-NONEXISTENT-999"
    Then the response status should be 404
    And the JSON response should have field "error"
    And the JSON response should have field "requestId"
