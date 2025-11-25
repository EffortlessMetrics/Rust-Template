# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-25

Feature: Platform security and log hygiene
  As a platform operator
  I want platform endpoints to require authentication in hardened mode
  And never leak secrets in status responses

  Background:
    Given the platform is running

  @AC-TPL-PLATFORM-AUTH-BASIC
  Scenario: Write endpoints require a platform token when auth mode is basic
    Given platform auth mode is "basic" with token "test-token"
    And a task "TASK-AUTH-001" exists with status "Todo"
    When I send a POST request to "/platform/tasks/TASK-AUTH-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 401
    When I set "X-Platform-Token" header to "test-token"
    And I send a POST request to "/platform/tasks/TASK-AUTH-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 204

  @AC-TPL-LOG-NO-SECRETS
  Scenario: Platform status redacts secrets
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    And the response body should not contain "dev-secret-key"
    And the response body should not contain "dev-platform-token"
