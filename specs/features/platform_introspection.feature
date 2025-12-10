# Template Version: v3.3.8
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

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Questions endpoint returns all questions
    When I GET "http://localhost:8080/platform/questions"
    Then the response status should be 200
    And the JSON response should have field "questions"
    And the field "questions" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Questions endpoint supports status filtering
    When I GET "http://localhost:8080/platform/questions?status=open"
    Then the response status should be 200
    And the JSON response should have field "questions"
    And the field "questions" should be of type "array"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Question by ID endpoint returns specific question
    When I GET "http://localhost:8080/platform/questions/Q-EXAMPLE-001"
    Then the response status should be 200
    And the JSON response should have field "id"
    And the field "id" should equal "Q-EXAMPLE-001"
    And the JSON response should have field "summary"
    And the JSON response should have field "context"
    And the field "context" should be of type "object"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Question by ID endpoint returns 404 for unknown question
    When I GET "http://localhost:8080/platform/questions/Q-NONEXISTENT-999"
    Then the response status should be 404
    And the JSON response should have field "error"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
  Scenario: Forks endpoint returns all registered forks
    When I GET "http://localhost:8080/platform/forks"
    Then the response status should be 200
    And the JSON response should have field "forks"
    And the field "forks" should be of type "array"
    And the JSON response should have field "total"
    And the field "total" should be of type "number"

  @AC-TPL-PLATFORM-GOVERNANCE-APIS
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

  # ============================================================================
  # CLI/HTTP Parity Scenarios (REQ-TPL-INTROSPECTION-PARITY)
  # ============================================================================

  @AC-TPL-STATUS-PARITY-CLI-HTTP
  Scenario: Platform status exposes same key fields as CLI
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have field "governance"
    And the JSON response should have nested field "governance.ledger.stories"
    And the JSON response should have nested field "governance.ledger.requirements"
    And the JSON response should have nested field "governance.ledger.acs"
    And the JSON response should have nested field "governance.tasks.total"

  @AC-TPL-STATUS-AC-COVERAGE
  Scenario: Platform status includes AC coverage metrics
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have nested field "governance.ac_coverage"
    And the JSON response should have nested field "governance.ac_coverage.total"
    And the JSON response should have nested field "governance.ac_coverage.passing"
    And the JSON response should have nested field "governance.ac_coverage.failing"
    And the JSON response should have nested field "governance.ac_coverage.unknown"

  @AC-TPL-STATUS-TASK-BREAKDOWN
  Scenario: Platform status includes task status breakdown
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have nested field "governance.tasks.total"
    And the JSON response should have nested field "governance.tasks.by_status"
    And the JSON response should have nested field "governance.tasks.by_status.todo"
    And the JSON response should have nested field "governance.tasks.by_status.in_progress"
    And the JSON response should have nested field "governance.tasks.by_status.done"
