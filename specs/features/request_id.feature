# Template Version: v3.3.15

Feature: HTTP request identifier propagation
  As an operator
  I want request identifiers to be preserved or generated
  So that requests are always traceable

  Background:
    Given the platform is running

  @AC-TPL-004
  Scenario: Existing request identifier is preserved
    When I set "x-request-id" header to "bdd-request-id-001"
    And I send a GET request to "/health"
    Then the response status code should be 200
    And the response includes "x-request-id" header with value "bdd-request-id-001"

  @AC-TPL-004
  Scenario: Request identifier is generated when missing
    When I send a GET request to "/health"
    Then the response status code should be 200
    And the response includes "x-request-id" header
    And the "x-request-id" header is a valid UUID or request identifier
