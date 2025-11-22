# Template Version: v3.0.0
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
