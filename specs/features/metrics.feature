Feature: Metrics endpoint
  As an operator
  I want the service to expose HTTP metrics
  So that I can monitor traffic and health

  @AC-TPL-007
  Scenario: Metrics endpoint is available
    When I GET /health
    And I GET /metrics
    Then I receive a 200 response
    And the response body contains "http_requests_total"
