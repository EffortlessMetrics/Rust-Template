# Template Version: v2.4.0
# Schema: spec_ledger.yaml v1.0

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
    And the JSON response should have field "edges"

  @AC-TPL-PLATFORM-DEVEX
  Scenario: DevEx flows endpoint returns flows
    When I GET "http://localhost:8080/platform/devex/flows"
    Then the response status should be 200
    And the JSON response should have field "commands"
    And the JSON response should have field "flows"

  @AC-TPL-PLATFORM-DOCS
  Scenario: Docs index endpoint returns document index
    When I GET "http://localhost:8080/platform/docs/index"
    Then the response status should be 200
    And the JSON response should have field "docs"
