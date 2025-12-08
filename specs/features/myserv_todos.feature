@domain @myserv
Feature: Todo Management
  As a user
  I want to manage my todos through the API
  So that I can track my tasks

  Background:
    Given the service is running

  @AC-MYSERV-001
  Scenario: GET /todos returns a JSON array of todos
    Given the user has existing todos
    When I send a GET request to "/todos"
    Then the response status should be 200
    And the response should be a JSON array
    And each todo should have an "id" and "title" field

  @AC-MYSERV-002
  Scenario: GET /todos returns empty array when no todos exist
    Given the user has no todos
    When I send a GET request to "/todos"
    Then the response status should be 200
    And the response should be a JSON array
    And the response array should be empty

  @AC-MYSERV-003
  Scenario: POST /todos with invalid JSON returns 400
    When I send a POST request to "/todos" with invalid JSON
    Then the response status should be 400
    And the response should contain an error message

  @AC-MYSERV-003
  Scenario: POST /todos with missing required field returns 400
    When I send a POST request to "/todos" with body:
      """
      {"id": "todo-3"}
      """
    Then the response status should be 400
    And the response should contain an error message

  @AC-MYSERV-004
  Scenario: DELETE /todos/:id removes the todo from the list
    Given the user has existing todos
    When I send a DELETE request to "/todos/todo-1"
    Then the response status should be 204
    When I send a GET request to "/todos"
    Then the response status should be 200
    And the response should be a JSON array
    And the todo with id "todo-1" should not be in the list

  @AC-MYSERV-004
  Scenario: DELETE /todos/:id with non-existent id returns 404
    When I send a DELETE request to "/todos/non-existent-id"
    Then the response status should be 404

  @AC-MYSERV-005
  Scenario: POST /todos with duplicate ID returns 409 Conflict
    Given the user has existing todos
    When I send a POST request to "/todos" with body:
      """
      {"id": "todo-1", "title": "Duplicate ID attempt"}
      """
    Then the response status should be 409
    And the response should contain an error message

  @AC-MYSERV-006
  Scenario: POST /todos with empty title returns 400
    When I send a POST request to "/todos" with body:
      """
      {"id": "todo-new", "title": ""}
      """
    Then the response status should be 400
    And the response should contain an error message

  @AC-MYSERV-006
  Scenario: POST /todos with title exceeding 256 characters returns 400
    When I send a POST request to "/todos" with body:
      """
      {"id": "todo-long", "title": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
      """
    Then the response status should be 400
    And the response should contain an error message
