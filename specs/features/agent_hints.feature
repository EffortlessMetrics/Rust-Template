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
  Scenario: Each hint includes task id, title, status, owner, labels, requirement, ACs, and reason
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-030     | Update dependencies       | Todo        | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the first hint should have field "task_id"
    And the first hint should have field "title"
    And the first hint should have field "status"
    And the first hint should have field "owner"
    And the first hint should have field "labels"
    And the first hint should have field "requirement_ids"
    And the first hint should have field "ac_ids"
    And the first hint should have field "reason"
    And the first hint should have field "recommended_sequence"
    And the first hint "task_id" should equal "TASK-HINT-030"
    And the first hint "status" should equal "open"
    And the first hint "reason" should not be empty

  @AC-TPL-AGENT-HINTS
  Scenario: Hints include meaningful reason information
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-040     | Implement user login      | InProgress  | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the first hint "reason" should not be empty

  @AC-TPL-AGENT-HINTS
  Scenario: Empty hints when no tasks are Todo or InProgress
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-050     | Complete milestone        | Done        | REQ-TPL-001          |
      | TASK-HINT-051     | Review architecture       | Review      | REQ-TPL-002          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON should have an empty hints array

  @AC-TPL-AGENT-HINTS
  Scenario: Hints include recommended_sequence with more than 2 commands from flows
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-060     | Implement new feature     | Todo        | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the first hint should have field "recommended_sequence"
    And the first hint "recommended_sequence" should have more than 2 items

  @AC-TPL-AGENT-HINTS
  Scenario: Hints include stable metadata (title, owner, labels) from tasks.yaml
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           | owner         | labels                        |
      | TASK-HINT-070     | Implement auth module     | InProgress  | REQ-TPL-001          | team-backend  | security,authentication,v3    |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the first hint should have field "title"
    And the first hint should have field "owner"
    And the first hint should have field "labels"
    And the first hint "title" should equal "Implement auth module"
    And the first hint "owner" should equal "team-backend"
    And the first hint "labels" array should contain "security"
    And the first hint "labels" array should contain "authentication"
    And the first hint "labels" array should contain "v3"

  @AC-TPL-AGENT-HINTS
  Scenario: Filter hints by owner
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | owner     |
      | TASK-HINT-080     | Implement auth            | Todo        | REQ-TPL-001   | alice     |
      | TASK-HINT-081     | Write tests               | InProgress  | REQ-TPL-002   | bob       |
      | TASK-HINT-082     | Update docs               | Todo        | REQ-TPL-003   | alice     |
    When I send a GET request to "/platform/agent/hints?owner=alice"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the "hints" array should contain task "TASK-HINT-080"
    And the "hints" array should contain task "TASK-HINT-082"
    And the "hints" array should not contain task "TASK-HINT-081"

  @AC-TPL-AGENT-HINTS
  Scenario: Filter hints by label
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | labels        |
      | TASK-HINT-090     | Fix security bug          | Todo        | REQ-TPL-001   | security      |
      | TASK-HINT-091     | Add feature               | InProgress  | REQ-TPL-002   | feature       |
      | TASK-HINT-092     | Security audit            | Todo        | REQ-TPL-003   | security,audit|
    When I send a GET request to "/platform/agent/hints?label=security"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the "hints" array should contain task "TASK-HINT-090"
    And the "hints" array should contain task "TASK-HINT-092"
    And the "hints" array should not contain task "TASK-HINT-091"

  @AC-TPL-AGENT-HINTS
  Scenario: Filter hints by requirement
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-100     | Implement endpoint        | Todo        | REQ-TPL-HEALTH        |
      | TASK-HINT-101     | Add metrics               | InProgress  | REQ-TPL-METRICS       |
      | TASK-HINT-102     | Update health check       | Todo        | REQ-TPL-HEALTH        |
    When I send a GET request to "/platform/agent/hints?requirement=REQ-TPL-HEALTH"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the "hints" array should contain task "TASK-HINT-100"
    And the "hints" array should contain task "TASK-HINT-102"
    And the "hints" array should not contain task "TASK-HINT-101"

  @AC-TPL-AGENT-HINTS
  Scenario: Hints sorted by status then ID
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-HINT-110     | Task A                    | Todo        | REQ-TPL-001          |
      | TASK-HINT-111     | Task B                    | InProgress  | REQ-TPL-002          |
      | TASK-HINT-112     | Task C                    | Todo        | REQ-TPL-003          |
      | TASK-HINT-113     | Task D                    | InProgress  | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the hints should be sorted with "in_progress" before "open"
    And within same status hints should be sorted by id

  @AC-TPL-AGENT-HINTS
  Scenario: Hints sorted by priority labels
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | labels            |
      | TASK-HINT-120     | Low priority task         | Todo        | REQ-TPL-001   | priority:low      |
      | TASK-HINT-121     | High priority task        | Todo        | REQ-TPL-002   | priority:high     |
      | TASK-HINT-122     | Medium priority task      | Todo        | REQ-TPL-003   | priority:medium   |
      | TASK-HINT-123     | No priority task          | Todo        | REQ-TPL-001   |                   |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the hints should be sorted by priority: high, medium, low, none

  # CLI tests for suggest-next command (unified HintEngine)
  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next returns hints in JSON format
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           | owner    |
      | TASK-CLI-001      | Setup database            | Todo        | REQ-TPL-001          | alice    |
      | TASK-CLI-002      | Write tests               | InProgress  | REQ-TPL-002          | bob      |
      | TASK-CLI-003      | Deploy to prod            | Done        | REQ-TPL-003          | alice    |
    When I run the command "cargo xtask suggest-next --format json"
    Then the exit code should be 0
    And the output should be valid JSON
    And the JSON output should have field "hints"
    And the "hints" array should contain task "TASK-CLI-001"
    And the "hints" array should contain task "TASK-CLI-002"
    And the "hints" array should not contain task "TASK-CLI-003"

  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next filters by owner
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | owner     |
      | TASK-CLI-010      | Frontend work             | Todo        | REQ-TPL-001   | alice     |
      | TASK-CLI-011      | Backend work              | InProgress  | REQ-TPL-002   | bob       |
      | TASK-CLI-012      | Security audit            | Todo        | REQ-TPL-003   | alice     |
    When I run the command "cargo xtask suggest-next --owner alice --format json"
    Then the exit code should be 0
    And the JSON output should have field "hints"
    And the "hints" array should contain task "TASK-CLI-010"
    And the "hints" array should contain task "TASK-CLI-012"
    And the "hints" array should not contain task "TASK-CLI-011"

  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next filters by label
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | labels           |
      | TASK-CLI-020      | Fix auth bypass           | Todo        | REQ-TPL-001   | security         |
      | TASK-CLI-021      | Add feature               | InProgress  | REQ-TPL-002   | feature          |
      | TASK-CLI-022      | Security review           | Todo        | REQ-TPL-003   | security,audit   |
    When I run the command "cargo xtask suggest-next --label security --format json"
    Then the exit code should be 0
    And the JSON output should have field "hints"
    And the "hints" array should contain task "TASK-CLI-020"
    And the "hints" array should contain task "TASK-CLI-022"
    And the "hints" array should not contain task "TASK-CLI-021"

  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next filters by requirement
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-CLI-030      | Implement health check    | Todo        | REQ-TPL-HEALTH        |
      | TASK-CLI-031      | Add metrics               | InProgress  | REQ-TPL-METRICS       |
      | TASK-CLI-032      | Update health check       | Todo        | REQ-TPL-HEALTH        |
    When I run the command "cargo xtask suggest-next --requirement REQ-TPL-HEALTH --format json"
    Then the exit code should be 0
    And the JSON output should have field "hints"
    And the "hints" array should contain task "TASK-CLI-030"
    And the "hints" array should contain task "TASK-CLI-032"
    And the "hints" array should not contain task "TASK-CLI-031"

  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next respects limit parameter
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-CLI-040      | Task A                    | Todo        | REQ-TPL-001          |
      | TASK-CLI-041      | Task B                    | Todo        | REQ-TPL-002          |
      | TASK-CLI-042      | Task C                    | Todo        | REQ-TPL-003          |
      | TASK-CLI-043      | Task D                    | Todo        | REQ-TPL-004          |
    When I run the command "cargo xtask suggest-next --limit 2 --format json"
    Then the exit code should be 0
    And the JSON output should have field "hints"
    And the "hints" array should have 2 items

  @AC-TPL-AGENT-HINTS
  Scenario: CLI suggest-next outputs structured hints with all required fields
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement   | owner         | labels              |
      | TASK-CLI-050      | Implement feature         | InProgress  | REQ-TPL-001   | team-backend  | feature,high        |
    When I run the command "cargo xtask suggest-next --format json"
    Then the exit code should be 0
    And the JSON output should have field "hints"
    And the first hint in JSON should have field "task_id"
    And the first hint in JSON should have field "title"
    And the first hint in JSON should have field "status"
    And the first hint in JSON should have field "owner"
    And the first hint in JSON should have field "labels"
    And the first hint in JSON should have field "requirement_ids"
    And the first hint in JSON should have field "ac_ids"
    And the first hint in JSON should have field "reason"
    And the first hint "task_id" should equal "TASK-CLI-050"
    And the first hint "status" should equal "in_progress"
    And the first hint "owner" should equal "team-backend"

  @AC-TPL-AGENT-HINTS-SCHEMA
  Scenario: HTTP hints include all required schema fields
    Given the following tasks exist in "specs/tasks.yaml":
      | id                | title                     | status      | requirement           |
      | TASK-SCHEMA-001   | Schema validation task    | Todo        | REQ-TPL-001          |
    When I send a GET request to "/platform/agent/hints"
    Then the response status code should be 200
    And the JSON response should have field "hints"
    And the first hint should have field "id"
    And the first hint should have field "kind"
    And the first hint should have field "priority"
    And the first hint should have field "status"
    And the first hint should have field "reason"
    And the first hint should have field "target"
    And the first hint should have field "tags"
    And the first hint should have field "links"

