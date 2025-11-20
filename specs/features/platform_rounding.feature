Feature: Platform Rounding Features
  As a platform user or agent
  I want to introspect the platform's state and get guidance on next steps
  So that I can work efficiently and ensure governance compliance

  Background:
    Given the platform is running
    And I have a valid workspace

  @AC-TPL-SUGGEST-NEXT-CLI
  Scenario: suggest-next prints structured steps for a task
    When I run "cargo xtask suggest-next --task implement_ac"
    Then the command succeeds
    And the output contains "Suggested next steps for task"
    And the output contains "cargo xtask ac-new"
    And the output contains "cargo xtask bundle"

  @AC-TPL-SUGGEST-NEXT-HTTP
  Scenario: suggest-next HTTP endpoint returns JSON
    When I GET "http://localhost:8080/platform/tasks/suggest-next?task=implement_ac"
    Then the response status should be 200
    And the JSON response should have field "task"
    And the JSON response should have field "recommended_sequence"

  @AC-TPL-PLATFORM-STATUS
  Scenario: Platform status returns governance summary
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have field "template_version"
    And the JSON response should have field "service_id"
    And the JSON response should have field "governance"
    And the JSON response should have field "governance.ledger.requirements"

  @AC-TPL-POLICY-STATUS-OVERVIEW
  Scenario: Platform status exposes policy health
    When I GET "http://localhost:8080/platform/status"
    Then the response status should be 200
    And the JSON response should have field "governance.policies.status"
