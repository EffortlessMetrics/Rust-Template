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
