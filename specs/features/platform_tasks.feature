Feature: Platform Tasks API

  Background:
    Given the platform is running

  @AC-TPL-TASK-TRANSITIONS
  Scenario: Update task status via API
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
  Scenario: Invalid task transition
    Given a task "TASK-002" exists with status "Done"
    When I send a POST request to "/platform/tasks/TASK-002/status" with body:
      """
      {
        "status": "Todo"
      }
      """
    Then the response status code should be 500
