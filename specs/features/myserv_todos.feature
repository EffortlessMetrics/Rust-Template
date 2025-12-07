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
