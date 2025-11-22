# Template Version: v3.0.0
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Platform Web UI
  As a developer or platform operator
  I want a web-based UI to visualize governance and platform health
  So that I can quickly understand the system without parsing YAML or JSON

  Background:
    Given the platform is running with UI enabled

  @AC-TPL-PLATFORM-UI-DASHBOARD
  Scenario: Dashboard serves HTML with platform metrics
    When I GET "http://localhost:8080/"
    Then the response status should be 200
    And the response content-type should be "text/html"
    And the response body should contain "Rust-as-Spec Platform"
    And the response body should contain "Platform Health"

  @AC-TPL-PLATFORM-UI-GRAPH
  Scenario: Graph view renders Mermaid diagram
    When I GET "http://localhost:8080/ui/graph"
    Then the response status should be 200
    And the response content-type should be "text/html"
    And the response body should contain "Governance Graph"
    And the response body should contain "mermaid"

  @AC-TPL-PLATFORM-UI-FLOWS
  Scenario: Flows view displays DevEx flows and tasks
    When I GET "http://localhost:8080/ui/flows"
    Then the response status should be 200
    And the response content-type should be "text/html"
    And the response body should contain "DevEx Flows"
    And the response body should contain "Tasks"
