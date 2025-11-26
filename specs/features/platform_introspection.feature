# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Platform Introspection API
  As an operator or agent
  I want the service to expose its governance state
  So I can understand and reason about it at runtime

  Background:
    Given the platform service is running on port 8080

  @AC-TPL-PLATFORM-GRAPH
  Scenario: Graph endpoint returns governance graph
    When I GET "http://localhost:8080/platform/graph"
    Then the response status should be 200
    And the JSON response should have field "nodes"
    And the field "nodes" should be of type "array"
    And the JSON response should have field "edges"
    And the field "edges" should be of type "array"

  @AC-TPL-PLATFORM-DEVEX
  Scenario: DevEx flows endpoint returns flows
    When I GET "http://localhost:8080/platform/devex/flows"
    Then the response status should be 200
    And the JSON response should have field "commands"
    And the field "commands" should be of type "object"
    And the field "commands" should not be empty
    And the JSON response should have field "flows"
    And the field "flows" should be of type "object"

  @AC-TPL-PLATFORM-DOCS
  Scenario: Docs index endpoint returns document index
    When I GET "http://localhost:8080/platform/docs/index"
    Then the response status should be 200
    And the JSON response should have field "docs"
    And the field "docs" should be of type "array"

  @AC-TPL-PLATFORM-QUESTIONS
  Scenario: Questions endpoint returns all questions
    When I GET "http://localhost:8080/platform/questions"
    Then the response status should be 200
    And the JSON response should have field "questions"
    And the field "questions" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"

  @AC-TPL-PLATFORM-QUESTIONS
  Scenario: Questions endpoint supports status filtering
    When I GET "http://localhost:8080/platform/questions?status=open"
    Then the response status should be 200
    And the JSON response should have field "questions"
    And the field "questions" should be of type "array"

  @AC-TPL-PLATFORM-QUESTIONS
  Scenario: Question by ID endpoint returns specific question
    When I GET "http://localhost:8080/platform/questions/Q-EXAMPLE-001"
    Then the response status should be 200
    And the JSON response should have field "id"
    And the field "id" should equal "Q-EXAMPLE-001"
    And the JSON response should have field "summary"
    And the JSON response should have field "context"
    And the field "context" should be of type "object"

  @AC-TPL-PLATFORM-QUESTIONS
  Scenario: Question by ID endpoint returns 404 for unknown question
    When I GET "http://localhost:8080/platform/questions/Q-NONEXISTENT-999"
    Then the response status should be 404
    And the JSON response should have field "error"

  @AC-TPL-PLATFORM-FORKS
  Scenario: Forks endpoint returns all registered forks
    When I GET "http://localhost:8080/platform/forks"
    Then the response status should be 200
    And the JSON response should have field "forks"
    And the field "forks" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"

  @AC-TPL-PLATFORM-FORKS
  Scenario: Fork by name endpoint returns 404 for unknown fork
    When I GET "http://localhost:8080/platform/forks/FORK-NONEXISTENT-999"
    Then the response status should be 404
    And the JSON response should have field "error"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Questions endpoint returns valid JSON for AI/IDP consumption
    When I GET "http://localhost:8080/platform/questions"
    Then the response status should be 200
    And the JSON response should have field "questions"
    And the field "questions" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Friction endpoint returns valid JSON for AI/IDP consumption
    When I GET "http://localhost:8080/platform/friction"
    Then the response status should be 200
    And the JSON response should have field "entries"
    And the field "entries" should be of type "array"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Forks endpoint returns valid JSON for AI/IDP consumption
    When I GET "http://localhost:8080/platform/forks"
    Then the response status should be 200
    And the JSON response should have field "forks"
    And the field "forks" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"
