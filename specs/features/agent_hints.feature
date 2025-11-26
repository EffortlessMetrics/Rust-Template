# Template Version: v3.3.1
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-11-22

Feature: Agent Hints API
  As an AI agent
  I want context-aware hints about next steps
  So that I can work efficiently within the governance framework

  Background:
    Given the platform is running

  @AC-TPL-AGENT-HINTS
  Scenario: GET /platform/agent/hints returns context-aware hints
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-001     | Implement authentication  | Todo        | REQ-TPL-001          |
      | TASK-HINT-002     | Write unit tests          | InProgress  | REQ-TPL-002          |
      | TASK-HINT-003     | Deploy to production      | Done        | REQ-TPL-003          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the response body should be valid JSON
    And the JSON response should have field "hints"

  @AC-TPL-AGENT-HINTS
  Scenario: Hints include next steps based on current state
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-010     | Fix authentication bug    | Todo        | REQ-TPL-001          |
      | TASK-HINT-011     | Add integration tests     | InProgress  | REQ-TPL-002          |
      | TASK-HINT-012     | Update documentation      | Done        | REQ-TPL-003          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the "hints" array should contain task "TASK-HINT-010"
    And the "hints" array should contain task "TASK-HINT-011"
    And the "hints" array should not contain task "TASK-HINT-012"

  @AC-TPL-AGENT-HINTS
  Scenario: Hints filter tasks by Todo and InProgress status
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-020     | Refactor auth module      | Todo        | REQ-TPL-001          |
      | TASK-HINT-021     | Review PR #123            | Review      | REQ-TPL-002          |
      | TASK-HINT-022     | Implement API endpoint    | InProgress  | REQ-TPL-001          |
      | TASK-HINT-023     | Release v1.2.0            | Done        | REQ-TPL-003          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the "hints" array should contain task "TASK-HINT-020"
    And the "hints" array should contain task "TASK-HINT-022"
    And the "hints" array should not contain task "TASK-HINT-021"
    And the "hints" array should not contain task "TASK-HINT-023"

  @AC-TPL-AGENT-HINTS
  Scenario: Each hint includes task id, status, requirement, ACs, and reason
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-030     | Update dependencies       | Todo        | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the first hint should have field "task_id"
    And the first hint should have field "status"
    And the first hint should have field "requirement_ids"
    And the first hint should have field "ac_ids"
    And the first hint should have field "reason"
    And the first hint should have field "recommended_sequence"
    And the first hint "task_id" should equal "TASK-HINT-030"
    And the first hint "status" should equal "Todo"
    And the first hint "reason" should contain "ready for work"

  @AC-TPL-AGENT-HINTS
  Scenario: Hints include task title in reason
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-040     | Implement user login      | InProgress  | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the first hint "reason" should contain "Implement user login"

  @AC-TPL-AGENT-HINTS
  Scenario: Empty hints when no tasks are Todo or InProgress
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-050     | Complete milestone        | Done        | REQ-TPL-001          |
      | TASK-HINT-051     | Review architecture       | Review      | REQ-TPL-002          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON should have an empty hints array

