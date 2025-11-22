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

  @AC-TPL-TASKS-CLI
  Scenario: List tasks filtered by status
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-001     | Implement API        | Todo        | REQ-TPL-001          |
      | TASK-002     | Write tests          | InProgress  | REQ-TPL-002          |
      | TASK-003     | Deploy service       | Done        | REQ-TPL-003          |
    When I run "cargo xtask tasks-list --status InProgress"
    Then the command should succeed
    And the output should contain "TASK-002"
    And the output should not contain "TASK-001"
    And the output should not contain "TASK-003"

  @AC-TPL-TASKS-CLI
  Scenario: List tasks filtered by requirement
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-001     | Implement API        | Todo        | REQ-TPL-001          |
      | TASK-002     | Write tests          | InProgress  | REQ-TPL-001          |
      | TASK-003     | Deploy service       | Done        | REQ-TPL-002          |
    When I run "cargo xtask tasks-list --requirement REQ-TPL-001"
    Then the command should succeed
    And the output should contain "TASK-001"
    And the output should contain "TASK-002"
    And the output should not contain "TASK-003"

  # CLI Task Creation
  @AC-TPL-TASKS-CREATE-CLI
  Scenario: Create a new task via CLI
    When I run "cargo xtask task-create TASK-NEW-001 --title 'New feature task' --requirement REQ-TPL-001 --status Todo"
    Then the command should succeed
    And "specs/tasks.yaml" should contain a task with id "TASK-NEW-001"
    And the task "TASK-NEW-001" should have title "New feature task"
    And the task "TASK-NEW-001" should have requirement "REQ-TPL-001"
    And the task "TASK-NEW-001" should have status "Todo"

  @AC-TPL-TASKS-CREATE-CLI
  Scenario: Create task fails with duplicate ID
    Given a task "TASK-DUP-001" exists in "specs/tasks.yaml"
    When I run "cargo xtask task-create TASK-DUP-001 --title 'Duplicate task' --requirement REQ-TPL-001"
    Then the command should fail
    And the output should contain "Task ID already exists"

  @AC-TPL-TASKS-CREATE-CLI
  Scenario: Create task with ACs
    When I run "cargo xtask task-create TASK-AC-001 --title 'Task with ACs' --requirement REQ-TPL-001 --acs AC-TPL-001,AC-TPL-002"
    Then the command should succeed
    And the task "TASK-AC-001" should have ACs "AC-TPL-001" and "AC-TPL-002"

  # CLI Task Updates
  @AC-TPL-TASKS-UPDATE-CLI
  Scenario: Update task status via CLI
    Given a task "TASK-UPD-001" exists with status "Todo"
    When I run "cargo xtask task-update TASK-UPD-001 --status InProgress"
    Then the command should succeed
    And the task "TASK-UPD-001" should have status "InProgress"

  @AC-TPL-TASKS-UPDATE-CLI
  Scenario: Update task title via CLI
    Given a task "TASK-UPD-002" exists with title "Old title"
    When I run "cargo xtask task-update TASK-UPD-002 --title 'New title'"
    Then the command should succeed
    And the task "TASK-UPD-002" should have title "New title"

  @AC-TPL-TASKS-UPDATE-CLI
  Scenario: Update task owner via CLI
    Given a task "TASK-UPD-003" exists with owner "team-platform"
    When I run "cargo xtask task-update TASK-UPD-003 --owner agent"
    Then the command should succeed
    And the task "TASK-UPD-003" should have owner "agent"

  # Governance Validation
  @AC-TPL-TASK-TRANSITIONS
  Scenario: Task status transition validation - valid transitions
    Given a task "TASK-TRANS-001" exists with status "Todo"
    When I run "cargo xtask task-update TASK-TRANS-001 --status InProgress"
    Then the command should succeed
    And the task "TASK-TRANS-001" should have status "InProgress"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Task status transition validation - invalid backwards transition
    Given a task "TASK-TRANS-002" exists with status "Done"
    When I run "cargo xtask task-update TASK-TRANS-002 --status Todo"
    Then the command should fail
    And the output should contain "Invalid status transition"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Validate task requirement exists in spec ledger
    When I run "cargo xtask task-create TASK-VAL-001 --title 'Invalid task' --requirement REQ-NONEXISTENT"
    Then the command should fail
    And the output should contain "Requirement REQ-NONEXISTENT not found in spec_ledger.yaml"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Validate task ACs exist in spec ledger
    When I run "cargo xtask task-create TASK-VAL-002 --title 'Invalid ACs' --requirement REQ-TPL-001 --acs AC-NONEXISTENT"
    Then the command should fail
    And the output should contain "AC AC-NONEXISTENT not found in spec_ledger.yaml"

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

  # HTTP API - Task Creation (POST /platform/tasks)
  @AC-TPL-TASKS-HTTP
  Scenario: Create a new task via HTTP API
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-HTTP-001",
        "title": "New API task",
        "requirement": "REQ-TPL-HEALTH",
        "status": "Todo"
      }
      """
    Then the response status code should be 201
    And the response body should be valid JSON
    And the JSON should have field "id" with value "TASK-HTTP-001"
    And the JSON should have field "title" with value "New API task"
    And the JSON should have field "status" with value "Todo"

  @AC-TPL-TASKS-HTTP
  Scenario: Create task with ACs via HTTP API
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-HTTP-002",
        "title": "Task with ACs",
        "requirement": "REQ-TPL-VERSION",
        "acs": ["AC-TPL-001", "AC-TPL-002"],
        "status": "Todo"
      }
      """
    Then the response status code should be 201
    And the JSON array field "acs" should contain "AC-TPL-001"
    And the JSON array field "acs" should contain "AC-TPL-002"

  @AC-TPL-TASKS-HTTP
  Scenario: Create task fails with duplicate ID via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id           | title                | status      | requirement           |
      | TASK-DUP-HTTP | Existing task       | Todo        | REQ-TPL-HEALTH       |
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-DUP-HTTP",
        "title": "Duplicate task",
        "requirement": "REQ-TPL-HEALTH",
        "status": "Todo"
      }
      """
    Then the response status code should be 409
    And the response body should contain "already exists"

  @AC-TPL-TASKS-HTTP
  Scenario: Create task fails without required fields
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-NO-TITLE"
      }
      """
    Then the response status code should be 400
    And the response body should contain "required"

  # HTTP API - Task Updates (PUT /platform/tasks/:id)
  @AC-TPL-TASKS-HTTP
  Scenario: Update task title via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id              | title           | status      | requirement           |
      | TASK-UPDATE-001 | Original title  | Todo        | REQ-TPL-HEALTH       |
    When I send a PUT request to "/platform/tasks/TASK-UPDATE-001" with body:
      """
      {
        "title": "Updated title"
      }
      """
    Then the response status code should be 200
    And the JSON should have field "title" with value "Updated title"
    And the task "TASK-UPDATE-001" should have title "Updated title"

  @AC-TPL-TASKS-HTTP
  Scenario: Update task owner via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id              | title     | status      | requirement      | owner         |
      | TASK-UPDATE-002 | Some task | InProgress  | REQ-TPL-VERSION  | team-platform |
    When I send a PUT request to "/platform/tasks/TASK-UPDATE-002" with body:
      """
      {
        "owner": "agent"
      }
      """
    Then the response status code should be 200
    And the JSON should have field "owner" with value "agent"

  @AC-TPL-TASKS-HTTP
  Scenario: Update task with multiple fields via HTTP API
    Given the following tasks exist in "specs/tasks.yaml":
      | id              | title      | status | requirement      |
      | TASK-UPDATE-003 | Old task   | Todo   | REQ-TPL-HEALTH   |
    When I send a PUT request to "/platform/tasks/TASK-UPDATE-003" with body:
      """
      {
        "title": "New task name",
        "owner": "agent",
        "labels": ["priority-high", "backend"]
      }
      """
    Then the response status code should be 200
    And the JSON should have field "title" with value "New task name"
    And the JSON should have field "owner" with value "agent"
    And the JSON array field "labels" should contain "priority-high"

  @AC-TPL-TASKS-HTTP
  Scenario: Update non-existent task returns 404
    When I send a PUT request to "/platform/tasks/TASK-NONEXISTENT" with body:
      """
      {
        "title": "This should fail"
      }
      """
    Then the response status code should be 404
    And the response body should contain "not found"

  # Validation Rules via HTTP API
  @AC-TPL-TASK-TRANSITIONS
  Scenario: Create task with invalid requirement fails validation
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-INVALID-REQ",
        "title": "Task with bad requirement",
        "requirement": "REQ-NONEXISTENT",
        "status": "Todo"
      }
      """
    Then the response status code should be 400
    And the response body should contain "Requirement REQ-NONEXISTENT not found"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Create task with invalid ACs fails validation
    When I send a POST request to "/platform/tasks" with body:
      """
      {
        "id": "TASK-INVALID-AC",
        "title": "Task with bad ACs",
        "requirement": "REQ-TPL-HEALTH",
        "acs": ["AC-NONEXISTENT"],
        "status": "Todo"
      }
      """
    Then the response status code should be 400
    And the response body should contain "AC AC-NONEXISTENT not found"

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Update task with invalid status transition via PUT
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title     | status | requirement     |
      | TASK-BAD-TRANS-01 | Done task | Done   | REQ-TPL-HEALTH  |
    When I send a PUT request to "/platform/tasks/TASK-BAD-TRANS-01" with body:
      """
      {
        "status": "Todo"
      }
      """
    Then the response status code should be 400
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
