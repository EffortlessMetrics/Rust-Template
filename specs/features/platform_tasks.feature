# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Platform Tasks Management
  As a developer or agent
  I want to manage tasks via CLI and API
  So that I can track work items and enforce governance rules

  Background:
    Given the platform is running

  # CLI Task Listing
  @AC-TPL-TASKS-CLI
  Scenario: List all tasks via CLI
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-001     | Implement API        | Todo        | REQ-TPL-001          |
      | TASK-002     | Write tests          | InProgress  | REQ-TPL-002          |
      | TASK-003     | Deploy service       | Done        | REQ-TPL-003          |
    When I run "cargo xtask tasks-list"
    Then the command should succeed
    And the output should contain "TASK-001"
    And the output should contain "TASK-002"
    And the output should contain "TASK-003"
    And the output should contain "Implement API"

  @AC-TPL-TASKS-CREATE-CLI
  Scenario: Create a new task via CLI
    Given the following tasks exist in "specs/tasks.yaml":
      | id            | title              | status | requirement                  |
      | TASK-SEED-001 | Seed existing task | Todo   | REQ-TPL-PLATFORM-INTROSPECTION |
    When I run "cargo xtask task-create --id TASK-NEW-001 --title 'New platform task' --req REQ-TPL-PLATFORM-INTROSPECTION"
    Then the command should succeed
    And task "TASK-NEW-001" should exist with title "New platform task" and requirement "REQ-TPL-PLATFORM-INTROSPECTION"
    And task "TASK-NEW-001" should have status "Todo" in tasks.yaml

  @AC-TPL-TASKS-UPDATE-CLI
  Scenario: Update a task via CLI
    Given the following tasks exist in "specs/tasks.yaml":
      | id            | title               | status | requirement     |
      | TASK-UPD-001  | Update candidate    | Todo   | REQ-TPL-HEALTH |
    When I run "cargo xtask task-update --id TASK-UPD-001 --status InProgress --owner agent --title 'Updated platform task'"
    Then the command should succeed
    And task "TASK-UPD-001" should have status "InProgress" in tasks.yaml
    And task "TASK-UPD-001" should have owner "agent"
    And task "TASK-UPD-001" should have title "Updated platform task"

  # HTTP API Tests
  @AC-TPL-TASKS-HTTP
  Scenario: Get tasks via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-API-001 | Implement API        | Todo        | REQ-TPL-001          |
      | TASK-API-002 | Write tests          | InProgress  | REQ-TPL-002          |
    When I send a GET request to "/platform/tasks"
    Then the response status code should be 200
    And the response body should be valid JSON
    And the JSON should contain a task with id "TASK-API-001"
    And the JSON should contain a task with id "TASK-API-002"

  @AC-TPL-TASKS-HTTP
  Scenario: Get tasks filtered by status via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-API-003 | Implement API        | Todo        | REQ-TPL-001          |
      | TASK-API-004 | Write tests          | InProgress  | REQ-TPL-002          |
    When I send a GET request to "/platform/tasks?status=InProgress"
    Then the response status code should be 200
    And the JSON should contain a task with id "TASK-API-004"
    And the JSON should not contain a task with id "TASK-API-003"

  @AC-TPL-TASKS-HTTP
  Scenario: Task statuses are normalized to the canonical set
    Given the following tasks exist in "specs/tasks.yaml":
      | id              | title                     | status        | requirement      |
      | TASK-NORM-001   | Backlog item              | open          | REQ-TPL-HEALTH  |
      | TASK-NORM-002   | Work in progress (snake)  | in_progress   | REQ-TPL-HEALTH  |
      | TASK-NORM-003   | Work in progress (dash)   | in-progress   | REQ-TPL-HEALTH  |
      | TASK-NORM-004   | Closed alias              | closed        | REQ-TPL-HEALTH  |
    When I send a GET request to "/platform/tasks"
    Then the response status code should be 200
    And the response body should contain '"status":"Todo"'
    And the response body should contain '"status":"InProgress"'
    And the response body should contain '"status":"Done"'

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Update task status via HTTP API
    Given a task "TASK-001" exists with status "Todo"
    When I send a POST request to "/platform/tasks/TASK-001/status" with body:
      """
      {
        "status": "InProgress"
      }
      """
    Then the response status code should be 204
    And the task "TASK-001" should have status "InProgress"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Invalid task transition via HTTP API
    Given a task "TASK-002" exists with status "Done"
    When I send a POST request to "/platform/tasks/TASK-002/status" with body:
      """
      {
        "status": "Todo"
      }
      """
    Then the response status code should be 500
    And the response body should contain "Invalid status transition"

  # GET /platform/tasks with advanced filtering
  @AC-TPL-TASKS-HTTP
  Scenario: Get tasks filtered by requirement via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-REQ-001 | Health task          | Todo        | REQ-TPL-HEALTH       |
      | TASK-REQ-002 | Version task         | InProgress  | REQ-TPL-VERSION      |
      | TASK-REQ-003 | Another health task  | Done        | REQ-TPL-HEALTH       |
    When I send a GET request to "/platform/tasks?req=REQ-TPL-HEALTH"
    Then the response status code should be 200
    And the JSON should contain a task with id "TASK-REQ-001"
    And the JSON should contain a task with id "TASK-REQ-003"
    And the JSON should not contain a task with id "TASK-REQ-002"

  @AC-TPL-TASKS-HTTP
  Scenario: Get empty task list when no tasks match filters
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title          | status      | requirement           |
      | TASK-FIL-001 | Some task      | Todo        | REQ-TPL-HEALTH       |
    When I send a GET request to "/platform/tasks?status=Done"
    Then the response status code should be 200
    And the JSON should have an empty tasks array
