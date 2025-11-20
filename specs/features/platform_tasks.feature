Feature: Platform Tasks Surfacing
  The template must surface tasks defined in specs/tasks.yaml via CLI and HTTP.

  Background:
    Given I am in a Rust-Template workspace
    And I am in a Nix devshell
    And the platform service is running on port 8080

  @AC-TPL-TASKS-CLI
  Scenario: tasks-list prints tasks grouped by status
    When I run "cargo xtask tasks-list"
    Then the command succeeds
    And the output contains "📋 Tasks"
    And the output contains "open"
    And the output contains "TASK-TPL-IMPLEMENT-AC-001"
    And the output contains "REQ-TPL-PLATFORM-INTROSPECTION"

  @AC-TPL-TASKS-HTTP
  Scenario: /platform/tasks returns tasks JSON
    When I GET "http://localhost:8080/platform/tasks?status=open&req=REQ-TPL-PLATFORM-INTROSPECTION"
    Then the response status should be 200
    And the JSON response should have field "tasks"
    And the "tasks" array should contain an object with "id" = "TASK-TPL-IMPLEMENT-AC-001"
    And that object should have field "requirement" = "REQ-TPL-PLATFORM-INTROSPECTION"
    And that object should have field "status" = "open"
