# Template Version: v3.3.8
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

  @AC-TPL-PLATFORM-AUTH-BASIC
  Scenario: Basic auth without a token surfaces a warning state
    Given platform auth mode is "basic" without a token
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    And the response body should contain '"mode":"basic"'
    And the response body should contain '"token_present":false'

  @AC-TPL-PLATFORM-AUTH-BASIC
  Scenario: Write endpoints require a valid JWT bearer token when auth mode is jwt
    Given platform auth mode is "jwt" with secret "jwt-test-secret"
    And a task "TASK-AUTH-JWT-001" exists with status "Todo"
    When I send a POST request to "/platform/tasks/TASK-AUTH-JWT-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 401
    When I set Authorization bearer token signed with secret "jwt-test-secret"
    And I send a POST request to "/platform/tasks/TASK-AUTH-JWT-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 204

  @AC-TPL-PLATFORM-AUTH-BASIC
  Scenario: Authorization bearer token takes precedence over legacy platform token
    Given platform auth mode is "basic" with token "test-token"
    And a task "TASK-AUTH-PRECEDENCE-001" exists with status "Todo"
    When I set "X-Platform-Token" header to "test-token"
    And I set "Authorization" header to "Bearer invalid.jwt.token"
    And I send a POST request to "/platform/tasks/TASK-AUTH-PRECEDENCE-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 401
    And the task "TASK-AUTH-PRECEDENCE-001" should have status "Todo"

  @AC-TPL-PLATFORM-AUTH-BASIC
  Scenario: Authorization bearer scheme is case-insensitive in basic mode
    Given platform auth mode is "basic" with token "test-token"
    And a task "TASK-AUTH-CASE-001" exists with status "Todo"
    When I set "Authorization" header to "bEaReR test-token"
    And I send a POST request to "/platform/tasks/TASK-AUTH-CASE-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 204
    And the task "TASK-AUTH-CASE-001" should have status "InProgress"

  @AC-TPL-LOG-NO-SECRETS
  Scenario: Platform status redacts secrets
    When I send a GET request to "/platform/status"
    Then the response status code should be 200
    And the response body should not contain "dev-secret-key"
    And the response body should not contain "dev-platform-token"
