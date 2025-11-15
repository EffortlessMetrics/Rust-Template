Feature: Template Core Endpoints
  As a service operator
  I want standard operational endpoints
  So that I can monitor and manage the service

  @AC-TPL-001 @smoke
  Scenario: Health endpoint reports service is healthy
    When I GET /health
    Then I receive 200 with status "ok"

  @AC-TPL-002
  Scenario: Version endpoint reports build information
    When I GET /version
    Then I receive 200 with JSON containing "version" and "gitSha"

  @AC-TPL-003
  Scenario: Error responses include standardized error envelope
    When I POST /refunds with invalid data { "orderId": "", "amountCents": 0 }
    Then I receive a 4xx response
    And the response body contains "error" field
    And the response body contains "message" field
    And the response body contains "requestId" field

  @skip @AC-TPL-003
  # TODO(v1.2): Re-enable after improving test isolation for service availability flag
  # Currently disabled due to state bleeding between scenarios
  Scenario: Internal error responses include standardized error envelope
    Given the refund processing service is unavailable
    When I POST /refunds with { "orderId": "ORD-1", "amountCents": 5000 }
    Then I receive a 500 response
    And the response body contains "error" field
    And the response body contains "message" field
    And the response body contains "requestId" field

  @AC-TPL-004
  Scenario: Error responses include X-Request-ID header
    When I POST /refunds with invalid data { "orderId": "", "amountCents": 0 }
    Then I receive a 4xx response
    And the response includes "X-Request-ID" header
    And the "requestId" field in response body matches the "X-Request-ID" header

  @AC-TPL-004
  Scenario: X-Request-ID is propagated when provided in request
    Given I set "X-Request-ID" header to "test-request-123"
    When I POST /refunds with invalid data { "orderId": "", "amountCents": 0 }
    Then I receive a 4xx response
    And the response includes "X-Request-ID" header with value "test-request-123"
    And the "requestId" field in response body equals "test-request-123"

  @AC-TPL-004
  Scenario: X-Request-ID is generated when not provided in request
    When I POST /refunds with invalid data { "orderId": "", "amountCents": 0 }
    Then I receive a 4xx response
    And the response includes "X-Request-ID" header
    And the "X-Request-ID" header is a valid UUID or request identifier
