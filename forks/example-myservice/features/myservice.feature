# Template Version: v3.3.5
# Schema: spec_ledger.yaml v1.0 (extended with spec_additions.yaml)
# Service: MyService
# Last Updated: 2025-12-01

Feature: MyService Echo Endpoint
  As a developer using MyService
  I want to send messages to the echo endpoint
  So that I can verify the service is working and test request/response patterns

  Background:
    Given the MyService application is running
    And the service is healthy

  @AC-MYSERV-001 @smoke
  Scenario: Echo endpoint returns the provided message
    When I GET "/api/echo?message=hello"
    Then the response status is 200
    And the response body contains "hello"
    And the response header "X-Request-ID" is present

  @AC-MYSERV-001
  Scenario: Echo endpoint handles multi-word messages
    When I GET "/api/echo?message=hello%20world"
    Then the response status is 200
    And the response body contains "hello world"
    And the response header "X-Request-ID" is present

  @AC-MYSERV-001
  Scenario: Echo endpoint handles special characters
    When I GET "/api/echo?message=test%21%40%23"
    Then the response status is 200
    And the response body contains "test!@#"

  @AC-MYSERV-002
  Scenario: Echo endpoint rejects missing message parameter
    When I GET "/api/echo"
    Then the response status is 400
    And the response body contains "error"
    And the response body contains "message"
    And the response body contains "requestId"
    And the response header "X-Request-ID" is present

  @AC-MYSERV-002
  Scenario: Echo endpoint rejects empty message parameter
    When I GET "/api/echo?message="
    Then the response status is 400
    And the response body contains "error"
    And the response body contains "message parameter is required"

  @AC-MYSERV-002
  Scenario: Echo endpoint rejects whitespace-only message
    When I GET "/api/echo?message=%20%20%20"
    Then the response status is 400
    And the response body contains "error"
    And the response body contains "message parameter is required"

  # Optional: Additional scenarios for edge cases
  @AC-MYSERV-001 @edge-case
  Scenario: Echo endpoint handles maximum length message
    Given a message with 1000 characters
    When I GET "/api/echo" with that message
    Then the response status is 200
    And the response body contains the full message

  @AC-MYSERV-002 @edge-case
  Scenario: Echo endpoint rejects excessively long message
    Given a message with 10000 characters
    When I GET "/api/echo" with that message
    Then the response status is 400
    And the response body contains "error"
    And the response body contains "message too long"
